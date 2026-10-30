---
layout: base.njk
locale: fr
title: "Migrer vers la 0.4"
translationKey: "guides/migration-0-4/"
permalink: "{{ 'guides/migration-0-4/' | localeUrl(locale) }}index.html"
---

# Migrer vers Slint Kit 0.4

Gardez `slint-kit`, son intégration de build et le pack de présentation éventuel à des versions correspondantes. Le comportement des composants reste dans le kit de base ; le pack Tailwind optionnel fournit la présentation et les ressources de police.

## Application minimale

Une application avec le thème clair ou sombre intégré conserve `slint-kit-build` et `slint_kit_build::compile("ui/app.slint")`. Elle n’a besoin ni du pack Tailwind ni d’un import de police.

Les littéraux `KitTheme` personnalisés incluent désormais les groupes de tokens typés et `icons: KitIconSet`. Copiez un thème fourni puis modifiez ses champs, plutôt qu’un littéral partiel : Slint remplit les champs omis avec zéro ou une valeur transparente. Utilisez les réglages locaux du composant pour une exception isolée.

## Déplacer l’import Tailwind optionnel

L’ancien fichier `@kit/themes/tailwind.slint` a été déplacé dans `slint-kit-tailwind`. Ajoutez le pack aux dépendances de build :

```toml
[build-dependencies]
slint-kit-tailwind = "0.4"
```

```rust
fn main() {
    slint_kit_tailwind::compile("ui/app.slint").expect("compile UI");
}
```

Remplacez l’ancien import par :

```slint
import { TailwindThemes } from "@kit-tailwind/theme.slint";
```

Le thème n’enregistre plus Inter implicitement. Pour conserver Inter, importez `{ InterFont } from "@kit-tailwind/fonts/inter.slint"` et réglez `default-font-family: InterFont.family` sur la fenêtre. Omettez les deux pour employer le même thème avec la police de l’application.

Pour des bibliothèques applicatives, transmettez leur table d’alias à `slint_kit_tailwind::config_with_libraries(...)`, puis compilez avec `slint_build::compile_with_config`. Le pack vérifie sa version face à kit-build et refuse les collisions avec les alias `kit` ou `kit-tailwind`.

## Icônes et boutons

`IconKind` est remplacé par des valeurs `IconGlyph`. Remplacez `kind: IconKind.close` sur `KitIcon` et `icon: IconKind.close` sur `KitIconButton` par `glyph: KitGlyphs.close()`. Les fonctions de glyphes Lucide ont le même type.

Les boutons acceptent `leading-icon` et `trailing-icon` : une action ordinaire avec icône et libellé ne nécessite plus un Pressable personnalisé. `KitIconButton.intent` sélectionne désormais son intention visuelle comme les boutons texte. Vérifiez les boutons icônes dont l’intention héritée était réglée mais auparavant ignorée.

Les rôles d’icône internes suivent `theme.icons` : changer de thème modifie les chevrons et actions intégrés ainsi que les icônes applicatives. Un rôle personnalisé doit être une valeur `IconGlyph` valide, pas un nom sous forme de chaîne. [Parcourir la galerie](/fr/components/icon/).

## Portée du thème et réglages locaux

`KitThemeContext.current` fournit le défaut de la fenêtre. Un `theme` explicite remplace celui d’un composant et de ses contrôles internes ; il n’injecte pas le thème dans des `@children` arbitraires. Transmettez explicitement le thème de la région aux enfants composés par l’application. Évitez de modifier le global dans l’initialisation d’un panneau pour simuler une portée imbriquée.

Pour une exception locale, utilisez `corner-radius`, le padding et la typographie lorsqu’ils sont exposés, ou l’`elevation` d’une surface. Leurs défauts suivent la recette active. Réservez les recettes complètes aux surcharges avancées et préservez leurs états. [Exemples de thème et de composition](/fr/guides/themes/).

Testez l’application en clair et sombre, avec le texte agrandi, la navigation clavier et les contrôles désactivés, en lecture seule ou invalides. Vérifiez notamment les tailles d’icône personnalisées, les affixes de champ et les noms des actions sans texte.
