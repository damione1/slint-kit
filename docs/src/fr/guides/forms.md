---
layout: base.njk
locale: fr
title: "Formulaires, options partagées et validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---
# Formulaires, options partagées et validation

Choisissez un champ, puis réglez ses options :

```slint
KitEmailField {
    label: "Email professionnel";
    required: true;
    pattern: ".+@example\\.com";
    pattern-message: "Utilisez votre adresse example.com.";
}
```

`label`, `description` et `error` sont intégrés aux champs. [KitFieldLabel](/fr/components/field-label/), [KitFieldDescription](/fr/components/field-description/) et [KitFieldError](/fr/components/field-error/) servent à composer une présentation personnalisée. Leurs pages listent les contrôles compatibles ; chaque référence affiche aussi ses propriétés héritées.

## Initialiser la validation

Les projets complets utilisent Slint Kit **0.3.0** et initialisent le moteur une fois par fenêtre. Pour l’ajouter à votre application, exportez ces types depuis le fichier Slint principal, puis installez le moteur dans Rust :

```slint
export { KitFormValidation, HtmlInputKind, NumberInputFormat,
    KitInputConstraints, KitValidationResult, KitValidityState } from "@kit/kit.slint";
```

```rust
// Après App::new(), avant app.run().
slint_kit::init_form_validation!(self, app);
```

Si le code généré est dans `mod ui`, utilisez `init_form_validation!(ui, app)`. Sans initialisation, `validation-ready` reste faux et les vérifications refusent la validation. Les projets [email](/fr/components/email-field/), [URL](/fr/components/url-field/) et [champ texte](/fr/components/text-field/) contiennent les fichiers complets.

## Choisir les contraintes

| Option | Usage |
| --- | --- |
| `required` | Refuser une valeur vide ou une sélection obligatoire manquante. |
| `multiple` | Autoriser plusieurs emails séparés par des virgules. |
| `min-length`, `max-length` | Longueur des champs texte en unités UTF-16 ; `-1` désactive la limite. Le brouillon natif reste modifiable. |
| `pattern`, `pattern-message` | Regex ECMAScript `v`, appliquée à toute la valeur. |
| `min`, `max`, `step` | Bornes et pas des nombres, dates et heures, sous forme canonique ; `step: "any"` enlève la contrainte de pas. |
| `default-value` | Valeur de réinitialisation et base du pas si `min` est absent. |
| `validation-timing` | `on-blur` par défaut ; aussi `on-input` et `on-submit`. |
| `custom-validity` | Erreur de l’application ; une chaîne vide la retire. |

Un email incomplet devient une erreur à la perte du focus. Un second `@` ou un caractère interdit révèle l’erreur dès la saisie. `name@machine` est valide selon HTML : un point dans le domaine n’est pas obligatoire. Une URL doit être absolue ; `https://example.comddf.dofmo@` est invalide, tandis que `mailto:a@example.com` est valide. La validation ne vérifie ni l’existence d’une boîte email ni celle d’un site. [Règles HTML email et URL](https://html.spec.whatwg.org/multipage/input.html).

`validity` expose les motifs d’erreur indépendants ; `value` fournit la valeur canonique et `text` conserve le brouillon natif. `check-validity()` vérifie ; `report-validity()` affiche l’erreur et place le focus. Les champs désactivés sont exclus, ainsi que les champs texte en lecture seule. [Validation des formulaires HTML](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#constraints).

## Protéger la valeur acceptée en Rust

Un message visuel ne remplace pas la vérification au moment d’accepter la valeur. Les exemples utilisent `FieldBinding` avec les mêmes contraintes :

```rust
field.set_html_constraints(slint_kit::HtmlConstraints {
    kind: slint_kit::HtmlInputKind::Email,
    required: true,
    pattern: Some(slint_kit::HtmlPattern::compile(r".+@example\.com").expect("valid application pattern")),
    ..Default::default()
});
```

Le moteur vérifie avant le parseur métier. Le parseur reçoit la valeur normalisée ; un brouillon invalide ne remplace pas la dernière valeur acceptée. Pour plusieurs champs, utilisez `HtmlFormValidation<CleChamp>` et `first_invalid()`, ou reliez `KitSubmitButton.validation-requested`. `form-no-validate` contourne cette vérification ; une réinitialisation ne soumet pas.

## Masque ou regex

Un [masque](/fr/components/masked-field/) guide une structure fixe. Une regex valide sans réécrire la saisie. Par exemple, `pattern: "[A-Z]{3}-[0-9]{4}"` accepte `ABC-1234`, refuse `abc-1234` et refuse `XABC-1234`. Il s’agit d’une option commune, pas d’un composant supplémentaire.

HTML n’applique pas `pattern` à `textarea`. Pour une règle multiligne, utilisez `HtmlPattern` en Rust et transmettez son résultat avec `custom-validity`. Une regex invalide est ignorée comme en HTML ; `pattern-error` expose le diagnostic. `pattern-enabled: true` distingue un motif explicitement vide de l’absence de motif. [Attribut HTML pattern](https://html.spec.whatwg.org/multipage/input.html#the-pattern-attribute).

## Composer et étendre

Conservez l’éditeur natif pour le curseur, la sélection et l’historique. Les thèmes, libellés, messages, préfixes et suffixes restent personnalisables. Pour un autre habillage, importez le composant non visuel `FormConstraintState` depuis `@kit/validation.slint`. Les règles métier, montants exacts, masques, alphabets OTP et résolution des fuseaux restent dans les contrôleurs typés. [Contrôleurs](/fr/guides/controllers/).

Le kit fournit une validation native inspirée du standard HTML. Le transport HTTP des formulaires, l’autofill des navigateurs et les claviers propres à chaque plateforme nécessitent une intégration hôte ; les contrôles de couleur conservent leur modèle sRGB typé.
