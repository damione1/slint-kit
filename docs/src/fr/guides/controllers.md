---
layout: base.njk
locale: fr
title: "Contrôleurs typés"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Contrôleurs typés

Cette référence documente Slint Kit 0.2.0. Pour les libellés traduisibles, le focus composé et le parcours natif des modales, consultez les [nouveautés et la compatibilité](/fr/guides/source-updates/). Passez les deux dépendances du kit à `"0.2"` pour migrer depuis 0.1.

Conservez la valeur acceptée en Rust et projetez sa présentation dans Slint. La façade `slint_kit` réexporte les valeurs du cœur et les adaptateurs de fenêtre faible. Un jeton de vue n’est valide que pour sa génération de modèle ; l’identité métier reste une clé stable.

[Exécuter et lire l’application typée complète](/fr/guides/typed-example/) : champ natif, validation, sélection stable et tous les signaux reliés.

## Brouillons et valeurs acceptées

`FieldBinding<T, E>` relie un `FieldController<T, E>` à une fonction de publication. Choisissez un type métier pour `T` et une enum d’erreurs pour `E`. Son constructeur reçoit la valeur initiale acceptée, le mode de propriété, la politique de validation, la politique d’absence, le parseur, le formateur, le formateur d’erreurs et la publication de projection.

| Choix | Valeurs et comportement |
| --- | --- |
| `Ownership` | `Local` accepte les validations locales réussies. `Controlled` émet une intention et attend l’appel de l’hôte à `acknowledge`. |
| `CommitPolicy` | `OnBlur`, `OnSubmit`, `OnChangeWhenValid`. L’hôte appelle `commit` avec le `CommitTrigger` correspondant. |
| `AbsencePolicy` | `Required` ou `Optional` ; un brouillon vide diffère d’une valeur invalide. |
| `DraftValue<T>` | `Empty`, `Incomplete`, `Parsed(T)` ou `Invalid(ParseIssue)`. Préservez la saisie incomplète. |
| `FieldAccess` | `Editable`, `ReadOnly`, `Disabled`. Le texte en lecture seule conserve sa sélection native. |

Transmettez les éditions à `edited(&str)`, reliez la perte de focus et l’envoi à `commit`, et utilisez `cancel` ou `reset` pour les actions explicites. Publiez `FieldProjection.draft`, l’accès, la validation et l’erreur vers la vue. La fonction de publication capture `window.as_weak()` et ne récupère une fenêtre forte que pendant sa mise à jour. Gardez la liaison vivante avec le contrôleur de fenêtre.

Préservez l’éditeur et évitez de réécrire son texte à chaque frappe. L’édition native Slint gère sélection, curseur, annulation et IME. Le formatage intervient lors de l’acceptation. Consultez les [liaisons complètes du showcase](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_fields.rs) et les [signaux de saisie](/fr/components/text-field/).

## Validation et envoi du formulaire

`begin_validation()` renvoie une `ValidationGeneration` ; conservez ce ticket avec la requête asynchrone. `finish_validation(ticket, errors)` ignore un résultat dépassé par un nouveau brouillon. N’identifiez pas une requête par un index de ligne mutable. `FormCoordinator` utilise aussi des tickets d’envoi et des clés de champs typées. Un bouton d’envoi émet l’action de formulaire ; l’hôte gère validation et persistance.

## Sélection par clé stable

`SelectBinding<K>` conserve les clés métier et publie dans `SelectProjection` les lignes, jetons actifs et sélectionnés, disponibilité, ouverture et génération. `activate(token, generation)` rejette une action périmée. `reconcile` remplace les options source ; `query` filtre la vue sans supprimer la clé acceptée. Les clés dupliquées constituent une erreur.

Utilisez `on_intent` pour `SelectionIntent::Commit(K)` et `SelectionIntent::Clear`. En propriété contrôlée, accusez réception de la clé acceptée avec `set_selected`. La macro `bind_select!` raccorde le contrat conventionnel de fenêtre ; ses propriétés `rows`, `select-state` et signaux `select-*` sont décrits dans le [code public de la macro](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/src/macros.rs).

`SelectionAvailability` distingue `Absent`, `Available` et `Unavailable`. Un choix masqué par un filtre ou retiré du modèle courant ne demande pas automatiquement d’effacer la valeur métier. Voir [KitSelect](/fr/components/select/) et les [exemples de navigation](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs).

## Nombres exacts et temps civil

Utilisez `DecimalValue`, `Money`, `CurrencyMetadata` et des règles explicites `NumberLocale`, `DecimalScale` et `RoundingPolicy`. Les positions de curseurs et projections des canaux de couleur servent au rendu ; elles ne remplacent pas les montants acceptés exacts. Analysez dates civiles et heures locales avec les types du cœur, puis résolvez le fuseau explicitement. Une heure ambiguë demande un choix explicite de l’instant.

## Surfaces modales et retour utilisateur

Gardez `FocusRegistry` et l’ouvreur exact dans le contrôleur hôte. Les primitives de dialogue et de tiroir présentent les demandes de focus ; enregistrez les vraies cibles et restaurez l’ouvreur à la fermeture. Une infobulle utilise une [KitTooltipLayer](/fr/components/tooltip-layer/) par fenêtre, montée après le contenu. `ToastQueue` gère les durées des toasts ; les pauses de pointeur et de focus utilisent des sources `ToastPause` distinctes pour préserver les interactions superposées.


## Tableau et identité des lignes

L’[application complète de tableau triable](/fr/guides/table-example/) montre une sélection qui survit au tri, la navigation clavier et le rejet des événements d’une ancienne génération.
