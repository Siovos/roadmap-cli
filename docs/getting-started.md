# Guide de démarrage

Ce guide vous accompagne de l'installation à votre première roadmap.

---

## Installation

### Option 1 : Homebrew (recommandé)

```bash
brew tap siovos/tap
brew install roadmap-cli
```

### Option 2 : Script d'installation

```bash
curl -fsSL https://raw.githubusercontent.com/siovos/roadmap-cli/main/install.sh | bash
```

### Option 3 : Cargo

```bash
cargo install --git https://github.com/siovos/roadmap-cli
```

### Vérification

```bash
roadmap --version
# roadmap 0.1.0
```

---

## Initialiser une roadmap

Dans le répertoire de votre projet :

```bash
cd mon-projet
roadmap init
```

Résultat :
```
✓ Roadmap initialisée !
  Créé: .phases/config.yml

  Prochaine étape: roadmap add <id> <nom> pour créer une phase
```

Cela crée le dossier `.phases/` avec la configuration par défaut.

---

## Créer des phases

### Phase simple

```bash
roadmap add 1 "MVP"
```

### Phase avec priorité

```bash
roadmap add 2 "Beta"
roadmap priority 2 --set 5
```

### Sous-phase

```bash
roadmap add 1a "MVP - Auth" --parent 1
```

### Changer le statut

```bash
roadmap status 1 --set in_progress
```

---

## Ajouter des tâches

### Tâche simple

```bash
roadmap task add 1 "Setup projet"
# ✓ Tâche 1.1 ajoutée à la phase 1
```

### Tâche optionnelle

```bash
roadmap task add 1 "Documentation API" --optional
```

### Sous-tâche

```bash
roadmap task add 1 "Configurer ESLint" --parent 1.1
# ✓ Tâche 1.1.1 ajoutée à la phase 1
```

---

## Suivre la progression

### Démarrer une tâche

```bash
roadmap task start 1.1
# 🔄 Tâche 1.1 marquée comme in_progress
```

### Terminer une tâche

```bash
roadmap task done 1.1
# ✅ Tâche 1.1 marquée comme done
```

---

## Voir l'état du projet

### Rapport de progression

```bash
roadmap report
```

```
═══════════════════════════════════════════════════════════
                    📊 RAPPORT DE PROGRESSION
═══════════════════════════════════════════════════════════

📈 RÉSUMÉ
   Phases: 0 terminées / 2 total
   Tâches: 1 / 3 (33%)
   [██████████░░░░░░░░░░░░░░░░░░░░]

🔄 PHASES EN COURS
   [P1] 1 — MVP (1/3)

📋 PROCHAINES TÂCHES
   ⬜ 1.2 Documentation API — MVP

═══════════════════════════════════════════════════════════
```

### Vue arborescente

```bash
roadmap tree
```

```
📋 Roadmap

├── 🔄 Phase 1 — MVP (P1)
│   ├── ✅ 1.1 — Setup projet
│   │   └── ✅ 1.1.1 — Configurer ESLint
│   └── ⬜ 1.2 — Documentation API (opt)
└── ⬜ Phase 2 — Beta (P5)
```

### Interface interactive

```bash
roadmap ui
```

Navigation : `↑↓` ou `jk`, `→` pour les tâches, `q` pour quitter.

---

## Exporter en Markdown

```bash
roadmap export
```

Génère `ROADMAP.md` (ou le chemin configuré dans `config.yml`).

---

## Ajouter des notes

Documentez vos décisions :

```bash
roadmap note 1 "Choix Next.js pour le SSR"
roadmap note 1 "Architecture hexagonale validée"
```

Les notes apparaissent dans :
- `roadmap show 1`
- Le fichier exporté

---

## Récapitulatif des commandes

| Action | Commande |
|--------|----------|
| Initialiser | `roadmap init` |
| Créer phase | `roadmap add <id> <nom>` |
| Créer tâche | `roadmap task add <phase> <nom>` |
| Terminer tâche | `roadmap task done <id>` |
| Voir état | `roadmap report` |
| Vue arbre | `roadmap tree` |
| Exporter | `roadmap export` |

---

## Prochaines étapes

- [Référence des commandes](commands.md) — Toutes les options
- [Format des données](data-format.md) — Structure YAML
- [Exemples](examples.md) — Cas d'usage avancés
