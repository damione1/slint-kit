---
layout: base.njk
locale: fr
title: "Contrôleurs typés"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Valeurs typées et collections applicatives

Commencez par [Configuration du projet](/fr/guides/project-setup/) et un appel à `slint_kit::install(&app)` par fenêtre. Les composants interactifs possèdent leur état local et appellent des réducteurs purs partagés. Les callbacks métier reçoivent une projection vérifiée convertible en type Rust. L’application reste propriétaire de ses documents, listes, tableaux, fichiers et tâches asynchrones.

## FieldSpec et brouillons natifs

`FieldSpec<T, E>` contient analyseur, format de la valeur acceptée, format d’affichage, validateur métier et format des erreurs. `FieldSpec::codec` convient lorsque l’analyse exprime toute la règle ; `FieldSpec::new` ajoute un type d’erreur et un validateur métier. Les fabriques couvrent texte, courriel, URL, entier, décimal, argent, pourcentage, quantité, valeurs temporelles, téléphone, OTP et masques.

Enregistrez les codecs nommés avec `install_field_specs(&app, specs)` avant l’affichage ; les doublons de `FieldSpecKey` sont rejetés atomiquement. Un codec est une définition partagée immuable. Il ne stocke jamais brouillon ou valeur acceptée d’un composant. `KitFieldService.reduce` reçoit explicitement `FieldState`, action, règles et clé du codec ; les champs conservent eux-mêmes l’état retourné.

Slint conserve les brouillons natifs et les projections formatées des valeurs acceptées. Le service pur valide et formate le candidat avec le codec. À la frontière métier Rust, `spec.accept(FieldCommit)` décode cette projection et retourne `T`, en appliquant les mêmes contraintes intrinsèques et règles métier. Conservez la valeur retournée. `with_display` et `with_grouping` modifient la présentation au repos sans changer la valeur acceptée ni réécrire l’édition native. Une erreur ou un brouillon incomplet conserve la dernière valeur acceptée.

Argent et décimaux restent exacts : utilisez `Money`, `DecimalValue` et leurs règles explicites. Les curseurs utilisent des chaînes décimales et une grille exacte partagée ; les flottants normalisés servent uniquement au pointeur et au rendu. `SliderSpec` définit minimum, maximum et pas. `DecimalRangeView` se convertit en `OrderedRange<DecimalValue>`, dont les bornes ne peuvent se croiser.

`FieldBinding` et `FieldController` restent disponibles pour les intégrations personnalisées. Le chemin ordinaire n’exige aucune macro propre au composant. `init_form_validation!` est dépréciée au profit de `install`.

La macro historique `bind_select!` reste disponible pour la migration et est dépréciée dans la documentation. Remplacez sa convention de callbacks de fenêtre par `Choices<K>`, le contrat `options`/`selected`/`chosen` du composant et l’installateur partagé. Les projections explicites `SelectBinding` restent disponibles pour les intégrations contrôlées personnalisées.

## Choix stables

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Destination { Overview, Activity }
let choices = slint_kit::Choices::new([
    (Destination::Overview, "Vue d’ensemble"),
    (Destination::Activity, "Activité"),
]).unwrap();
app.set_options(choices.model());
app.on_chosen(move |row| {
    if let Some(destination) = choices.accept(row) {
        println!("Ouvrir {destination:?}");
    }
});
```

Les jetons projettent des clés stables, jamais des positions de tableau. `Choices::replace` préserve l’identité lors d’un réordonnancement et invalide les générations de source obsolètes. Les choix désactivés ne peuvent être acceptés. Chaque composant conserve sélection, élément actif, ouverture et recherche indépendants. `ownership: Ownership.controlled` émet une intention et attend la confirmation via la propriété sélectionnée. MultiSelect et ToggleGroup possèdent des ensembles et émettent `selection-changed(row, added)` ; `Choices::accept` retourne toujours la clé métier.

`KitSelectKeyboard`, `KitSelectTrigger` et `KitSelectSurface` sont des primitives de composition. Leurs propriétés permettent de partager une instance de sélection entre les parties ; elles ne sont pas des contrôleurs applicatifs distincts. Select et Combobox fournissent déjà cette composition. Les composants calendrier/date/heure conservent également leur état local ; obtenez `CivilDate`, `DateRange`, `LocalDateTime` ou `ResolvedDateTime` par conversion vérifiée de leur charge `chosen`.

`KitSelectKeyboard.move-active(NavigationStep)` et `type-ahead(string)` font passer les surfaces personnalisées par la même session que le clavier natif. Utilisez `choose-option(token, generation)` pour activer une option ; la surface appelante conserve ses barrières d’interaction.

## KeyedModel pour les données applicatives

`KeyedModel<K, R>` associe une clé typée unique à chaque ligne publique. `R: KeyedRow` définit la projection ; les lignes intégrées couvrent tableaux, contenus, textes, puces, fichiers, notifications et toasts. `model()` fournit un modèle virtualisable. `replace` rejette les doublons avant toute mutation et conserve les jetons des clés restantes. Un jeton retiré n’est jamais recyclé.

Pour les vues à ports explicites, `bind` reçoit la fenêtre et les setters des lignes, de la génération et de l’index actif. La liaison conserve une fenêtre faible. `navigation()` implémente le parcours partagé ; `activation(handler)` actualise l’état `SelectableRow` et appelle le métier avec `K`. Validez les autres intentions avec `accept(token, generation)` ou `remove_fenced`. Les callbacks de puces sans génération utilisent `remove(token)` avec un jeton jamais réutilisé. Une modification de source invalide les gestes portant une génération ; ne substituez pas un index à une clé.

L’application choisit toujours données, tri, filtrage, chargement et commandes. Un grand modèle appartient à un contrôle virtualisé, pas à une répétition d’enfants visuels. `action`, `event`, `event2` et `event3` connectent les callbacks métier par une fenêtre faible et transmettent temporairement une instance vivante au handler.

## Effets hôtes et notifications

`FilePickerBinding` et `DirectoryPickerBinding` reçoivent des ports de projection explicites, installent des handlers à fenêtre faible et livrent des `FileHandle` ou `DirectoryHandle` typés. Les générations éliminent les résultats asynchrones obsolètes. La fonctionnalité optionnelle `native-winit` ajoute un pont de dépôt pour la fenêtre avec une capacité explicite. Consultez [Intégration native](/fr/guides/native/) pour les limites des plateformes.

`ToastBinding<Action>` stocke les actions typées dans `ToastQueue`, projette les lignes `ToastEntry` et gère un minuteur faible. Capacité et `ToastLifetime` sont explicites. Ses trois handlers routent action, fermeture et pauses indépendantes pointeur/focus ; l’application exécute uniquement l’action métier reçue.

`KeyedModel::sorting_by` transforme un comparateur de clés métier en handler de tri alterné. Le tri conserve sélection et identité active et avance la génération : un geste commencé avant le réordonnancement ne peut pas cibler la nouvelle projection.

`Choices::selections(&keys)` prépare une sélection multiple initiale ; une clé absente ou répétée refuse toute la projection. `KeyedModel::bind_active` publie la ligne active vers un éditeur au moyen d’une fenêtre faible. `update_active` refuse une confirmation pour une ligne qui n’est plus active.

Accordez `DecimalRules.locale` du codec décimal au `number-format` du composant, et `ClockFormat` du codec temporel à son `clock-format`. Le moteur HTML valide la grammaire native ; le codec reçoit ensuite cette grammaire, y compris les virgules décimales et les heures sur 12 heures. Le format accepté reste lisible par le même analyseur : Annuler restaure donc une valeur éditable dans le format déclaré.
