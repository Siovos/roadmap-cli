# Exemples

Cas d'usage concrets basés sur le projet siovos-archi.

---

## Démarrer un nouveau projet

### Initialisation complète

```bash
cd siovos-archi
roadmap init

# Configurer le projet
# Éditer .phases/config.yml avec le bon nom
```

### Créer la structure initiale

```bash
# Phases principales
roadmap add 1 "MVP Technique"
roadmap add 2 "UI/UX"
roadmap add 3 "Backend & Auth"
roadmap add 4 "Release Alpha"

# Définir les priorités
roadmap priority 1 --set 1
roadmap priority 2 --set 2
roadmap priority 3 --set 3
roadmap priority 4 --set 4

# Démarrer la première phase
roadmap status 1 --set in_progress
```

---

## Organiser une phase complexe

### Exemple : Phase Observabilité

```bash
# Créer la phase
roadmap add 9 "Observabilité & Infra"
roadmap status 9 --set in_progress
roadmap priority 9 --set 1

# Ajouter les tâches principales
roadmap task add 9 "Logs structurés Pino"
roadmap task add 9 "Métriques Prometheus/Grafana"
roadmap task add 9 "Mode debug utilisateur"

# Tâche optionnelle
roadmap task add 9 "GlitchTip error tracking" --optional

# Sous-tâches pour décomposer
roadmap task add 9 "Installer pino" --parent 9.1
roadmap task add 9 "Migrer console.error" --parent 9.1
roadmap task add 9 "Configurer pino-pretty" --parent 9.1

# Documenter les décisions
roadmap note 9 "Choix Pino pour la performance (vs Winston)"
roadmap note 9 "Format JSON en prod, pretty en dev"
```

### Voir le résultat

```bash
roadmap show 9
```

```
🔄 Phase 9 - Observabilité & Infra

  Priorité:   1
  Statut:     in_progress
  Créée le:   2026-02-23

  Tâches:
    ⬜ 9.1 - Logs structurés Pino
      ⬜ 9.1.1 - Installer pino
      ⬜ 9.1.2 - Migrer console.error
      ⬜ 9.1.3 - Configurer pino-pretty
    ⬜ 9.2 - Métriques Prometheus/Grafana
    ⬜ 9.3 - Mode debug utilisateur
    ⬜ 9.4 - GlitchTip error tracking (optionnel)

  Notes:
    2026-02-23 - Choix Pino pour la performance (vs Winston)
    2026-02-23 - Format JSON en prod, pretty en dev
```

---

## Workflow de développement quotidien

### Matin : Voir où on en est

```bash
roadmap report
```

### Démarrer une tâche

```bash
roadmap task start 9.1
roadmap workflow 9.1 --set implementation
```

### Terminer et passer à la suite

```bash
roadmap task done 9.1
roadmap task start 9.2
```

### Fin de journée : Ajouter une note

```bash
roadmap note 9 "Pino intégré, reste 50 fichiers à migrer"
```

---

## Utilisation avec une AI

### Obtenir l'état pour discussion

```bash
roadmap report --json | jq '.'
```

### Lister les prochaines tâches

```bash
roadmap report --json | jq '.next_tasks[] | "\(.id) - \(.name)"'
```

Output :
```
"9.2 - Métriques Prometheus/Grafana"
"9.3 - Mode debug utilisateur"
```

### Trouver les tâches optionnelles

```bash
roadmap report --json | jq '.optional_tasks'
```

### Script de vérification CI

```bash
#!/bin/bash
# check-roadmap.sh

BLOCKED=$(roadmap report --json | jq '.blocked.tasks | length')

if [ "$BLOCKED" -gt 0 ]; then
  echo "⚠️  Il y a des tâches bloquées !"
  roadmap report --json | jq '.blocked.tasks[] | "\(.id) - \(.name)"'
  exit 1
fi

echo "✅ Aucune tâche bloquée"
```

---

## Gérer plusieurs projets

### Structure recommandée

```
siovos/
├── siovos-archi/
│   └── .phases/          # Roadmap siovos-archi
├── siovos-landing/
│   └── .phases/          # Roadmap landing
└── roadmap-cli/
    └── .phases/          # Roadmap de l'outil lui-même
```

### Voir l'état de chaque projet

```bash
cd siovos-archi && roadmap report
cd ../siovos-landing && roadmap report
cd ../roadmap-cli && roadmap report
```

---

## Réorganiser une roadmap

### Déplacer une tâche vers une autre phase

```bash
# La tâche 9.4 (GlitchTip) devrait être dans une phase dédiée
roadmap add 13 "Error Tracking"
roadmap task move 9.4 --to 13
```

### Changer les priorités

```bash
# Phase 11 devient urgente
roadmap priority 11 --set 1

# Phase 9 passe en priorité normale
roadmap priority 9 --set 5
```

### Voir le nouvel ordre

```bash
roadmap list --table
```

---

## Export et partage

### Générer la documentation

```bash
roadmap export
```

Crée `ROADMAP.md` :

```markdown
# siovos-archi - Roadmap

## Phases

| Phase | Nom | Statut | Priorité | Progression |
|-------|-----|--------|----------|-------------|
| 9 | Observabilité & Infra | 🔄 | 1 | 1/4 |
| 10 | Modèle Sémantique | ✅ | 10 | - |
...
```

### Commiter avec git

```bash
git add .phases/ ROADMAP.md
git commit -m "docs: mise à jour roadmap phase 9"
```

---

## Patterns utiles

### Phase avec workflow personnalisé

Pour une phase qui ne suit pas le workflow standard :

```yaml
# .phases/phase-9.yml
workflow:
  enabled: true
  stages:
    - id: poc
      name: "Proof of Concept"
    - id: implementation
      name: "Implémentation"
    - id: rollout
      name: "Déploiement progressif"
    - id: monitoring
      name: "Monitoring"
```

### Tâches dépendantes

Utiliser les sous-tâches pour les dépendances :

```bash
roadmap task add 9 "Setup Grafana"
roadmap task add 9 "Créer dashboards" --parent 9.2
roadmap task add 9 "Configurer alertes" --parent 9.2
```

La tâche parent ne peut être "done" que quand les enfants sont terminés (convention).

### Phase "Backlog"

Pour les idées futures :

```bash
roadmap add 99 "Backlog"
roadmap priority 99 --set 100

roadmap task add 99 "Support multi-tenant" --optional
roadmap task add 99 "API GraphQL" --optional
roadmap task add 99 "Mobile app" --optional
```
