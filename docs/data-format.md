# Format des données

Structure des fichiers YAML utilisés par roadmap-cli.

---

## Structure des fichiers

```
projet/
└── .phases/
    ├── config.yml          # Configuration globale
    ├── phase-9.yml         # Phase 9
    ├── phase-9a.yml        # Sous-phase 9a
    ├── phase-10.yml        # Phase 10
    └── ...
```

---

## config.yml

Configuration globale du projet.

```yaml
project:
  name: "siovos-archi"
  description: "Application SaaS de visualisation d'architectures"

default_workflow:
  enabled: true
  stages:
    - id: analysis
      name: "Analyse"
    - id: design
      name: "Conception"
    - id: implementation
      name: "Développement"
    - id: testing
      name: "Tests"
    - id: documentation
      name: "Documentation"

statuses:
  - id: pending
    label: "À faire"
    icon: "⬜"
  - id: in_progress
    label: "En cours"
    icon: "🔄"
  - id: done
    label: "Terminé"
    icon: "✅"
  - id: blocked
    label: "Bloqué"
    icon: "🚫"

export:
  roadmap_path: "./ROADMAP.md"
  phases_docs_path: "./docs"
```

### Champs

| Champ | Type | Description |
|-------|------|-------------|
| `project.name` | string | Nom du projet |
| `project.description` | string | Description courte |
| `default_workflow.enabled` | bool | Activer le workflow par défaut |
| `default_workflow.stages` | array | Liste des étapes |
| `statuses` | array | Statuts personnalisés |
| `export.roadmap_path` | string | Chemin du fichier exporté |

---

## phase-{id}.yml

Définition d'une phase.

```yaml
id: "9"
name: "Observabilité & Infra"
description: "Logs structurés, métriques, monitoring"
priority: 1
status: in_progress
parent: null
created_at: "2026-02-23"
updated_at: "2026-02-23"

workflow:
  enabled: true
  stages:
    - id: analysis
      name: "Analyse"
    - id: implementation
      name: "Implémentation"

tasks:
  - id: "9.1"
    name: "Logs structurés Pino"
    description: "Remplacer console.error par Pino"
    status: done
    parent: null
    workflow_stage: documentation
    optional: false
    completed_at: "2026-02-23"

  - id: "9.2"
    name: "Métriques Prometheus/Grafana"
    description: null
    status: pending
    parent: null
    workflow_stage: null
    optional: false
    completed_at: null

  - id: "9.1.1"
    name: "Configurer transport"
    description: null
    status: done
    parent: "9.1"
    workflow_stage: null
    optional: false
    completed_at: "2026-02-23"

notes:
  - date: "2026-02-23"
    content: "Pino intégré dans toutes les routes API (274 console.error migrés)"
  - date: "2026-02-22"
    content: "Choix Pino pour la performance"
```

### Champs de phase

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `id` | string | ✓ | Identifiant unique |
| `name` | string | ✓ | Nom de la phase |
| `description` | string | | Description détaillée |
| `priority` | int | | Priorité (1 = haute, défaut: 10) |
| `status` | string | | `pending`, `in_progress`, `done`, `blocked` |
| `parent` | string | | ID de la phase parente |
| `created_at` | string | | Date de création (YYYY-MM-DD) |
| `updated_at` | string | | Date de modification |
| `workflow` | object | | Workflow spécifique à la phase |
| `tasks` | array | | Liste des tâches |
| `notes` | array | | Notes et décisions |

### Champs de tâche

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `id` | string | ✓ | Identifiant (ex: 9.1, 9.1.1) |
| `name` | string | ✓ | Nom de la tâche |
| `description` | string | | Description détaillée |
| `status` | string | | `pending`, `in_progress`, `done`, `blocked` |
| `parent` | string | | ID de la tâche parente (sous-tâche) |
| `workflow_stage` | string | | Étape actuelle du workflow |
| `optional` | bool | | Tâche optionnelle (défaut: false) |
| `completed_at` | string | | Date de complétion |

### Champs de note

| Champ | Type | Description |
|-------|------|-------------|
| `date` | string | Date (YYYY-MM-DD) |
| `content` | string | Contenu de la note |

---

## Sous-phases

Une sous-phase est une phase avec un `parent` :

```yaml
# .phases/phase-9a.yml
id: "9a"
name: "Logs Pino"
description: "Intégration des logs structurés"
priority: 10
status: pending
parent: "9"              # ← Référence à la phase parente
created_at: "2026-02-23"
updated_at: "2026-02-23"
tasks: []
notes: []
```

Les sous-phases apparaissent indentées dans `roadmap tree`.

---

## Sous-tâches

Une sous-tâche est une tâche avec un `parent` :

```yaml
tasks:
  - id: "9.1"
    name: "Intégrer Pino"
    status: in_progress
    parent: null          # ← Tâche racine

  - id: "9.1.1"
    name: "Configurer transport"
    status: done
    parent: "9.1"         # ← Sous-tâche de 9.1

  - id: "9.1.2"
    name: "Ajouter contexte request"
    status: pending
    parent: "9.1"         # ← Sous-tâche de 9.1
```

---

## Workflow personnalisé

Chaque phase peut avoir son propre workflow :

```yaml
workflow:
  enabled: true
  stages:
    - id: analysis
      name: "Analyse du besoin"
    - id: codebase
      name: "Analyse codebase"
    - id: implementation
      name: "Implémentation"
    - id: review
      name: "Code review"
    - id: deploy
      name: "Déploiement"
```

Si `workflow` est `null` ou absent, le workflow par défaut de `config.yml` est utilisé.

---

## Bonnes pratiques

1. **IDs lisibles** : Utiliser des numéros séquentiels (1, 2, 3) ou avec lettres pour sous-phases (9a, 9b)

2. **Priorités** :
   - 1-3 : Critique
   - 4-6 : Important
   - 7-9 : Normal
   - 10+ : Faible

3. **Notes** : Documenter les décisions importantes, pas les détails d'implémentation

4. **Optionnel** : Marquer les tâches "nice to have" pour les distinguer du MVP
