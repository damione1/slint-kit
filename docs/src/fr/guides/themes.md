---
layout: base.njk
locale: fr
title: "Thèmes et accessibilité"
translationKey: "guides/themes/"
permalink: "{{ 'guides/themes/' | localeUrl(locale) }}index.html"
---

# Thèmes et accessibilité


Un thème est une valeur `KitTheme`. Transmettez-la à chaque composant composé ; les valeurs publiques par défaut sont `KitThemes.light` et `KitThemes.dark`. Utilisez le même thème pour le fond et le texte de la fenêtre.

```slint
import { KitTheme, KitThemes, KitButton, ControlSize, ControlWidth } from "@kit/kit.slint";

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    in-out property <KitTheme> theme: KitThemes.dark;
    background: root.theme.canvas;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        KitButton {
            theme: root.theme;
            text: "Continue";
            control-size: ControlSize.comfortable;
            control-width: ControlWidth.fill;
        }
    }
}
```

## Couleurs sémantiques

Un thème personnalisé fournit canevas, surfaces normale et élevée ; textes primaire, secondaire et indicatif ; accent, texte sur accent, accent survolé et pressé, accent atténué et sélection de texte ; danger avec ses tons survolé et pressé, succès, avertissement et information avec leurs fonds atténués ; bordures légère et marquée, survol, pression et désactivation ; voile modal ; `reduced-motion` et `text-scale`. Au-delà des couleurs, un thème porte des groupes de jetons typés : `shape` (rayons par rôle pour les contrôles, conteneurs, surcouches et pastilles, et épaisseurs de bordure), `elevation` (un niveau par rôle, et une couche d’ombre principale et une couche ambiante par niveau), `focus` (style, épaisseur, décalage et couleurs de l’anneau), `typography` (famille, graisses, tailles), `spacing` (une unité de base, et la hauteur et les marges de chaque `ControlSize`) et `motion` (durées rapide, de base et lente). Slint n’a pas de syntaxe de décomposition et un champ de struct omis vaut silencieusement zéro ou transparent : écrivez donc un thème personnalisé en littéral complet, ou copiez un thème livré et modifiez ses champs depuis Rust. Les [déclarations publiques du thème](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/ui/theme.slint) définissent ce contrat.

Chaque composant prend par défaut `KitThemeContext.current`, initialisé à `KitThemes.light`. Affectez-le une fois pour définir la présentation de la fenêtre. Un `theme` explicite remplace celui du composant et de ses contrôles internes. Les enfants fournis par l’application via `@children` gardent leurs propres valeurs par défaut : transmettez le thème du parent lors de leur composition. Pour changer le défaut depuis Rust, réexportez `KitThemeContext` du fichier Slint principal, puis appelez `KitThemeContext::get(&window).set_current(theme)`.

`ButtonIntent` propose `primary`, `secondary`, `outline`, `quiet` et `destructive`. `secondary` est une surface neutre remplie, `outline` un contrôle transparent bordé et `quiet` un contrôle sans bordure. Le retour utilisateur utilise des enums de sévérité et de ton. Choisissez l’état selon son sens, puis laissez le thème fournir la couleur. Accompagnez une bordure d’erreur d’un message et d’un libellé accessibles.

Chaque `AlertSeverity` reçoit son propre fond atténué, sa bordure et sa marque à partir des jetons `information`, `success`, `warning` et `danger` et de leurs fonds `*-subtle` : les alertes restent distinctes sans lire leur titre. Les conteneurs défilants, listes et tableaux du kit dessinent leurs barres avec [KitScrollBar](/fr/components/scroll-bar/), qui suit le thème. Un `ScrollView` ou un `ListView` std-widgets dans le code de l’application conserve le style Slint et `Palette.color-scheme`.

## Présentation Tailwind intégrée

Le kit fournit un seul système visuel inspiré de Tailwind, en modes clair et sombre. `KitThemes.light` et `.dark` définissent les surfaces ardoise, les actions indigo, les couleurs sémantiques, la typographie, les espacements, le focus, les élévations et les icônes Lucide. L’intégration standard `slint_kit_build::compile("ui/app.slint")` enregistre tout le nécessaire aux composants.

```slint
import { KitThemes, KitThemeContext, KitButton } from "@kit/kit.slint";

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    in property <bool> dark;
    init => { KitThemeContext.current = root.dark ? KitThemes.dark : KitThemes.light; }
    changed dark => { KitThemeContext.current = root.dark ? KitThemes.dark : KitThemes.light; }
    background: KitThemeContext.current.canvas;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        KitButton { text: "Continuer"; }
    }
}
```

**Inter est une police optionnelle.** Importez `{ InterFont } from "@kit/fonts/inter.slint"`, puis réglez `default-font-family: InterFont.family` sur la fenêtre, comme les exemples documentaires. Sans cet import, le thème utilise la police de l’application. Les quatre graisses statiques totalisent environ 1,6 Mo de fichiers sources ; une application embarquée peut choisir une police adaptée à son périphérique.

Pour ajouter des alias de bibliothèques applicatives, partez de `slint_kit_build::config_with_libraries(...)`, puis utilisez `slint_build::compile_with_config`. Le helper préserve les alias de l’application et réserve `kit`. La hauteur des composants reste intrinsèque : contenu et padding peuvent demander davantage que la taille nominale.

La palette dérive de Tailwind CSS sous MIT, Inter est sous SIL Open Font License et Lucide conserve les notices ISC/MIT. Préservez les [notices tierces](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md). Le rendu des ombres dépend du renderer ; les bordures conservent les limites des surfaces lorsqu’elles sont indisponibles.

## Réglages locaux et régions composées

Utilisez le thème pour l’identité partagée, `control-size` pour la densité et les propriétés locales pour une exception voulue. Leurs défauts suivent la recette active ; modifier le rayon ou le padding conserve hover, focus, désactivation et validation.

| Composant | Présentation locale |
| --- | --- |
| `KitButton`, `KitIconButton`, `KitPressable` | `corner-radius`, `padding-inline`, `padding-block`, `text-size`, `text-weight` |
| `KitTextField` et ses types dérivés | `corner-radius`, `padding-inline`, `padding-block`, `text-size`, `leading-icon`, `trailing-icon`, `decoration`, `underline-segments` |
| `KitCard`, `KitSurface` | `corner-radius`, `elevation` ; propriétés existantes d’inset et de gap |
| `KitCheckbox`, `KitSwitch`, `KitOption`, `KitNavigationAction` | réglages hérités de Pressable sur le libellé et la surface ; géométrie de l’indicateur issue de la recette de famille |
| `KitAlert` | `corner-radius`, `inset` (longueur) ; couleurs toujours résolues par la sévérité |

Les groupes composés, sélecteurs, onglets et menus conservent leurs contrats de thème, taille et composition ; ce tableau ne promet pas les mêmes propriétés locales sur chaque conteneur. Les libellés, aides et erreurs gardent leurs rôles typographiques et ne prennent pas le `text-size` de l’éditeur.

```slint
import { KitButton, ButtonIntent, KitThemeContext } from "@kit/kit.slint";
import { LucideIcons } from "@kit/icons/lucide/all.slint";

component ExportAction inherits KitButton {
    text: "Exporter";
    intent: ButtonIntent.primary;
    leading-icon: LucideIcons.download();
    corner-radius: 10px;
    padding-inline: KitThemeContext.current.spacing.unit * 5;
}

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        ExportAction { }
    }
}
```

`text-size` est une longueur de rendu. Son défaut inclut l’échelle du texte ; reliez aussi une taille personnalisée à `theme.text-scale` si elle doit suivre cette préférence. Remplacer entièrement une `recipe` est une surcharge avancée : préservez tous ses champs et les états qu’elle résout, plutôt qu’un littéral partiel.

Une région locale nécessite une transmission explicite aux enfants applicatifs :

```slint
import { KitCard, KitButton, KitThemes } from "@kit/kit.slint";

component DarkPanel inherits KitCard {
    theme: KitThemes.dark;
    KitButton { theme: root.theme; text: "Continuer"; }
}

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        DarkPanel { }
    }
}
```

Un contrôle voisin sans thème explicite continue d’utiliser le contexte de la fenêtre. Ce contrat de composition n’est pas une cascade CSS d’ancêtres.

## Icônes

`KitIcon` et `KitIconButton` peignent un `IconGlyph`. Les glyphes propres au kit sont dans `KitGlyphs` (`KitGlyphs.check()`, `KitGlyphs.chevron-down()`…). Toutes les icônes de [Lucide](https://lucide.dev) 1.64.0 sont aussi disponibles : `@kit/icons/lucide/all.slint` exporte `LucideIcons`, et chaque catégorie Lucide a son propre fichier avec les mêmes glyphes, par exemple `LucideArrows` dans `@kit/icons/lucide/arrows.slint`. Chaque icône est une fonction nommée : un nom mal orthographié échoue à la compilation, et seules les icônes appelées par l’application arrivent dans son binaire.

[Parcourir toutes les icônes Lucide et copier leur intégration](/fr/components/icon/).

```slint
import { KitIcon, KitIconButton, IconPurpose } from "@kit/kit.slint";
import { LucideIcons } from "@kit/icons/lucide/all.slint";

component Toolbar inherits HorizontalLayout {
    spacing: 8px;
    KitIconButton { glyph: LucideIcons.trash(); accessible-name: "Supprimer"; }
    KitIcon { glyph: LucideIcons.cloud-check(); purpose: IconPurpose.informative; label: "Enregistré"; }
}

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        Toolbar { }
    }
}
```

La taille d’une icône suit `control-size` (14, 16 ou 20 px) multiplié par le `text-scale` du thème. Les icônes dans les contrôles prennent la taille de leur hôte : `KitIconButton`, actions des champs, chevrons et coches des sélecteurs et combobox, boutons du champ numérique ; le bouton d’effacement ou d’affichage d’un champ grandit avec son icône à 200 %. L’épaisseur du trait vient du jeton `shape.icon-stroke` du thème, 2 dans tous les thèmes fournis. Avec `stroke-mode: IconStroke.scaled`, la valeur par défaut, le jeton est en unités de grille et suit la taille de l’icône, comme en SVG : les 2 unités sur 24 de Lucide font 1,33 px à 16 px et 2,67 px avec un texte à 200 %. Avec `IconStroke.absolute`, le trait reste à 2 px quelle que soit la taille. Un `IconGlyph` a une couche `stroke` (tracée) et une couche `fill` (remplie, vide pour la plupart des icônes) : quelques icônes Lucide, comme `tag` ou `palette`, ont des points pleins, peints avec le même `ink`. Transmettez `ink` depuis la recette du composant qui l’entoure. Une icône est décorative par défaut et reste hors de l’arbre d’accessibilité ; quand elle porte un sens qu’aucun texte voisin ne donne, réglez `purpose: IconPurpose.informative` et un `label` : elle est alors annoncée comme une image.

Les icônes intégrées aux contrôles lisent les rôles sémantiques de `theme.icons`, un `KitIconSet`. Le thème intégré fournit des glyphes Lucide pour ces rôles. Surchargez un rôle dans une copie du thème pour modifier ses consommateurs, ou utilisez la propriété de glyphe explicite du contrôle lorsqu’elle existe. Les listes déroulantes, les combobox, les lignes dépliables et les en-têtes de tableau triés font tourner leur chevron quand ils s’ouvrent, se ferment ou changent de sens, pendant la durée `motion.fast` du thème, ou immédiatement avec `reduced-motion`. Au repos, le chevron est dessiné sans rotation : il pointe donc dans le bon sens avec tous les moteurs de rendu. Les puces signalent leur retrait par le glyphe de fermeture.


Les fichiers Lucide sont générés à partir d’une version recopiée dans le dépôt et ne sont jamais modifiés à la main. Lucide est sous licence ISC, et les icônes dérivées de Feather sont aussi sous licence MIT ; les deux notices figurent dans [THIRD-PARTY-NOTICES.md](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md) et sont livrées avec la crate.

## Taille et mise en page

`ControlSize` vaut `compact`, `medium` ou `comfortable`. `ControlWidth.intrinsic` conserve la largeur du contenu ; `fill` utilise l’espace attribué par la mise en page. La hauteur est généralement intrinsèque. Donnez un viewport fini aux [composants de données virtualisées](/fr/components/#data) et conservez l’identité du modèle pendant le défilement.

`LayoutSpace` fournit les valeurs partagées de marges internes et d’espacement, notamment pour `KitCard` et `KitHero`. Utilisez [KitStack](/fr/components/stack/), [KitInline](/fr/components/inline/) et les autres conteneurs lorsque leur contrat d’espacement convient à l’application.

## Agrandissement du texte et mouvement

Pour un texte à 200 %, copiez un thème et réglez `text-scale` sur `2`. Pour réduire le mouvement, réglez `reduced-motion` sur `true`. Ce sont des choix du thème applicatif ; l’hôte doit les relier aux préférences réelles du système si nécessaire. Laissez grandir le texte et les contrôles plutôt que de les couper dans un contenu de hauteur fixe.

## Vérifier l’expérience native

Examinez les thèmes clair et sombre, le texte à 200 %, la navigation clavier, le focus visible et les actions désactivées dans l’application réelle. Préservez l’éditeur natif pour curseur, sélection, annulation et IME. Donnez un nom accessible pertinent aux boutons d’icône, surfaces pressables et cibles de focus.

Ce site propose des previews navigateur et des captures natives ; leurs légendes indiquent l’environnement de rendu. Un exemple rendu ne qualifie ni lecteur d’écran, ni IME, ni dépôt de fichiers externe, ni autre backend. La qualification des périphériques et technologies d’assistance relève des plateformes ciblées par l’application. La [documentation d’accessibilité de Slint](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) décrit les propriétés et les outils d’inspection native.

Utilisez `KitAvatar` avec `AvatarInteraction.action` pour accéder à un profil : fournissez un nom d’action et traitez `activated`. Les avatars statiques restent des images. `KitBadge` et `KitStat` acceptent des glyphes `leading-icon` optionnels ; gardez le statut et la valeur lisibles. Les champs OTP soulignés utilisent toujours un seul éditeur natif. Alignez les `slots` visuels sur la longueur de code acceptée par l’hôte. Pour les infobulles des boutons Kit, liez `target-focused` à `focus-visible` afin de garder l’aide au survol sans ouvrir une projection clavier après un clic.

Les valeurs `KitTypography` personnalisées doivent fournir `family-monospace`, utilisé pour la saisie de codes. Les thèmes fournis utilisent la famille à chasse fixe de la plateforme. `KitTextField.text-cell-width` mesure le glyphe zéro réel pour aligner les emplacements de code.
