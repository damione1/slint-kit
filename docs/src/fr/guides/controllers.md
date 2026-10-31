---
layout: base.njk
locale: fr
title: "Contrôleurs typés"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Contrôleurs typés


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

## Dates de calendrier, formats et disponibilités

`CivilDate` utilise le calendrier grégorien proleptique, pour les années 1 à 9999. `KitDateField` et `KitDatePicker` conservent leur contrat HTML date : brouillons, `min`, `max` et chaînes acceptées suivent `YYYY-MM-DD`. Le libellé d’affichage et le format de saisie sont deux choix distincts. Aucun ordre jour/mois ni fuseau horaire n’est deviné.

Pour les champs texte et libellés gérés par l’application, validez un `DatePattern` une fois, puis conservez-le avec le contrôleur :

```rust
use slint_kit::{CivilDate, DateNames, DatePattern, DraftValue};
let pattern = DatePattern::new("%d/%m/%Y").unwrap();
let DraftValue::Parsed(date) = pattern.parse("29/02/2024") else { return };
assert_eq!(date, CivilDate::new(2024, 2, 29).unwrap());
assert_eq!(date.iso_string(), "2024-02-29");
let spoken = DatePattern::new("%A %-d %B %Y").unwrap();
assert_eq!(spoken.format_with_names(date, &DateNames::french()), "jeudi 29 février 2024");
```

Les directives acceptées sont `%Y`, `%m`, `%d`, `%e`, `%b`, `%B`, `%a`, `%A`, `%j`, `%G`, `%V`, `%u`, `%F` et `%%` ; les directives numériques acceptent le remplissage `-`, `_` ou `0`. Les champs numériques sans remplissage ou remplis par des espaces exigent des séparateurs non numériques : `%Y%m%d` est valide, tandis que `%Y%-m%-d` est refusé car ambigu. Elles couvrent les ordres et séparateurs numériques, dates compactes, noms de mois anglais, dates ordinales et semaines ISO. `%y` est refusé : une année à deux chiffres exige une règle de siècle explicite. `parse`/`format` emploient les noms anglais. `parse_with_names`/`format_with_names` utilisent des `DateNames` validés fournis par l’hôte ; un jeu français est inclus. Les noms doivent être non vides et sans espaces en début ou fin. Leur comparaison utilise les minuscules Unicode, conserve les accents et exige la ponctuation des abréviations configurées. Les autres systèmes calendaires et formes de chiffres nécessitent un adaptateur explicite de l’hôte.

Gardez l’éditeur natif de `KitTextField` pour une saisie personnalisée. Analysez le brouillon avec le format choisi, puis validez le `CivilDate` avec les mêmes règles que les clics sur le calendrier. N’associez pas `HtmlInputKind.date` à un texte localisé : le moteur HTML commun attend ISO. Pour un formulaire, appliquez aussi les règles de présence et de validité personnalisée, puis protégez le chemin d’envoi Rust.

`DateConstraints` combine des bornes inclusives `NumericBounds<CivilDate>` facultatives, des périodes `DateRange` indisponibles et des jours de semaine indisponibles. Transmettez-le à `CalendarController::with_constraints`. `choose` refuse les dates bloquées ; le focus clavier peut les parcourir. Les sept valeurs `WeekStart` sont disponibles ; `set_week_start` demande aussi de réordonner les noms de jours fournis à la vue. `can_navigate` pilote `previous-enabled` et `next-enabled` ; le défilement par mois/année doit changer le mois visible, celui par jour/semaine doit déplacer la date focalisée. Le constructeur à bornes seules `CalendarController::new` rejette une sélection initiale hors bornes vers `None` et borne le focus ; utilisez `with_constraints` pour obtenir une erreur explicite de construction. `show_month` et `focus_date` déplacent la cible clavier visible sans accepter une valeur. `grid_for_month` permet de composer plusieurs `KitCalendar` avec une sélection et des règles communes.

Si l’hôte fournit une erreur métier via `error`, effacez cette ancienne erreur à `edited` avant la nouvelle soumission. La date acceptée reste inchangée tant que le nouveau brouillon ne respecte pas les mêmes contraintes.

`DateRangeRules` ajoute un nombre minimal et maximal de nuits facultatif. Une période d’un seul jour vaut zéro nuit ; la période inclusive ne peut traverser une date indisponible. `DateRangeController::with_rules` préserve la période acceptée jusqu’à Apply et la restaure avec Cancel. Un choix de fin refusé conserve le début. `DateSetController` gère plusieurs dates identifiées par `CivilDate` : projetez `selected().contains(&date)` dans `CalendarDay.selected`, puis appelez `toggle` depuis l’action du calendrier. L’index visuel n’identifie aucune date.

Le clavier utilise les flèches, Début/Fin et Page précédente/suivante ; Maj fait défiler les années. Un calendrier en lecture seule reste navigable et ne peut accepter une date. Fournissez titres de mois, noms de jours, libellés accessibles et actions de période traduits. Les exemples exécutables [calendrier](/fr/components/calendar/), [sélecteur de date](/fr/components/date-picker/) et [sélecteur de période](/fr/components/date-range-picker/) composent cartes, badges et aides du kit avec de vraies règles de disponibilité. Leur date de référence est fixe pour rendre les exemples reproductibles. Ces API ne qualifient pas, à elles seules, un lecteur d’écran ou l’IME sur un backend donné.

Les composants de navigation (`KitMenu`, `KitMenuBar`, `KitTabs`, `KitAccordion`, `KitCarousel`, `KitBreadcrumbs` et `KitPagination`) exposent `interactive`, qui vaut `true` par défaut. Liez cette propriété à `!modal-open` pour chaque composant du fond. Désactivez les actions ordinaires avec `disabled` et les champs avec `access: FieldAccess.disabled` dans le même périmètre. Un voile intercepte seulement le pointeur : il ne rend pas les actions accessibles inéligibles. Ces propriétés préservent les modèles, la sélection et le contenu monté. Désactivez séparément les enfants personnalisés, puis restaurez le focus sur l’ouvreur exact à la fermeture de la surface modale.
