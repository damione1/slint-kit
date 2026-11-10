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

`KitMaskedField` accepte un `mask` déclaratif : `#` pour un chiffre ASCII, `A` pour une lettre ASCII, `*` pour un emplacement alphanumérique ASCII, `\` pour échapper un littéral et les autres caractères comme séparateurs littéraux (1 à 256 jetons, au moins un emplacement). Après `slint_kit::install(&app)`, le moteur HTML partagé dérive `min-length`, `max-length`, le `pattern` de valeur complète et le placeholder à emplacements `_`. Le défaut `TextLengthPolicy.validate` conserve le texte hôte et les suppressions, coupes, annulations et rétablissements natifs : un brouillon invalide ou trop long reste visible et ne peut pas être accepté. Changer `mask` valide le texte existant sans le tronquer. Les insertions ordinaires ASCII et le collage natif explicite sont contrôlés avant édition ; un brouillon invalide se répare progressivement à sa première différence. Les entrées non ASCII et les chemins sans événement public avant insertion restent natifs, puis sont validés comme brouillon complet ; cela ne qualifie pas les IME. Les littéraux se saisissent ou se collent explicitement, sans insertion automatique qui réécrirait le curseur. Avant installation, un masque non vide garde le clavier et les actions publiques de collage/coupe en lecture seule. Une syntaxe mal formée renseigne `format-error` et `KitStrings.mask-format-message`, également utilisé comme erreur intrinsèque de validation. Associez `FieldSpec::masked(MaskPattern::compile("AA-####").unwrap())` pour protéger l’acceptation Rust. Le cache conserve au plus 64 sources bornées par `MaskPattern::MAX_SOURCE_BYTES`, y compris les compilations échouées ; il ne conserve jamais une source surdimensionnée.

Les champs numériques suivent le motif du bouton compteur des champs nombre du web : Haut/Bas déplacent d’un `step`, PageUp/PageDown de `page-steps` pas (10 par défaut), et Début/Fin sautent à `min`/`max` lorsque la borne est définie ; sinon ils gardent leur déplacement natif du curseur. Les pas utilisent l’arithmétique décimale exacte sur la grille `min`/`default-value` et remplacent le brouillon comme une frappe : `edited`, `value-edited`, le suivi dirty, l’annulation native et `commit-policy` s’appliquent sans changement, et un pas bloqué à une borne n’émet rien. Les champs avec `step: "any"`, en lecture seule ou désactivés ne bougent pas. Définissez `wheel-stepping: true` pour faire aussi avancer l’éditeur ayant le focus à la molette ; `step-by` et `step-to-bound` exposent le même chemin à des boutons personnalisés. Les boutons de `KitNumberStepper` valident toujours chaque appui aussitôt.

### Limites de plateforme de l’éditeur numérique

Avec Slint 1.18.1, installer le moteur partagé de validation fait utiliser `InputType.text` à l’éditeur natif d’un champ déclaré `input-type: InputType.decimal`. Cela conserve curseur, sélection et annulation natifs tout en admettant des brouillons comme `1e` et la virgule décimale configurée. Le moteur HTML partagé refuse toujours les valeurs incomplètes ou invalides à l’acceptation. Rétablir le type natif `Decimal` filtrerait ces brouillons avant leur validation par le kit.

Le composant numérique expose séparément son rôle de bouton compteur, ses bornes et ses actions incrémenter/décrémenter. Son éditeur natif intérieur reste une saisie de texte : le pont winit/AccessKit ne choisit `NumberInput` que pour les types natifs `Number` ou `Decimal`. Les [indications de méthode de saisie](https://docs.slint.dev/docs/slint/reference/keyboard-input/textinput/#input-method-hints) de Slint 1.18.1 couvrent capitalisation, correction et complétion, sans finalité numérique indépendante. Avec ce SDK, le kit ne peut garantir à la fois la grammaire complète des brouillons et ces métadonnées numériques natives.

Les backends diffèrent. Celui d’Android déduit sa classe de saisie textuelle ou numérique de ce même type natif ; ce constat vient du code source, sans essai de clavier sur appareil. Winit sur ordinateur associe texte et décimal à sa finalité IME normale, tandis que son auxiliaire de saisie WebAssembly ne projette pas le type numérique dans un mode de saisie HTML. Clavier numérique logiciel, composition IME et lecteur d’écran natif doivent être qualifiés sur le backend cible ; aucun rétablissement du clavier numérique n’est affirmé ici.

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


Les contrôles à contraintes de texte, choix, fichiers et date/heure partagent `report-validity()` sans prise de focus et `focus-if-invalid()` explicite. Enregistrez signalement et focus séparément dans un formulaire mixte. Le signalement date/heure avec fuseau vérifie les parties locale et fuseau avant de retourner ; son helper cible la première partie invalide. **Changement de comportement :** les signalements des choix, fichiers et dates/heures composées ne déplacent plus automatiquement le focus. `KitCheckbox`, `KitSwitch` et `KitRadioGroup` s’activent avec `Space` et rejettent `Enter` (`Return`) afin qu’un formulaire englobant puisse gérer la soumission ; `KitRadioGroup` accepte aussi bien `UpArrow`/`DownArrow` que `LeftArrow`/`RightArrow` selon `flow`.

Un sélecteur de fichier ou de dossier en lecture seule peut rester candidat à la validation alors que son action de parcours est inactive. Son helper exprime une demande de focus sans confirmation par la plateforme ; conservez un parcours modifiable pour corriger la sélection.


Composez directement les actions de début et de fin dans `KitTextField` avec `KitInputGroupStart/End`. Liez `layout-order` de chaque partie à `field.leading-adornment-order` ou `field.trailing-adornment-order` ; l’unique éditeur natif conserve filtrage, sélection, annulation, contraintes et politique de commit. Chaque action possède son nom accessible et sa garde d’accès. Utilisez `focus-on-pointer: false` pour une action auxiliaire qui ne doit pas faire perdre le focus. Le `layout-order` visuel de Slint conserve l’ordre Tab de déclaration : éditeur, actions intégrées, puis enfants déclarés. `clearable` contrôle le bouton d’effacement monoligne ; `reveal-toggle: SecretToggleVisibility.hidden` masque le bouton de révélation sans modifier `secret-visibility`. Observez `cleared()` après un effacement réel et `secret-visibility-changed(SecretVisibility)` après une transition. `busy` et `busy-label` affichent une activité indépendamment de la validation, sans modifier `access` ni masquer la validation en attente. Le défaut se règle par `KitStrings.busy-label`.


Conservez les mots de passe acceptés dans un état applicatif typé. Affichez seulement un statut d’acceptation non sensible ; ne rendez jamais accepted-text ni le texte de la valeur validée à côté du champ et n’imprimez jamais le mot de passe dans les journaux.

Les valeurs temporelles facultatives disposent d’un signal d’absence explicite. Dans `KitDatePicker` et `KitDateTimeField`, traitez `cleared()` comme `None` dans le modèle Rust `Option<CivilDate>` / `Option<LocalDateTime>` ; `chosen` contient toujours une valeur non vide. Activez l’action finale avec `clearable` et localisez `clear-label`. `clear()` modifie les brouillons natifs, puis l’acceptation suit `commit-policy` ; `commit()` demande une acceptation explicite validée. Les deux brouillons date/heure doivent être vides pour effacer la paire ; une paire partielle, `required` ou une erreur personnalisée conserve la valeur acceptée. Le mode controlled attend la confirmation de l’absence par l’hôte. Chaque éditeur natif garde sa propre entrée d’annulation. L’action de paire reste dans le groupe de focus ; `editor-focused` décrit toujours les deux éditeurs natifs. L’action date/heure demande séparément l’effacement natif public de chaque éditeur. Si une action est refusée, le brouillon incomplet résultant n’efface pas la paire acceptée. Slint 1.18.1 n’expose ni API documentée d’état de composition ni transaction entre éditeurs ; ne déduisez pas la composition des touches. L’IME natif doit être qualifié sur le backend cible, ce que les tests headless ne fournissent pas.

## Roue de couleur contrôlée

`KitColorWheel` propose des `SrgbColorView` opaques à valeur HSV maximale. Installez le kit une fois par fenêtre, liez `value` et `known`, appliquez `chosen` au brouillon local et soumettez ce brouillon à la frontière Rust typée. Incrémentez `generation` au changement de source pour retirer le geste tenu. Gauche/droite règlent la teinte, haut/bas la saturation, Entrée demande confirmation et Échap demande annulation. Le propriétaire restaure la dernière valeur acceptée à l’annulation. Localisez `label`, `unknown-label` et `value-format`, ou leurs valeurs par défaut `KitStrings.color-wheel-*`. Ce contrôle à deux axes expose un groupe de focus nommé et une description ; l’édition par lecteur d’écran natif n’est pas qualifiée.

## Combobox éditable et recherche distante

`KitCombobox` associe un `KitTextField` natif en ligne, un bouton d’ouverture et un popup de suggestions qui ne vole pas le focus. La saisie filtre `options` localement et émet `query-edited(string)` dès que `query` atteint `min-query-length` (ou lorsqu’elle est effacée), après `query-debounce`. `retention` vaut `SelectionRetention.retain` par défaut : remplacer `options` par une page distante conserve le jeton `selected` accepté, son état `available` et son dernier `selected-label` connu. Activez `allow-custom-value: true` et traitez `custom-chosen(string)` lorsqu’un texte libre hors `options` est permis ; avec `false` (par défaut), la perte de focus restaure `selected-label`. `clearable` et `clear()` réinitialisent `text`, `query` et `selected` et émettent `cleared()`. `search-state: SearchPresentation.loading` affiche un indicateur en ligne et dans le popup tout en conservant les lignes existantes, et `end-reached()` signale l’entrée dans les `near-end-rows` dernières lignes.

## Sens du texte

Les libellés, légendes, aides, erreurs, résumés, éditeurs natifs, placeholders et vues formatées utilisent par défaut le bord logique start. Le kit résout Start/End en Left/Right physiques via `KitTheme.direction`, sans déduire le sens du contenu. Utilisez `direction: ChoiceFlow.right-to-left` pour les défauts RTL. Slint 1.18.1 Software a été observé rendant Start natif à gauche ; les champs projettent donc ces bords physiques explicitement. `text-alignment: left/center/right` explicite est transmis inchangé. `flow` remplace uniquement le chrome du champ/résumé : les adornements et les parties entières `KitInputGroupStart/End` sont inversés ; liez leur `layout-order` projeté. Disposez vous-même le contenu interne des parties. Tab suit l’ordre de déclaration. Une perte de focus Slint ordinaire peut replier la sélection ; les actions clavier d’effacement/révélation restaurent le focus de l’éditeur, tandis que le pointeur conserve focus et sélection. OTP reste explicitement à gauche. Vérifiez façonnage, édition native, IME et accessibilité sur le backend cible. Sur les aides autonomes héritant de Text, une surcharge explicite de `horizontal-alignment` est transmise à Slint ; utilisez un bord physique si nécessaire.

Les champs et sélecteurs couleur utilisent `accepted-color` comme valeur de l’hôte. Ses changements et ceux de `alpha-mode` actualisent les projections hexadécimales/canaux sans focus, même après une édition. Un éditeur natif actif conserve brouillon, curseur et historique ; à la perte du focus, son texte reflète la couleur acceptée courante. Le mode contrôlé attend l’acquittement de l’hôte.

Une édition native valide de couleur hexadécimale ou de canal est acceptée au blur comme sur Entrée ; `chosen` émet une fois pour ce brouillon modifié. Un brouillon invalide ou incomplet retrouve le texte de la couleur acceptée au blur. `commit()` reste une demande immédiate explicite ; les observations `commit-requested` et de demande de canal restent liées à Entrée.
