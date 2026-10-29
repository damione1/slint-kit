---
layout: base.njk
locale: fr
title: "Formulaires, options et validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---
# Formulaires, options et validation

Pour un champ courant, choisissez le contrôle et définissez ses options :

```slint
KitEmailField {
    label: "Adresse email";
    placeholder: "nom@example.com";
    description: "Utilisée pour les notifications.";
}
```

Le libellé, l’aide et l’erreur sont déjà intégrés au contrôle. Les briques [KitFieldLabel](/fr/components/field-label/), [KitFieldDescription](/fr/components/field-description/) et [KitFieldError](/fr/components/field-error/) servent à une composition personnalisée. Leurs pages recensent les contrôles qui offrent déjà la propriété correspondante ; chaque référence affiche aussi ses propriétés héritées.

## Valider la valeur acceptée

Les projets téléchargeables de cette documentation utilisent Slint Kit 0.3.0 avec un `FieldBinding` Rust. Le parseur du projet distingue le brouillon de la valeur acceptée : une erreur reste modifiable et ne remplace pas la dernière valeur valide. Essayez les exemples complets [email](/fr/components/email-field/), [URL](/fr/components/url-field/) et [masque](/fr/components/masked-field/).

L’email suit la grammaire HTML : un seul `@`, une partie locale et un domaine valides. Un domaine sans point, comme `nom@machine`, est autorisé. Une URL doit être absolue ; cette validation ne vérifie pas que le site existe. [Référence HTML email et URL](https://html.spec.whatwg.org/multipage/input.html#e-mail-state-(type=email)).

Un champ vide est facultatif sauf si le contrôleur utilise `AbsencePolicy::Required`. Vérifiez aussi la soumission en Rust ; une indication visuelle ne remplace pas le refus d’accepter une valeur invalide. [Contraintes et soumission HTML](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#constraints).

## Masque ou regex

Le masque vérifie une structure à positions fixes. Une regex permet d’exprimer une règle plus libre. Pour ajouter une règle au [projet de champ texte](/fr/components/text-field/), ajoutez `regress = "=0.12.0"` dans ses dépendances, puis remplacez son parseur :

```rust
fn parse(text: &str) -> DraftValue<Value> {
    use std::sync::OnceLock;
    static PATTERN: OnceLock<regress::Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        regress::Regex::with_flags(r"^(?:[A-Z]{3}-\d{4})$", "v")
            .expect("valid application pattern")
    });
    if text.is_empty() {
        DraftValue::Empty
    } else if pattern.find(text).is_some_and(|m| m.range() == (0..text.len())) {
        DraftValue::Parsed(Value(text.into()))
    } else {
        DraftValue::Invalid(ParseIssue::InvalidSyntax)
    }
}
```

Définissez aussi la valeur initiale du projet avec `let initial = Value("ABC-1234".into());`.

`ABC-1234` est accepté ; `abc-1234` et `XABC-1234` sont refusés. Le matching porte sur toute la valeur. Le mode `v` utilise la syntaxe ECMAScript, avec les ensembles Unicode. [API du moteur regex](https://docs.rs/regress/0.12.0/regress/struct.Regex.html).

En HTML, `pattern` s’applique aux champs texte, email, URL, téléphone, recherche et mot de passe ; il ne s’applique pas à `textarea`. Une règle spécifique sur une zone multiligne reste une validation explicite de l’application. [Attribut `pattern`](https://html.spec.whatwg.org/multipage/input.html#the-pattern-attribute).

## Étendre sans remplacer l’éditeur

Conservez le `TextInput` du contrôle pour sa sélection, son curseur et son historique. Personnalisez le thème, les libellés, les préfixes et suffixes ; raccordez votre parseur et vos contraintes de domaine à `FieldBinding`. Pour des règles partagées ou asynchrones, consultez les [contrôleurs typés](/fr/guides/controllers/).
