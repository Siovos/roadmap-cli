# CLAUDE.md — roadmap-cli

Point d'entrée pour comprendre et utiliser roadmap-cli.

---

## Qu'est-ce que roadmap-cli ?

CLI pour gérer les roadmaps projet avec :
- **Phases** numérotées avec priorité et statut
- **Sous-phases** (ex: 9a, 9b) avec parent référencé
- **Tâches** avec workflow optionnel et sous-tâches
- **Export** Markdown et JSON

**Données** : Fichiers YAML dans `.phases/` (versionnable avec git).

---

## Commandes essentielles

### Voir l'état du projet

```bash
# Rapport de progression (RECOMMANDÉ pour commencer)
roadmap report

# Vue arborescente
roadmap tree

# Liste des phases
roadmap list --table

# Détails d'une phase
roadmap show 9
```

### Créer et modifier

```bash
# Phase
roadmap add 9 "Observabilité" --description "..." --priority 1 --status in_progress
roadmap edit 9 --name "Observabilité & Infra"
roadmap status 9 --set in_progress
roadmap priority 9 1              # Deux positionnels (plus naturel)
roadmap priority 9 --set 1        # Ou avec --set (compatible)
roadmap remove 9                  # Supprimer (avec confirmation)
roadmap remove 9 -y               # Supprimer sans confirmation
roadmap remove 9 -y --force       # Forcer si sous-phases

# Tâche
roadmap task add 9 "Logs Pino"
roadmap task add 9 "Métriques" --edit  # Ouvre $EDITOR pour la description
roadmap task start 9.1
roadmap task done 9.1
roadmap task edit 9.1 --name "Logs structurés Pino"
roadmap task remove 9.1               # Supprimer (avec confirmation)
roadmap task remove 9.1 9.2 -y        # Supprimer plusieurs sans confirmation

# Import en masse
roadmap import 9 tasks.yml         # Depuis un fichier YAML
cat tasks.yml | roadmap import 9 - # Depuis stdin

# Notes
roadmap note 9 "Décision: utiliser Pino"  # Ajouter
roadmap note 9 --list                     # Lister avec index
roadmap note 9 "Corrigé" --edit 2         # Éditer par index
roadmap note 9 --remove 1                 # Supprimer par index
```

### Format d'import YAML

```yaml
# Forme simple (nom seul)
- "Setup CI"
- "Tests unitaires"

# Forme complète
- name: "Deploy staging"
  description: "Configurer le pipeline"
  optional: true
  tags: [backend, infra]
  assignee: "dev-team"
```

### Sortie JSON (pour parsing)

```bash
roadmap report --json    # Résumé structuré
roadmap list --json      # Toutes les phases
roadmap tree --json      # Arborescence hiérarchique
roadmap show 9 --json    # Détails d'une phase
```

---

## Format JSON du rapport

```bash
roadmap report --json
```

```json
{
  "summary": {
    "total_phases": 6,
    "phases_done": 3,
    "phases_in_progress": 1,
    "phases_pending": 2,
    "total_tasks": 4,
    "tasks_done": 1,
    "tasks_pending": 2,
    "tasks_optional": 1,
    "progress_percent": 33.33
  },
  "phases_in_progress": [
    { "id": "9", "name": "Observabilité & Infra", "priority": 1, "progress": "1/4" }
  ],
  "next_tasks": [
    { "id": "9.2", "name": "Métriques Prometheus/Grafana", "phase_id": "9", "status": "pending" }
  ],
  "optional_tasks": [
    { "id": "9.4", "name": "GlitchTip error tracking", "optional": true }
  ],
  "blocked": { "phases": [], "tasks": [] }
}
```

---

## Structure des fichiers

```
projet/
├── .phases/
│   ├── config.yml       # Configuration projet
│   ├── phase-9.yml      # Phase 9
│   ├── phase-9a.yml     # Sous-phase 9a
│   └── ...
└── ROADMAP.md           # Généré par `roadmap export`
```

### config.yml

```yaml
project:
  name: "siovos-archi"
  description: "Application SaaS de visualisation d'architectures"

default_workflow:
  enabled: true
  stages:
    - id: analysis
      name: "Analyse"
    - id: implementation
      name: "Développement"
    - id: testing
      name: "Tests"

export:
  roadmap_path: ./ROADMAP.md
```

### phase-9.yml

```yaml
id: "9"
name: "Observabilité & Infra"
priority: 1
status: in_progress
parent: null

tasks:
  - id: "9.1"
    name: "Logs structurés Pino"
    status: done
    optional: false
  - id: "9.2"
    name: "Métriques Prometheus/Grafana"
    status: pending
    optional: false

notes:
  - date: 2026-02-23
    content: "Pino intégré dans toutes les routes API"
```

---

## Statuts disponibles

| Statut | Icône | Description |
|--------|-------|-------------|
| `pending` | ⬜ | À faire |
| `in_progress` | 🔄 | En cours |
| `done` | ✅ | Terminé |
| `blocked` | 🚫 | Bloqué |

---

## Workflows

Les tâches peuvent suivre un workflow d'étapes :

```bash
# Avancer à l'étape suivante
roadmap workflow 9.1 --advance

# Forcer une étape spécifique
roadmap workflow 9.1 --set testing
```

Étapes par défaut : `analysis` → `design` → `implementation` → `testing` → `documentation`

---

## Bonnes pratiques

1. **Commencer par `roadmap report`** pour voir l'état actuel
2. **Utiliser `--json`** pour parser les données programmatiquement
3. **Phases = grandes fonctionnalités**, tâches = étapes concrètes
4. **Marquer `--optional`** les tâches "nice to have"
5. **Ajouter des notes** pour documenter les décisions

---

## Commandes utiles pour AI

```bash
# État complet en JSON
roadmap report --json

# Prochaines tâches à faire
roadmap report --json | jq '.next_tasks'

# Phases en cours
roadmap report --json | jq '.phases_in_progress'

# Tâches optionnelles mises de côté
roadmap report --json | jq '.optional_tasks'
```

---

*Dernière mise à jour : 23 février 2026*
