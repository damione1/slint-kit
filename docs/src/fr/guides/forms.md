---
layout: base.njk
locale: fr
title: "Formulaires, options partagées et validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---

# Formulaires, options partagées et validation

Les formulaires Slint Kit partagent un moteur de contraintes HTML et des codecs Rust typés. Ils refusent les courriels et URL mal formés, conservent les brouillons incomplets et retournent des valeurs validées à votre code métier. L’édition native reste dans Slint ; votre commande de soumission vérifie aussi la frontière Rust.

Configurez l’application une fois avec [Configuration du projet](/fr/guides/project-setup/). Importez les contrôles depuis `@kit` et appelez `slint_kit::install(&app)` avant d’afficher chaque fenêtre. Cette fonction installe les services purs partagés ; champs, calendriers et sélections conservent leur propre état. Une installation répétée est sans effet supplémentaire. Ces API ciblent Slint Kit **{{ project.version }}** ; utilisez des paquets runtime et build correspondants.

```slint
import { KitEmailField, FieldCommit } from "@kit";
export component App inherits Window {
    callback contact-changed(FieldCommit);
    KitEmailField {
        label: "Courriel professionnel";
        required: true;
        spec-key: "contact";
        committed(value) => { root.contact-changed(value); }
    }
}
```

Enregistrez le codec métier une fois et protégez aussi la frontière Rust :

```rust
let app = App::new()?;
slint_kit::install(&app);
let spec = slint_kit::FieldSpec::email();
slint_kit::install_field_specs(&app, [
    spec.named(slint_kit::FieldSpecKey::new("contact").unwrap())
]).unwrap();
app.on_contact_changed(move |commit| {
    if let Ok(Some(address)) = spec.accept(commit) {
        println!("Contact : {}", spec.format(&address));
    }
});
```

Le moteur de contraintes HTML applique les règles intrinsèques des courriels, URL, nombres, dates et autres champs : `required`, bornes, longueurs, motif et pas. Un texte non vide ne suffit pas pour un courriel ou une URL. Le codec personnalisé ajoute les règles métier après ces contraintes partagées. `FieldSpec::accept` retourne la valeur analysée ; transmettez-la au métier sans l’analyser de nouveau. Ne remplacez pas l’édition native par une réécriture du texte côté hôte.

Les champs numériques suivent le motif du bouton compteur des champs nombre du web : Haut/Bas déplacent d’un `step`, PageUp/PageDown de `page-steps` pas (10 par défaut), et Début/Fin sautent à `min`/`max` lorsque la borne est définie ; sinon ils gardent leur déplacement natif du curseur. Les pas utilisent l’arithmétique décimale exacte sur la grille `min`/`default-value` et remplacent le brouillon comme une frappe : `edited`, `value-edited`, le suivi dirty, l’annulation native et `commit-policy` s’appliquent sans changement, et un pas bloqué à une borne n’émet rien. Les champs avec `step: "any"`, en lecture seule ou désactivés ne bougent pas. Définissez `wheel-stepping: true` pour faire aussi avancer l’éditeur ayant le focus à la molette ; `step-by` et `step-to-bound` exposent le même chemin à des boutons personnalisés. Les boutons de `KitNumberStepper` valident toujours chaque appui aussitôt.

## Brouillon et acceptation

`text` reste le brouillon de l’éditeur natif, y compris lorsqu’il est incomplet. `field-state` expose valeur acceptée, brouillon, indicateurs dirty/touched et dernière transition ; `accepted-text` projette le dernier texte accepté. `committed(FieldCommit)` contient le candidat et ses contraintes. La clé du codec est une donnée applicative, indépendante de la position visuelle.

`commit-policy` vaut `CommitPolicy.on-blur` par défaut. Choisissez `on-submit` ou `on-change-when-valid` selon le besoin. `commit()` signale la validité et demande l’acceptation, `cancel-edit()` restaure le dernier brouillon accepté et `reset-field()` restaure `default-value` et la validation. `ownership: Ownership.controlled` émet des propositions ; confirmez les valeurs acceptées par l’hôte avec `acknowledge(FieldValue)`. Le [guide des contrôleurs](/fr/guides/controllers/) explique codecs personnalisés et décimaux exacts.

`access: FieldAccess.read-only` conserve la sélection native du texte et interdit l’acceptation. Un contrôle désactivé refuse l’interaction. Curseur, sélection, annulation et IME restent dans le TextInput natif. Les tests sans écran exercent les chemins publics d’entrée ; ils ne qualifient pas tous les claviers physiques, moteurs de composition ou lecteurs d’écran.

Les champs multilignes réservent les glissements de souris à la sélection native. Leur barre verticale reste dans une gouttière dimensionnée par le thème ; la molette défile lorsque le texte déborde.

## Options partagées et composition

Les champs incluent déjà `label`, `description`, `error`, validation et nom accessible. `leading-icon` et `trailing-icon` sont des `IconGlyph` décoratifs ; ils ne remplacent pas le nom accessible textuel. Les options de présentation telles que `decoration`, `corner-radius`, marges et taille du texte conservent éditeur natif et validation partagée.

[KitFieldLabel](/fr/components/field-label/), [KitFieldDescription](/fr/components/field-description/) et [KitFieldError](/fr/components/field-error/) sont des primitives de composition. Leurs liens de compatibilité proviennent des API effectivement héritées. Elles ne fournissent ni éditeur, ni analyse métier, ni contrôleur de validation indépendant. `KitInputGroup` et ses parties start/end/text organisent un champ existant.

## Soumission et réinitialisation

`KitSubmitButton` et `KitFormButton` expriment les actions du formulaire. Avant de soumettre, appelez `report-validity()` sur les champs et focalisez le premier champ invalide. La commande Rust doit aussi recevoir des valeurs métier vérifiées ou valider sa charge utile ; désactiver un bouton ne protège pas la soumission. Les exemples conservent commande et validation métier dans Rust. `KitResetButton` déclenche une action applicative qui réinitialise explicitement les champs et l’état métier.

Les composites calendrier/intervalle valident les indisponibilités en plus des bornes HTML. Le champ avec fuseau refuse les heures inexistantes et exige une occurrence explicite lorsqu’une heure se répète. Son `ZonedDateTimeCommit` accepté se convertit en `ResolvedDateTime` ; aucun fuseau n’est déduit d’une date-heure locale.

Les dates et heures saisies suivent la même `commit-policy` que les champs texte. Avec `CommitPolicy.on-blur` par défaut, `KitDatePicker` accepte une date valide lorsque le focus quitte son éditeur, `KitDateTimeField` accepte la paire lorsque le focus quitte ses deux éditeurs et `KitZonedDateTimeField` résout le brouillon lorsque le focus quitte ses éditeurs de date, d’heure et de zone. Passer d’un éditeur à l’autre dans un même groupe n’est pas une perte de focus. Un texte invalide conserve la valeur acceptée et le brouillon, et une valeur inchangée ne répète pas `chosen`.

`on-change-when-valid` émet la valeur finale de chaque édition native. Les flèches et modificateurs ne changent pas l’état d’acceptation.

`report-validity()` ne déplace jamais le focus. Signalez tous les champs, puis appelez `focus-if-invalid()` uniquement sur le premier champ invalide pour éviter les effets secondaires du flou pendant la collecte des erreurs.

`KitSubmitButton` exige un callback `validation-requested` raccordé qui retourne la validité du formulaire, ou `form-no-validate` explicite. Traitez `form-action(FormAction)` pour soumettre/réinitialiser ; `activated` observe ensuite l’action et ne remplace pas la validation. `KitResetButton` ne valide pas.

L’édition expose les offsets UTF-8 en octets via `cursor-position`, `selection-anchor` et `set-selection(anchor, cursor)`. `insert-at-cursor(text)` prend le focus et suit l’insertion et les filtres natifs ; `undo()`, `redo()`, `cut-selection()`, `copy-selection()` et `paste()` conservent le même éditeur. `blur()` libère son focus. Utilisez `context-menu: TextContextMenu.disabled` pour remplacer le menu standard localisable.

Les mots de passe masqués empêchent couper/copier dans le menu et les fonctions publiques ; les autres champs en lecture seule conservent copie et sélection.

Le popup thémé s’ouvre par clic droit, Menu ou Maj+F10. Les flèches et Début/Fin parcourent les actions actives ; Entrée agit, Échap revient à l’éditeur et Tab poursuit le parcours normal. Ouvrir ce menu conserve le focus logique du champ sans validation par flou. Le popup expose les rôles liste/élément car Slint 1.18 ne fournit pas de rôles menu ; localisez son nom via `KitStrings.text-menu-label`. L’appui tactile prolongé n’est pas fourni.Slint 1.18 restaure le focus parent à la fermeture du popup ; le clic extérieur de fermeture peut être consommé avant que le contrôle visé reçoive le focus. Sa composition utilise les API Slint publiques sans imposer de décodeur d’image ni de moteur de rendu.

Localisez les valeurs par défaut via le global de fenêtre `KitStrings`, par exemple `KitStrings.required-message = "Ce champ est obligatoire."`. Les modèles utilisent `{count}`, `{value}`, `{min}` et `{max}`. Les propriétés de message de chaque champ restent prioritaires. Configurez l’instantané des messages d’un `FieldBinding` avec `binding.set_messages(window.global::<slint_kit::ui::KitStrings>().get_fields())` ; actualisez cet instantané après un changement de langue. Les erreurs de parseur typées sont traduites avec ces messages et les erreurs métier conservent le texte de leur formateur.

La [référence du champ texte](/fr/components/text-field/) explique les payloads des callbacks, fonctions natives, focus, visibilité de validation et accessibilité. Les variantes héritent de ces API. `focus-observed` suit l’éditeur et son menu d’édition, `secret-visibility` régit la présentation du mot de passe et `input-method-hints` exprime les préférences natives de capitalisation/correction/complétion disponibles dans Slint. Localisez libellés d’action et modèles de compteur via `KitStrings` ; les propriétés de champ restent prioritaires.


Les contrôles à contraintes de texte, choix, fichiers et date/heure partagent `report-validity()` sans prise de focus et `focus-if-invalid()` explicite. Enregistrez signalement et focus séparément dans un formulaire mixte. Le signalement date/heure avec fuseau vérifie les parties locale et fuseau avant de retourner ; son helper cible la première partie invalide. **Changement de comportement :** les signalements des choix, fichiers et dates/heures composées ne déplacent plus automatiquement le focus.

Un sélecteur de fichier ou de dossier en lecture seule peut rester candidat à la validation alors que son action de parcours est inactive. Son helper exprime une demande de focus sans confirmation par la plateforme ; conservez un parcours modifiable pour corriger la sélection.


Composez directement les actions de début et de fin dans `KitTextField` avec `KitInputGroupStart/End`. Liez `layout-order` de chaque partie à `field.leading-adornment-order` ou `field.trailing-adornment-order` ; l’unique éditeur natif conserve filtrage, sélection, annulation, contraintes et politique de commit. Chaque action possède son nom accessible et sa garde d’accès. Utilisez `focus-on-pointer: false` pour une action auxiliaire qui ne doit pas faire perdre le focus. Le `layout-order` visuel de Slint conserve l’ordre Tab de déclaration : éditeur, actions intégrées, puis enfants déclarés. `clearable` contrôle le bouton d’effacement monoligne ; `reveal-toggle: SecretToggleVisibility.hidden` masque le bouton de révélation sans modifier `secret-visibility`. Observez `cleared()` après un effacement réel et `secret-visibility-changed(SecretVisibility)` après une transition. `busy` et `busy-label` affichent une activité indépendamment de la validation, sans modifier `access` ni masquer la validation en attente. Le défaut se règle par `KitStrings.busy-label`.


Conservez les mots de passe acceptés dans un état applicatif typé. Affichez seulement un statut d’acceptation non sensible ; ne rendez jamais accepted-text ni le texte de la valeur validée à côté du champ et n’imprimez jamais le mot de passe dans les journaux.

## Roue de couleur contrôlée

`KitColorWheel` propose des `SrgbColorView` opaques à valeur HSV maximale. Installez le kit une fois par fenêtre, liez `value` et `known`, appliquez `chosen` au brouillon local et soumettez ce brouillon à la frontière Rust typée. Incrémentez `generation` au changement de source pour retirer le geste tenu. Gauche/droite règlent la teinte, haut/bas la saturation, Entrée demande confirmation et Échap demande annulation. Le propriétaire restaure la dernière valeur acceptée à l’annulation. Localisez `label`, `unknown-label` et `value-format`, ou leurs valeurs par défaut `KitStrings.color-wheel-*`. Ce contrôle à deux axes expose un groupe de focus nommé et une description ; l’édition par lecteur d’écran natif n’est pas qualifiée.
