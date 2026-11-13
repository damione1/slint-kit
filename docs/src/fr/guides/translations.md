---
layout: base.njk
locale: fr
title: "Traductions"
translationKey: "guides/translations/"
permalink: "{{ 'guides/translations/' | localeUrl(locale) }}index.html"
---

# Traductions

Une application multilingue traduit deux sortes de textes : le sien, et celui que les composants du kit affichent (noms de boutons, messages de validation, noms de mois). Slint Kit n’écrit jamais en dur la seconde sorte. Chaque phrase intégrée est une propriété du global `KitStrings`, local à la fenêtre, avec une valeur par défaut en anglais. Vous écrivez chaque phrase une seule fois, dans votre propre catalogue, et vous changez de langue à l’exécution.

## Pourquoi `KitStrings` et pas `@tr` dans le kit

Slint traduit le texte `@tr(...)` avec des catalogues gettext qui appartiennent à l’application. Le kit est une bibliothèque : il ne livre pas de catalogue et n’appelle pas `@tr` lui-même. Un catalogue de bibliothèque ne pourrait pas suivre votre liste de langues, vos règles de pluriel ni votre ordre des mots. Le kit expose donc son texte sous forme de propriétés et de gabarits (`{label}`, `{count}`, `{title}`), et vous les affectez depuis `@tr`. Un gabarit nomme ses emplacements, donc chaque langue choisit son ordre des mots : `Dismiss {title}` devient `Fermer {title}`.

Les propriétés propres à un composant, comme `required-message` ou `dismiss-label`, remplacent toujours la valeur globale pour une seule instance.

## Configurer la compilation

Ajoutez un catalogue par langue sous `lang/<code>/LC_MESSAGES/<nom-du-crate>.po` et demandez au compilateur de les intégrer :

```rust
// build.rs
use slint_build::DefaultTranslationContext;

fn main() {
    let configuration = kit_build::config()
        .with_bundled_translations("lang")
        .with_default_translation_context(DefaultTranslationContext::None);
    kit_build::compile_with_config("ui/app.slint", configuration).expect("compiler l’interface");
}
```

Ajoutez `slint-build` à côté de `slint-kit-build` dans `[build-dependencies]`. Les catalogues sont compilés dans le binaire : aucun fichier n’est lu à l’exécution et aucune bibliothèque gettext n’est nécessaire.

## Faire passer le texte du kit par `@tr`

Créez un global qui affecte chaque propriété de texte de `KitStrings` depuis `@tr`. L’exemple de référence le fournit complet, avec les 150 propriétés et un catalogue français :

```slint
import { KitStrings } from "@kit";

export global KitTranslations {
    public function apply() {
        KitStrings.required-message = @tr("Please fill out this field.");
        KitStrings.email-message = @tr("Please enter a valid email address.");
        // Un gabarit du kit garde ses emplacements : doublez les accolades dans @tr.
        KitStrings.dismiss-label-template = @tr("Dismiss {{title}}");
        // Donnez un contexte quand deux phrases ont le même texte anglais.
        KitStrings.pagination-position-template = @tr("pagination" => "{{current}} / {{total}}");
    }
}
```

Deux règles viennent de `@tr` lui-même. Une accolade simple désigne un emplacement gettext : un emplacement du kit comme `{title}` s’écrit donc `{{title}}` dans la source et dans le catalogue, et le résultat est le texte littéral `{title}` que le kit attend. Le texte d’application au pluriel utilise `@tr("{n} message" | "{n} messages" % count)`, avec `msgid_plural` et un en-tête `Plural-Forms` dans le catalogue.

Traduisez ensuite le texte de l’application dans les mêmes fichiers :

```slint
title: @tr("Account settings");
KitTextField { theme: KitThemes.light; label: @tr("Full name"); required: true; }
```

## Changer de langue

Le texte `@tr` suit seul le catalogue sélectionné. `KitStrings` reçoit une copie : affectez-le donc de nouveau après chaque changement. Gardez la langue sous forme d’énumération en Rust et convertissez-la une seule fois, à la frontière avec Slint :

```rust
#[derive(Clone, Copy)]
enum Language {
    English,
    French,
}

impl Language {
    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
        }
    }
}

fn translate(app: &App, language: Language) {
    slint::select_bundled_translation(language.code()).expect("intégré à la compilation");
    app.invoke_translate(); // appelle KitTranslations.apply()
}
```

`"en"` sélectionne le texte écrit dans les sources. Appelez `select_bundled_translation` après la création de la première fenêtre, puis appelez `translate` une fois au démarrage et à chaque fois que l’utilisateur choisit une autre langue. Les fenêtres ouvertes se mettent à jour tout de suite et les valeurs des champs restent intactes.

Si vous utilisez un `FieldBinding`, rafraîchissez son instantané de messages après le changement avec `binding.set_messages(window.global::<slint_kit::ui::KitStrings>().get_fields())`.

## Les formats sont explicites, jamais déduits

La traduction ne change pas la façon d’analyser ou d’afficher les valeurs. Choisissez vous-même les formats et changez-les avec la langue :

- Nombres et montants : passez `NumberSyntax::fr_ca()` ou `NumberSyntax::en_ca()` à la spécification du champ et réglez `number-format` en conséquence (voir [Formulaires](/fr/guides/forms/)).
- Dates : donnez aux calendriers et aux sélecteurs de date `month-names`, `weekday-names` ainsi que `title-pattern` et `accessible-date-pattern` correspondants, ou utilisez `DateNames::french()` en Rust.
- Langues écrites de droite à gauche : le kit inverse la disposition et la navigation au clavier quand la direction de la fenêtre est de droite à gauche.

## Essayer l’exemple

Le crate `docs/translations` du dépôt est une application complète en français et en anglais. Il contient le script de compilation, le global `KitTranslations` avec tout le texte du kit, le catalogue français et une fenêtre avec un sélecteur de langue. Lancez-le avec `cargo run -p kit-docs-translations`, ou passez `fr` pour démarrer en français.

Des tests sans affichage vérifient le mécanisme, la couverture de chaque propriété de `KitStrings` et les emplacements du texte français. Ils ne qualifient ni un lecteur d’écran ni une disposition de droite à gauche sur un appareil.
