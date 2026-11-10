---
layout: base.njk
locale: fr
title: "Contrôleurs typés"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Valeurs typées et collections applicatives

`FieldSpec<T, E>` retourne des valeurs métier validées ; `Choices<K>` conserve les clés des choix ; `KeyedModel<K, Row>` relie les lignes à leurs identités stables. Ces API partagent les comportements de validation et de sélection des composants Slint Kit. Votre application fournit ses types, ses règles et ses données.

Commencez par [Configuration du projet](/fr/guides/project-setup/) et un appel à `slint_kit::install(&app)` par fenêtre. Les composants interactifs possèdent leur état local et appellent des réducteurs purs partagés. Les callbacks métier reçoivent une projection vérifiée convertible en type Rust. L’application reste propriétaire de ses documents, listes, tableaux, fichiers et tâches asynchrones.

Les contrôles montés ensuite par `if`, un modèle ou un popup initialisent leur propre garde de sélection avec les services déjà installés. Une installation par fenêtre suffit ; inutile de réinstaller ou de remplacer artificiellement les options.

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

`KeyedModel::sorting_by` transforme un comparateur de clés métier en handler de tri alterné. Le tri conserve sélection et identité active et avance la génération : un geste commencé avant le réordonnancement ne peut pas cibler la nouvelle projection. L’[application de tableau triable](/fr/guides/table-example/) raccorde `bind`, `navigation()`, `activation` et `sorting_by` à un `KitDataTable` dans un seul fichier court.

`KeyedModel::cell_commits(handler)` est le handler de `KitDataTable.cell-committed` pour l’édition de cellule sur place : il n’admet un commit que sous la génération courante et pour une ligne active, puis transmet la clé de ligne, le jeton de colonne et le `FieldCommit`, que l’application passe dans le `FieldSpec::accept` de la colonne avant de republier la ligne.

`Choices::selections(&keys)` prépare une sélection multiple initiale ; une clé absente ou répétée refuse toute la projection. `KeyedModel::bind_active` publie la ligne active vers un éditeur au moyen d’une fenêtre faible. `update_active` refuse une confirmation pour une ligne qui n’est plus active.

Accordez `DecimalRules.locale` du codec décimal au `number-format` du composant, et `ClockFormat` du codec temporel à son `clock-format`. Le moteur HTML valide la grammaire native ; le codec reçoit ensuite cette grammaire, y compris les virgules décimales et les heures sur 12 heures. Le format accepté reste lisible par le même analyseur : Annuler restaure donc une valeur éditable dans le format déclaré.

`bind_active` publie uniquement lorsque la clé active ou la révision de son enregistrement change. Modifier une autre ligne ou trier conserve le brouillon. Un `replace` explicite fournit de nouveaux enregistrements et republie la ligne active. `get`, `len`, `is_empty`, `update_by_key` et `remove_by_key` donnent accès aux enregistrements métier, même désactivés. Une mise à jour conserve la génération et notifie une seule ligne ; le remplacement ou le changement des membres invalide les anciens gestes.

Un clic sans mouvement ne redimensionne pas une colonne. Échap ou un appui secondaire annule l’aperçu. Un double-clic sur la séparation émet `autosize-requested(column-token)` ; calculez la largeur adaptée au contenu puis actualisez la politique de colonne.

Les cellules utilisent l’éditeur natif. Un clic choisit ligne et curseur de cellule, un double-clic ou F2 ouvre un champ modifiable, Entrée confirme, Échap annule et Tab valide avant déplacement. `edit-active()`, `commit-edit()` et `cancel-edit()` exposent ces opérations à l’hôte. Installez le kit une fois par fenêtre, même pour le texte libre. `active-index` et `active-column` sont des curseurs de vue ; l’identité de confirmation reste `(row-token, column-token, generation, FieldCommit)`. Lisez `edit-error` pour un rejet du codec ou d’une contrainte HTML. La cellule accessible expose sa valeur et l’indication localisée `editable-label` / `read-only-label` ; la preuve headless ne qualifie pas un lecteur d’écran.

Traduisez l’édition de document avec `editor-label`, `editor-placeholder`, `save-label` et `copy-label` ; leurs valeurs par défaut restent anglaises.


### Raccourcis clavier depuis Rust

Les hooks clavier publics renvoient `KitKeyResult`, enum de présentation commune exportée par `@kit` et `slint_kit::ui::KitKeyResult`. Depuis Rust, retournez `KitKeyResult::Accept` pour consommer un raccourci ou `Reject` pour préserver le comportement intégré. Les liaisons Slint utilisent `KitKeyResult.accept` et `.reject` ; le kit les convertit à la frontière native `FocusScope` ou `TextInput`. N’utilisez ni le `EventResult` sans rapport du backend winit, ni les API privées de Slint.

Ce résultat concerne `KitTextField.key-pressed`, `KitDataTable.key-event`, le hook hérité `KitPressable.key-event`, `tab-navigation` des sélecteurs et leurs fonctions publiques de navigation `handle-key`. Le hook d’un champ s’exécute seulement sans préédition IME active. Laissez passer le texte et la navigation ordinaires pour conserver sélection, annulation et IME natifs.

Utilisez `CellContent::constrained_field(text, codec, access, constraints)` pour les règles intrinsèques d’une cellule. `KitInputConstraints` est la projection du moteur HTML partagé ; `kind: HtmlInputKind::Number` active le filtrage numérique natif et les longueurs absentes `min_length` / `max_length` valent -1. La valeur initiale fournit la base du pas et `custom_error` reste une contrainte persistante ; `FieldAccess` de la cellule contrôle l’accès. Présence, motif, bornes et pas sont revérifiés par `FieldSpec::accept(commit)` en Rust. Pour un verrou métier ou une autre règle synchrone, implémentez `validate-edit(row_token, column_token, generation, commit) -> string` : une chaîne vide accepte, un message localisé conserve le brouillon ouvert. Entrée, Tab et `commit-edit()` réévaluent le même brouillon après un refus temporaire ; la ligne canonique reste inchangée jusqu’à acceptation. Évitez de rétablir silencieusement une valeur rejetée après confirmation.

Utilisez `header-focus: TableHeaderFocus.roving` pour un arrêt Tab unique : Haut depuis la première ligne (ou le corps vide) entre dans les en-têtes, Gauche/Droite change de colonne, Bas revient au corps, Entrée/Espace trie et Alt+Gauche/Droite demande un redimensionnement borné. La politique `individual` par défaut conserve les en-têtes triables et leurs poignées dans Tab avant le corps. Localisez `header-label` ; `sort-active-column()` et `resize-active-column(delta)` partagent les callbacks protégés à tokens stables. Le redimensionnement utilise la largeur mesurée de l’en-tête courant : affichez/mesurez d’abord la vue et laissez le prochain tour de mise à jour Slint émettre son callback.

Placer une ligne éditée entièrement hors du viewport du tableau annule son brouillon sans confirmation, y compris par défilement à la molette ou réduction du viewport. Le retour de la ligne conserve sa valeur acceptée et ne restaure pas un éditeur ; ouvrez explicitement une nouvelle session.

Observez `edit-started(row, column, generation)` et `edit-ended(row, column, generation, outcome, message)` pour suivre une session par son identité armée. `TableEditOutcome` distingue committed, unchanged, cancelled, rejected-on-blur, superseded et viewport-exit. Lisez `edit-draft` pour récupérer le brouillon et `edit-bounds` (`valid`, x/y/width/height logiques de fenêtre) après layout. `edit-blur-policy: TableEditBlurPolicy.keep-open` garde le focus dans l’éditeur natif invalide ; l’annulation explicite reste disponible. Le mode commit-or-drop par défaut conserve le comportement natif à la perte du focus et signale le refus. `unchanged-edit-policy: TableUnchangedEditPolicy.skip` supprime une proposition inchangée tout en terminant la session. Les événements de fin figent leur identité, ce qui permet d’ouvrir une autre cellule dans le gestionnaire ; les mises à jour canoniques passent toujours par cell-committed typé et ses gardes.

Lisez `page-rows` de la collection lors de PageUp/PageDown : il reflète le viewport du corps après l’en-tête du tableau ou le pied d’édition du document, la densité et l’échelle du texte. Exposez-le sur votre fenêtre puis branchez `model.navigation(&window, Window::get_page_rows)` ; ce gestionnaire faible relit le getter à chaque touche. Un contrôleur direct transmet cette valeur à `navigate_page`. `navigate(step, page_rows)` exige aussi une capacité explicite ; `navigate_page` reste un alias explicite de compatibilité.

Les viewports de collection exposent `first-visible-index`, `visible-row-count` et les décalages signés `scroll-x`/`scroll-y` indépendamment de la sélection. Utilisez `scroll-to-index` pour une position visuelle, `reveal-active` pour l’identité active courante, `scroll-to-top` pour les deux axes, et `set-scroll(x,y)` pour restaurer une position dans les bornes actuelles. La sauvegarde appartient à l’identité et à l’ordre du modèle où elle a été prise. `near-end-rows`, `end-near` et le signal de transition `near-end()` permettent le chargement progressif ; si une page ajoutée laisse `end-near` vrai, poursuivez le remplissage explicitement sans attendre une autre transition. Le défilement programmatique termine aussi l’édition d’un tableau si sa ligne quitte le viewport.

Quand le nombre de colonnes change, `active-column` est borné aux positions restantes pour le corps comme pour l’en-tête. Supprimer toutes les colonnes ramène le corps au curseur de ligne entière (-1) ; un ajout ultérieur ne ressuscite pas une cellule obsolète. Supprimer la cellule éditée termine toujours la session comme `superseded`, sans engager le brouillon.
## Étapes interactives et chronologies

Utilisez `KeyedModel<K, StepEntry>` et résolvez `step-activated(token, generation)` avec `accept`. Définissez `enabled` pour les étapes accessibles ; le kit émet une demande et votre flux décide du changement d’état. La génération refuse les gestes maintenus après remplacement. `interactive` vaut false par défaut ; orientation et descriptions fonctionnent dans les deux modes. Les événements de chronologie sont des projections ordonnées : fournissez un `tone` sémantique et un `glyph` décoratif facultatif, en conservant le sens dans le titre et la description.

Pour les bandes de commandes attachées, fournissez `KitButtonGroup.actions` avec `KeyedModel<K, ButtonGroupEntry>` et résolvez `activated(token, generation)` avec `accept`. Chaque bouton garde son arrêt Tab et son action ; ce n’est pas un groupe de sélection. L’hôte possède les états busy/disabled et les commandes. Remplacer le modèle annule les gestes maintenus. Utilisez @children pour une composition personnalisée, avec thème et désactivation propres à chaque enfant.

Les contraintes du calendrier peuvent changer pendant la modification d’un enregistrement. La date devenue indisponible reste acceptée jusqu’au choix de son remplacement ; la grille se projette sans sélection autorisée et propose encore les dates valides. `DateRangeController::reduce` récupère une ancienne période complète ou un début de brouillon invalides en traitant la prochaine date valide comme un nouveau début. Les nouveaux choix invalides restent refusés et Appliquer valide toute la nouvelle période. Les constructeurs qui valident explicitement une période acceptée restent stricts. Fournissez des bornes et disponibilités valides ; une configuration mal formée est une erreur distincte.

La configuration des dates expose `CalendarConfigurationState.unchecked` avant `install`, puis `.valid` ou `.invalid`. Des bornes min/max mal formées ou inversées, des dates civiles invalides ou des périodes indisponibles mal formées produisent une grille de référence désactivée et un `configuration-message` visible ; le parcours et l’acceptation ne contournent pas ces règles. Traduisez le message une fois avec `KitStrings.calendar-configuration-error`, ou remplacez une instance. Corriger la configuration rétablit le composant existant. Les durées sont validées séparément par le réducteur de période et Appliquer. Les littéraux Rust complets `CalendarProjection` doivent désormais inclure `configuration_state` ; `calendar_projection()` fournit Valid pour un contrôleur validé.


Les modèles `weekdays` du calendrier contiennent toujours sept noms dans l’ordre lundi à dimanche. Définissez `week-start: WeekStart.sunday` (ou un autre jour) pour faire pivoter les en-têtes et les colonnes. `KitDatePicker` et `KitDateRangePicker` transmettent cette convention au calendrier. Ne faites pas pivoter vous-même les noms localisés quand le premier jour change.

Après `slint_kit::install(&window)`, les colonnes du tableau peuvent être projetées indépendamment des données des lignes. `TableColumn.source` utilise par défaut `ColumnSourceMode.position` : son index dans le modèle de colonnes **d’entrée** choisit la cellule, avant filtrage des colonnes masquées. Utilisez `ColumnDataIndex::try_from(index_canonique)?.into()` pour conserver la source pendant un réordonnancement ou une omission de colonnes d’entrée. `visibility: ColumnVisibility.hidden` masque une colonne sans réécrire `TableRow.cells` ni `.content` ; `visible` est la valeur par défaut. `active-column` est une position visible, tandis que les callbacks de tri, largeur et édition conservent le jeton stable de colonne. Une source absente ou négative peint une cellule vide et n’ouvre aucun éditeur ; les valeurs supplémentaires d’une ligne sont ignorées. Masquer/retirer la colonne éditée ou changer sa source termine la session comme remplacée, sans valider le brouillon. Gardez les jetons de colonne uniques et leur source stable pendant un réordonnancement.
