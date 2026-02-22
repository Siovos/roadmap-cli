# Référence des commandes

Documentation complète de toutes les commandes roadmap-cli.

---

## Commandes globales

### `roadmap init`

Initialise une roadmap dans le projet courant.

```bash
roadmap init
```

Crée :
- `.phases/config.yml` — Configuration du projet

---

### `roadmap report`

Affiche un rapport de progression.

```bash
roadmap report [--json]
```

| Option | Description |
|--------|-------------|
| `--json` | Sortie JSON pour parsing |

**Exemple :**
```bash
roadmap report
```

**Sortie JSON :**
```json
{
  "summary": {
    "total_phases": 6,
    "phases_done": 3,
    "phases_in_progress": 1,
    "progress_percent": 33.33
  },
  "phases_in_progress": [...],
  "next_tasks": [...],
  "optional_tasks": [...],
  "blocked": { "phases": [], "tasks": [] }
}
```

---

### `roadmap list`

Liste toutes les phases.

```bash
roadmap list [--table] [--json]
```

| Option | Description |
|--------|-------------|
| `--table` | Affichage tableau formaté |
| `--json` | Sortie JSON |

**Exemples :**
```bash
roadmap list           # Liste simple colorée
roadmap list --table   # Tableau ASCII
roadmap list --json    # JSON complet
```

---

### `roadmap tree`

Affiche l'arborescence complète.

```bash
roadmap tree [--json]
```

| Option | Description |
|--------|-------------|
| `--json` | Sortie JSON hiérarchique |

**Sortie JSON :**
```json
[
  {
    "id": "9",
    "name": "Observabilité",
    "type": "phase",
    "status": "in_progress",
    "children": [
      { "id": "9.1", "type": "task", "status": "done", "children": [...] }
    ]
  }
]
```

---

### `roadmap ui`

Lance l'interface interactive (TUI).

```bash
roadmap ui
```

**Raccourcis :**
| Touche | Action |
|--------|--------|
| `↑↓` ou `jk` | Navigation |
| `→` ou `l` | Aller aux tâches |
| `←` ou `h` | Retour aux phases |
| `Tab` / `Enter` | Basculer focus |
| `q` / `Esc` | Quitter |

---

### `roadmap export`

Génère le fichier Markdown.

```bash
roadmap export
```

Le chemin de sortie est défini dans `.phases/config.yml` → `export.roadmap_path`.

---

## Gestion des phases

### `roadmap add`

Crée une nouvelle phase.

```bash
roadmap add <id> <nom> [--parent <phase_id>]
```

| Argument | Description |
|----------|-------------|
| `<id>` | Identifiant de la phase (ex: 9, 10, 9a) |
| `<nom>` | Nom de la phase |
| `--parent` | Phase parente (pour sous-phases) |

**Exemples :**
```bash
roadmap add 9 "Observabilité"
roadmap add 9a "Logs Pino" --parent 9
```

---

### `roadmap show`

Affiche les détails d'une phase.

```bash
roadmap show <id> [--json]
```

| Option | Description |
|--------|-------------|
| `--json` | Sortie JSON complète |

**Exemple :**
```bash
roadmap show 9
```

---

### `roadmap edit`

Modifie une phase.

```bash
roadmap edit <id> [--name <nom>] [--description <desc>]
```

| Option | Description |
|--------|-------------|
| `--name` | Nouveau nom |
| `--description` | Nouvelle description |

**Exemple :**
```bash
roadmap edit 9 --name "Observabilité & Monitoring" --description "Logs, métriques, alerting"
```

---

### `roadmap priority`

Change la priorité d'une phase.

```bash
roadmap priority <id> --set <n>
```

| Option | Description |
|--------|-------------|
| `--set` | Nouvelle priorité (1 = haute) |

**Exemple :**
```bash
roadmap priority 9 --set 1
```

---

### `roadmap status`

Change le statut d'une phase.

```bash
roadmap status <id> --set <statut>
```

| Statut | Description |
|--------|-------------|
| `pending` | À faire |
| `in_progress` | En cours |
| `done` | Terminé |
| `blocked` | Bloqué |

**Exemple :**
```bash
roadmap status 9 --set in_progress
```

---

### `roadmap note`

Ajoute une note à une phase.

```bash
roadmap note <id> <contenu>
```

**Exemple :**
```bash
roadmap note 9 "Décision: utiliser Pino pour la performance"
```

---

## Gestion des tâches

### `roadmap task add`

Ajoute une tâche à une phase.

```bash
roadmap task add <phase_id> <nom> [--parent <task_id>] [--optional]
```

| Option | Description |
|--------|-------------|
| `--parent` | Tâche parente (pour sous-tâches) |
| `--optional` | Marquer comme optionnelle |

**Exemples :**
```bash
roadmap task add 9 "Intégrer Pino"
roadmap task add 9 "Configurer transport" --parent 9.1
roadmap task add 9 "GlitchTip" --optional
```

---

### `roadmap task edit`

Modifie une tâche.

```bash
roadmap task edit <task_id> [--name <nom>] [--description <desc>] [--optional <bool>]
```

| Option | Description |
|--------|-------------|
| `--name` | Nouveau nom |
| `--description` | Nouvelle description |
| `--optional` | `true` ou `false` |

**Exemple :**
```bash
roadmap task edit 9.1 --name "Logs structurés Pino" --description "Remplacer console.error"
```

---

### `roadmap task start`

Marque une tâche comme en cours.

```bash
roadmap task start <task_id>
```

---

### `roadmap task done`

Marque une tâche comme terminée.

```bash
roadmap task done <task_id>
```

---

### `roadmap task move`

Déplace une tâche vers une autre phase.

```bash
roadmap task move <task_id> --to <phase_id>
```

**Exemple :**
```bash
roadmap task move 9.3 --to 10
# ✓ Tâche 9.3 → 10 (nouvel ID: 10.1)
```

---

## Gestion du workflow

### `roadmap workflow`

Gère l'étape workflow d'une tâche.

```bash
roadmap workflow <task_id> --advance
roadmap workflow <task_id> --set <stage>
```

| Option | Description |
|--------|-------------|
| `--advance` | Passe à l'étape suivante |
| `--set` | Force une étape spécifique |

**Étapes par défaut :**
`analysis` → `design` → `implementation` → `testing` → `documentation`

**Exemples :**
```bash
roadmap workflow 9.1 --advance
# → Tâche 9.1 : analysis → design

roadmap workflow 9.1 --set testing
# ✓ Tâche 9.1 : design → testing
```

---

## Options globales

| Option | Description |
|--------|-------------|
| `--help` | Affiche l'aide |
| `--version` | Affiche la version |

```bash
roadmap --help
roadmap task --help
roadmap task add --help
```
