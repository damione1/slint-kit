---
layout: base.njk
locale: fr
title: "Formulaires, options partagées et validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---

# Formulaires, options partagées et validation

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

## Brouillon et acceptation

`text` reste le brouillon de l’éditeur natif, y compris lorsqu’il est incomplet. `field-state` expose valeur acceptée, brouillon, indicateurs dirty/touched et dernière transition ; `accepted-text` projette le dernier texte accepté. `committed(FieldCommit)` contient le candidat et ses contraintes. La clé du codec est une donnée applicative, indépendante de la position visuelle.

`commit-policy` vaut `CommitPolicy.on-blur` par défaut. Choisissez `on-submit` ou `on-change-when-valid` selon le besoin. `commit()` signale la validité et demande l’acceptation, `cancel-edit()` restaure le dernier brouillon accepté et `reset-field()` restaure `default-value` et la validation. `ownership: Ownership.controlled` émet des propositions ; confirmez les valeurs acceptées par l’hôte avec `acknowledge(FieldValue)`. Le [guide des contrôleurs](/fr/guides/controllers/) explique codecs personnalisés et décimaux exacts.

`access: FieldAccess.read-only` conserve la sélection native du texte et interdit l’acceptation. Un contrôle désactivé refuse l’interaction. Curseur, sélection, annulation et IME restent dans le TextInput natif. Les tests sans écran exercent les chemins publics d’entrée ; ils ne qualifient pas tous les claviers physiques, moteurs de composition ou lecteurs d’écran.

## Options partagées et composition

Les champs incluent déjà `label`, `description`, `error`, validation et nom accessible. `leading-icon` et `trailing-icon` sont des `IconGlyph` décoratifs ; ils ne remplacent pas le nom accessible textuel. Les options de présentation telles que `decoration`, `corner-radius`, marges et taille du texte conservent éditeur natif et validation partagée.

[KitFieldLabel](/fr/components/field-label/), [KitFieldDescription](/fr/components/field-description/) et [KitFieldError](/fr/components/field-error/) sont des primitives de composition. Leurs liens de compatibilité proviennent des API effectivement héritées. Elles ne fournissent ni éditeur, ni analyse métier, ni contrôleur de validation indépendant. `KitInputGroup` et ses parties start/end/text organisent un champ existant.

## Soumission et réinitialisation

`KitSubmitButton` et `KitFormButton` expriment les actions du formulaire. Avant de soumettre, appelez `report-validity()` sur les champs et focalisez le premier champ invalide. La commande Rust doit aussi recevoir des valeurs métier vérifiées ou valider sa charge utile ; désactiver un bouton ne protège pas la soumission. Les exemples conservent commande et validation métier dans Rust. `KitResetButton` déclenche une action applicative qui réinitialise explicitement les champs et l’état métier.

Les composites calendrier/intervalle valident les indisponibilités en plus des bornes HTML. Le champ avec fuseau refuse les heures inexistantes et exige une occurrence explicite lorsqu’une heure se répète. Son `ZonedDateTimeCommit` accepté se convertit en `ResolvedDateTime` ; aucun fuseau n’est déduit d’une date-heure locale.
