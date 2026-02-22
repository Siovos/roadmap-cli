use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::phase::Phase;

pub struct App {
    phases: Vec<Phase>,
    phase_state: ListState,
    task_state: ListState,
    focus: Focus,
    should_quit: bool,
}

#[derive(PartialEq)]
enum Focus {
    Phases,
    Tasks,
}

impl App {
    pub fn new(phases: Vec<Phase>) -> Self {
        let mut phase_state = ListState::default();
        if !phases.is_empty() {
            phase_state.select(Some(0));
        }

        App {
            phases,
            phase_state,
            task_state: ListState::default(),
            focus: Focus::Phases,
            should_quit: false,
        }
    }

    fn selected_phase(&self) -> Option<&Phase> {
        self.phase_state.selected().and_then(|i| self.phases.get(i))
    }

    fn next_phase(&mut self) {
        if self.phases.is_empty() {
            return;
        }
        let i = match self.phase_state.selected() {
            Some(i) => {
                if i >= self.phases.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.phase_state.select(Some(i));
        self.task_state.select(None);
    }

    fn prev_phase(&mut self) {
        if self.phases.is_empty() {
            return;
        }
        let i = match self.phase_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.phases.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.phase_state.select(Some(i));
        self.task_state.select(None);
    }

    fn next_task(&mut self) {
        if let Some(phase) = self.selected_phase() {
            if phase.tasks.is_empty() {
                return;
            }
            let i = match self.task_state.selected() {
                Some(i) => {
                    if i >= phase.tasks.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.task_state.select(Some(i));
        }
    }

    fn prev_task(&mut self) {
        if let Some(phase) = self.selected_phase() {
            if phase.tasks.is_empty() {
                return;
            }
            let i = match self.task_state.selected() {
                Some(i) => {
                    if i == 0 {
                        phase.tasks.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.task_state.select(Some(i));
        }
    }

    fn toggle_focus(&mut self) {
        match self.focus {
            Focus::Phases => {
                if let Some(phase) = self.selected_phase() {
                    if !phase.tasks.is_empty() {
                        self.focus = Focus::Tasks;
                        if self.task_state.selected().is_none() {
                            self.task_state.select(Some(0));
                        }
                    }
                }
            }
            Focus::Tasks => {
                self.focus = Focus::Phases;
            }
        }
    }
}

pub fn run_tui(phases: Vec<Phase>) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Create app
    let mut app = App::new(phases);

    // Main loop
    while !app.should_quit {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => {
                            match app.focus {
                                Focus::Phases => app.next_phase(),
                                Focus::Tasks => app.next_task(),
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            match app.focus {
                                Focus::Phases => app.prev_phase(),
                                Focus::Tasks => app.prev_task(),
                            }
                        }
                        KeyCode::Tab | KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                            app.toggle_focus();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if app.focus == Focus::Tasks {
                                app.focus = Focus::Phases;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ])
        .split(main_layout[0]);

    // Left panel - Phases list
    render_phases_list(frame, app, content_layout[0]);

    // Right panel - Details
    render_details(frame, app, content_layout[1]);

    // Bottom - Help
    render_help(frame, main_layout[1]);
}

fn get_status_icon(status: &str) -> &'static str {
    match status {
        "done" => "✅",
        "in_progress" => "🔄",
        "blocked" => "🚫",
        _ => "⬜",
    }
}

fn render_phases_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .phases
        .iter()
        .map(|phase| {
            let icon = get_status_icon(&phase.status);
            let prefix = if phase.parent.is_some() { "  └─ " } else { "" };
            let line = format!("{}{} {} — {}", prefix, icon, phase.id, phase.name);
            ListItem::new(line)
        })
        .collect();

    let border_style = if app.focus == Focus::Phases {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(" 📋 Phases ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.phase_state);
}

fn render_details(frame: &mut Frame, app: &mut App, area: Rect) {
    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Tasks
            Constraint::Length(6),  // Notes
        ])
        .split(area);

    // Extrait les données nécessaires avant le rendu pour éviter les problèmes de borrow
    let phase_data = app.phase_state.selected().and_then(|i| {
        app.phases.get(i).map(|phase| {
            let header_text = format!(
                "Phase {} — {}\n\nPriorité: {}  │  Statut: {}  │  Tâches: {}/{}",
                phase.id,
                phase.name,
                phase.priority,
                phase.status,
                phase.tasks.iter().filter(|t| t.status == "done").count(),
                phase.tasks.len()
            );

            let task_items: Vec<ListItem> = phase
                .tasks
                .iter()
                .map(|task| {
                    let icon = get_status_icon(&task.status);
                    let optional = if task.optional { " (opt)" } else { "" };
                    let indent = if task.parent.is_some() { "    " } else { "" };
                    let line = format!("{}{} {} — {}{}", indent, icon, task.id, task.name, optional);
                    ListItem::new(line)
                })
                .collect();

            let notes_text = if phase.notes.is_empty() {
                String::from("Aucune note")
            } else {
                phase
                    .notes
                    .iter()
                    .map(|n| format!("• {} : {}", n.date, n.content))
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            (header_text, task_items, notes_text)
        })
    });

    if let Some((header_text, task_items, notes_text)) = phase_data {
        // Header
        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .title(" 📄 Détails ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(header, details_layout[0]);

        // Tasks
        let task_border_style = if app.focus == Focus::Tasks {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let task_list = List::new(task_items)
            .block(
                Block::default()
                    .title(" ☑️  Tâches ")
                    .borders(Borders::ALL)
                    .border_style(task_border_style),
            )
            .highlight_style(Style::default().bg(Color::DarkGray).bold())
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(task_list, details_layout[1], &mut app.task_state);

        // Notes
        let notes = Paragraph::new(notes_text)
            .block(
                Block::default()
                    .title(" 📝 Notes ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(notes, details_layout[2]);
    } else {
        let empty = Paragraph::new("Sélectionnez une phase")
            .block(
                Block::default()
                    .title(" 📄 Détails ")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center);

        frame.render_widget(empty, area);
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = " ↑↓/jk: Navigation │ →/l/Tab/Enter: Tâches │ ←/h: Phases │ q/Esc: Quitter ";

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(help, area);
}
