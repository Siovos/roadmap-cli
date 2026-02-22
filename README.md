# roadmap-cli

CLI pour gérer les roadmaps projet avec phases, tâches et workflows.

**Inspiré par Jira**, mais en CLI, simple, et intégré au workflow développeur.

## Pourquoi roadmap-cli ?

- **Versionnable** : Les données sont des fichiers YAML dans `.phases/`
- **Lisible** : Export Markdown automatique
- **Intégrable** : Sortie JSON pour scripts et AI
- **Léger** : Binaire unique de 2 MB

## Installation

```bash
# Homebrew (macOS/Linux)
brew tap siovos/tap
brew install roadmap-cli

# Script d'installation
curl -fsSL https://raw.githubusercontent.com/siovos/roadmap-cli/main/install.sh | bash

# Cargo (si Rust installé)
cargo install --git https://github.com/siovos/roadmap-cli
```

## Démarrage rapide

```bash
# Initialiser dans un projet
roadmap init

# Créer une phase
roadmap add 1 "MVP"

# Ajouter des tâches
roadmap task add 1 "Setup projet"
roadmap task add 1 "Implémenter API"
roadmap task done 1.1

# Voir l'état
roadmap report
roadmap tree
```

## Aperçu

```
📋 Roadmap

├── 🔄 Phase 9 — Observabilité & Infra (P1)
│   ├── ✅ 9.1 — Logs structurés Pino
│   ├── ⬜ 9.2 — Métriques Prometheus/Grafana
│   └── ⬜ 9.3 — Mode debug utilisateur
├── ✅ Phase 10 — Modèle Sémantique
└── ⬜ Phase 11 — Traçabilité Décisions (ADR)
```

## Documentation

| Document | Description |
|----------|-------------|
| [Guide de démarrage](docs/getting-started.md) | Tutoriel pas à pas |
| [Référence des commandes](docs/commands.md) | Toutes les commandes avec options |
| [Format des données](docs/data-format.md) | Structure YAML des fichiers |
| [Exemples](docs/examples.md) | Cas d'usage concrets |

## Commandes principales

| Commande | Description |
|----------|-------------|
| `roadmap init` | Initialiser une roadmap |
| `roadmap add <id> <nom>` | Créer une phase |
| `roadmap task add <phase> <nom>` | Ajouter une tâche |
| `roadmap task done <id>` | Terminer une tâche |
| `roadmap report` | Rapport de progression |
| `roadmap tree` | Vue arborescente |
| `roadmap ui` | Interface interactive |
| `roadmap export` | Générer Markdown |

## License

MIT
