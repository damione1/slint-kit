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

Le `theme` de chaque composant vaut par défaut `KitThemeContext.current`, initialisé à `KitThemes.light`. Affectez-le une fois pour thémer toute une fenêtre, y compris les composants qui n’ont pas reçu de thème ; la propriété `theme` d’un composant le remplace toujours pour son sous-arbre. Pour le changer depuis Rust, réexportez le global depuis le fichier principal de l’application avec `export { KitThemeContext } from "@kit/kit.slint";`, puis appelez `KitThemeContext::get(&window).set_current(theme)`.

`ButtonIntent` propose `primary`, `secondary`, `outline`, `quiet` et `destructive`. `secondary` est une surface neutre remplie, `outline` un contrôle transparent bordé et `quiet` un contrôle sans bordure. Le retour utilisateur utilise des enums de sévérité et de ton. Choisissez l’état selon son sens, puis laissez le thème fournir la couleur. Accompagnez une bordure d’erreur d’un message et d’un libellé accessibles.

Chaque `AlertSeverity` reçoit son propre fond atténué, sa bordure et sa marque à partir des jetons `information`, `success`, `warning` et `danger` et de leurs fonds `*-subtle` : les alertes restent distinctes sans lire leur titre. Les conteneurs défilants, listes et tableaux du kit dessinent leurs barres avec [KitScrollBar](/fr/components/scroll-bar/), qui suit le thème. Un `ScrollView` ou un `ListView` std-widgets dans le code de l’application conserve le style Slint et `Palette.color-scheme`.

## Thème inspiré de Tailwind

`@kit/themes/tailwind.slint` est un thème optionnel à l’allure de Tailwind CSS : une palette réduite de teintes slate, indigo, red, green, amber et sky, des contrôles `rounded-md`, des conteneurs `rounded-lg` et des surfaces superposées `rounded-xl`, les ombres de Tailwind, un anneau de deux pixels décalé de deux pixels, une hauteur de contrôle par défaut de 36 px et la police [Inter](https://rsms.me/inter/). Importez-le et affectez l’une de ses valeurs ; les composants ne l’importent jamais, donc une application qui ne l’importe pas n’en compile ni n’en embarque rien. Il n’y a aucune feature Cargo à activer.

```slint
import { KitThemeContext, KitButton } from "@kit/kit.slint";
import { TailwindThemes } from "@kit/themes/tailwind.slint";

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    in property <bool> dark;
    init => { KitThemeContext.current = root.dark ? TailwindThemes.dark : TailwindThemes.light; }
    changed dark => { KitThemeContext.current = root.dark ? TailwindThemes.dark : TailwindThemes.light; }
    background: KitThemeContext.current.canvas;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        KitButton { text: "Continuer"; }
    }
}
```

`TailwindThemes.light` et `TailwindThemes.dark` sont des valeurs `KitTheme` complètes ; les règles habituelles s’appliquent donc : modifiez des champs depuis Rust, réglez `text-scale` pour un texte à 200 %, ou transmettez une valeur au `theme` d’un seul sous-arbre. Importer le fichier embarque Inter 4.1 en quatre graisses statiques (400, 500, 600 et 700), soit environ 1,6 Mo de données de police dans le binaire de l’application ; son `typography.family` vaut `"Inter"`. Le texte du kit lit `typography.family`, tandis que le texte propre à l’application garde le `default-font-family` de la fenêtre.

Les couleurs proviennent de la palette de Tailwind CSS, sous licence MIT, et Inter est sous SIL Open Font License 1.1 ; les deux notices figurent dans [THIRD-PARTY-NOTICES.md](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md) et sont livrées avec la crate. Slint dessine une seule ombre par élément et ne gère pas l’étalement (spread) : les cartes et les toasts peignent la couche dominante de leur ombre Tailwind, tandis que les dialogues et les popovers empilent les deux couches sur des rectangles rétrécis de l’étalement de chaque couche. Les boutons pleins (primaire, secondaire, destructif) portent `shadow-xs` par `elevation.control`, peinte par une couche enfant qui ne change jamais leur hauteur mesurée ; les boutons outline et discrets restent plats. Dans le thème sombre, les surfaces élevées gardent le ton des cartes, comme le popover sombre de shadcn, pour que survol, pression et bordure restent visibles sur toutes les surfaces ; bordure, ombre et voile distinguent les surfaces élevées. Avec le moteur de rendu logiciel, qui ne dessine pas d’ombre, les surfaces gardent leur bordure.

## Icônes

`KitIcon` et `KitIconButton` peignent un `IconGlyph`. Les glyphes propres au kit sont dans `KitGlyphs` (`KitGlyphs.check()`, `KitGlyphs.chevron-down()`…). Toutes les icônes de [Lucide](https://lucide.dev) 1.64.0 sont aussi disponibles : `@kit/icons/lucide/all.slint` exporte `LucideIcons`, et chaque catégorie Lucide a son propre fichier avec les mêmes glyphes, par exemple `LucideArrows` dans `@kit/icons/lucide/arrows.slint`. Chaque icône est une fonction nommée : un nom mal orthographié échoue à la compilation, et seules les icônes appelées par l’application arrivent dans son binaire.

```slint
import { KitIcon, KitIconButton, IconPurpose } from "@kit/kit.slint";
import { LucideIcons } from "@kit/icons/lucide/all.slint";

export component Toolbar inherits HorizontalLayout {
    spacing: 8px;
    KitIconButton { glyph: LucideIcons.trash(); accessible-name: "Supprimer"; }
    KitIcon { glyph: LucideIcons.cloud-check(); purpose: IconPurpose.informative; label: "Enregistré"; }
}
```

La taille d’une icône suit `control-size` (14, 16 ou 20 px) multiplié par le `text-scale` du thème. Les icônes dans les contrôles prennent la taille de leur hôte : `KitIconButton`, actions des champs, chevrons et coches des sélecteurs et combobox, boutons du champ numérique ; le bouton d’effacement ou d’affichage d’un champ grandit avec son icône à 200 %. L’épaisseur du trait vient du jeton `shape.icon-stroke` du thème, 2 dans tous les thèmes fournis. Avec `stroke-mode: IconStroke.scaled`, la valeur par défaut, le jeton est en unités de grille et suit la taille de l’icône, comme en SVG : les 2 unités sur 24 de Lucide font 1,33 px à 16 px et 2,67 px avec un texte à 200 %. Avec `IconStroke.absolute`, le trait reste à 2 px quelle que soit la taille. Un `IconGlyph` a une couche `stroke` (tracée) et une couche `fill` (remplie, vide pour la plupart des icônes) : quelques icônes Lucide, comme `tag` ou `palette`, ont des points pleins, peints avec le même `ink`. Transmettez `ink` depuis la recette du composant qui l’entoure. Une icône est décorative par défaut et reste hors de l’arbre d’accessibilité ; quand elle porte un sens qu’aucun texte voisin ne donne, réglez `purpose: IconPurpose.informative` et un `label` : elle est alors annoncée comme une image.

Les composants du kit utilisent les mêmes glyphes. Les listes déroulantes, les combobox, les lignes dépliables et les en-têtes de tableau triés font tourner leur chevron quand ils s’ouvrent, se ferment ou changent de sens, pendant la durée `motion.fast` du thème, ou immédiatement avec `reduced-motion`. Au repos, le chevron est dessiné sans rotation : il pointe donc dans le bon sens avec tous les moteurs de rendu. Les puces signalent leur retrait par le glyphe de fermeture.

Depuis la 0.3 : `IconKind` n’existe plus. Remplacez `KitIcon { kind: IconKind.close; }` par `KitIcon { glyph: KitGlyphs.close(); }`, et `KitIconButton { icon: IconKind.close; }` par `KitIconButton { glyph: KitGlyphs.close(); }`. Le [journal des modifications](https://github.com/damione1/slint-kit/blob/main/CHANGELOG.md) liste chaque changement de la 0.4.0.

Les fichiers Lucide sont générés à partir d’une version recopiée dans le dépôt et ne sont jamais modifiés à la main. Lucide est sous licence ISC, et les icônes dérivées de Feather sont aussi sous licence MIT ; les deux notices figurent dans [THIRD-PARTY-NOTICES.md](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md) et sont livrées avec la crate.

## Taille et mise en page

`ControlSize` vaut `compact`, `medium` ou `comfortable`. `ControlWidth.intrinsic` conserve la largeur du contenu ; `fill` utilise l’espace attribué par la mise en page. La hauteur est généralement intrinsèque. Donnez un viewport fini aux [composants de données virtualisées](/fr/components/#data) et conservez l’identité du modèle pendant le défilement.

`LayoutSpace` fournit les valeurs partagées de marges internes et d’espacement, notamment pour `KitCard` et `KitHero`. Utilisez [KitStack](/fr/components/stack/), [KitInline](/fr/components/inline/) et les autres conteneurs lorsque leur contrat d’espacement convient à l’application.

## Agrandissement du texte et mouvement

Pour un texte à 200 %, copiez un thème et réglez `text-scale` sur `2`. Pour réduire le mouvement, réglez `reduced-motion` sur `true`. Ce sont des choix du thème applicatif ; l’hôte doit les relier aux préférences réelles du système si nécessaire. Laissez grandir le texte et les contrôles plutôt que de les couper dans un contenu de hauteur fixe.

## Vérifier l’expérience native

Examinez les thèmes clair et sombre, le texte à 200 %, la navigation clavier, le focus visible et les actions désactivées dans l’application réelle. Préservez l’éditeur natif pour curseur, sélection, annulation et IME. Donnez un nom accessible pertinent aux boutons d’icône, surfaces pressables et cibles de focus.

Les captures de ce site sont des rendus Slint natifs utilisant **winit et Skia sur macOS**. Elles montrent l’état déclaré par l’exemple avec des libellés adaptés à la langue de la page. Elles ne qualifient ni lecteur d’écran, ni IME, ni dépôt de fichiers externe, ni autre backend. La qualification des périphériques et technologies d’assistance relève des plateformes ciblées par l’application. La [documentation d’accessibilité de Slint](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) décrit les propriétés et les outils d’inspection native.
