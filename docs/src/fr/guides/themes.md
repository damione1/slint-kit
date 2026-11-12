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
import { KitTheme, KitThemes, KitButton, ControlSize, ControlWidth } from "@kit";

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

Les conteneurs de défilement thémés réservent un espace stable sur chaque axe dont la politique de barre n’est pas `always-off`, même quand `as-needed` masque la barre. Liez les contenus sensibles à la largeur à `visible-width`, et ceux sensibles à la hauteur à `visible-height` : ces dimensions excluent l’espace réservé. `KitScrollArea.overlay-scrollbars: true` rétablit les barres sur le contenu et impose au composant appelant de protéger les bords des contrôles.

Chaque composant prend par défaut `KitThemeContext.current`, initialisé à `KitThemes.light`. Affectez-le une fois pour définir la présentation de la fenêtre. Un `theme` explicite remplace celui du composant et de ses contrôles internes. Les enfants fournis par l’application via `@children` gardent leurs propres valeurs par défaut : transmettez le thème du parent lors de leur composition. Pour changer le défaut depuis Rust, réexportez `KitThemeContext` du fichier Slint principal, puis appelez `KitThemeContext::get(&window).set_current(theme)`.

`ButtonIntent` propose `primary`, `secondary`, `outline`, `quiet`, `destructive`, `success`, `warning`, `information` et `link`. `secondary` est une surface neutre remplie, `outline` un contrôle transparent bordé et `quiet` un contrôle sans bordure. Le retour utilisateur utilise des enums de sévérité et de ton. Choisissez l’état selon son sens, puis laissez le thème fournir la couleur. Accompagnez une bordure d’erreur d’un message et d’un libellé accessibles.

Chaque `AlertSeverity` reçoit son propre fond atténué, sa bordure et sa marque à partir des jetons `information`, `success`, `warning` et `danger` et de leurs fonds `*-subtle` : les alertes restent distinctes sans lire leur titre. Les conteneurs défilants, listes et tableaux du kit dessinent leurs barres avec [KitScrollBar](/fr/components/scroll-bar/), qui suit le thème. Un `ScrollView` ou un `ListView` std-widgets dans le code de l’application conserve le style Slint et `Palette.color-scheme`.

## Présentation Tailwind intégrée

Le kit fournit un seul système visuel inspiré de Tailwind, en modes clair et sombre. `KitThemes.light` et `.dark` définissent les surfaces ardoise, les actions indigo, les couleurs sémantiques, la typographie, les espacements, le focus, les élévations et les icônes Lucide. L’intégration standard `slint_kit_build::compile("ui/app.slint")` enregistre tout le nécessaire aux composants.

```slint
import { KitThemes, KitThemeContext, KitButton } from "@kit";

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

Pour ajouter des alias de bibliothèques applicatives, partez de `slint_kit_build::config_with_libraries(...)`, puis utilisez `slint_kit_build::compile_with_config("ui/app.slint", config)`. Le helper préserve les alias de l’application et réserve `kit`. La hauteur des composants reste intrinsèque : contenu et padding peuvent demander davantage que la taille nominale.

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
import { KitButton, ButtonIntent, KitThemeContext } from "@kit";
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
import { KitCard, KitButton, KitThemes } from "@kit";

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
import { KitIcon, KitIconButton, IconPurpose } from "@kit";
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

Ce site propose des previews navigateur et des captures natives ; leurs légendes indiquent l’environnement de rendu. Un exemple rendu ne qualifie ni lecteur d’écran, ni IME, ni dépôt de fichiers externe, ni autre backend. La qualification des périphériques et technologies d’assistance relève des plateformes ciblées par l’application. [Cibles annoncées et cibles qualifiées](/fr/guides/getting-started/) nomme le système, le backend et le moteur de rendu de chaque preuve publiée par le kit, et ce qu’aucune preuve ne couvre. La [documentation d’accessibilité de Slint](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) décrit les propriétés et les outils d’inspection native.

Utilisez `KitAvatar` avec `AvatarInteraction.action` pour accéder à un profil : fournissez un nom d’action et traitez `activated`. Les avatars statiques restent des images. `KitBadge` et `KitStat` acceptent des glyphes `leading-icon` optionnels ; gardez le statut et la valeur lisibles. Les champs OTP soulignés utilisent toujours un seul éditeur natif. Alignez les `slots` visuels sur la longueur de code acceptée par l’hôte. Pour les infobulles des boutons Kit, liez `target-focused` à `focus-visible` afin de garder l’aide au survol sans ouvrir une projection clavier après un clic.

Les valeurs `KitTypography` personnalisées doivent fournir `family-monospace`, utilisé pour la saisie de codes. Les thèmes fournis embarquent et enregistrent JetBrains Mono Regular (OFL 1.1) via `@kit/fonts/monospace.slint`, afin que chiffres, lettres larges et points de masquage gardent la même chasse en natif et en WebAssembly. Le nom système générique `monospace` ne suffit pas sur tous les backends. Une police de code personnalisée doit être enregistrée, réellement à chasse fixe et couvrir l’alphabet ainsi que les glyphes de masquage. `KitTextField.text-cell-width` mesure le glyphe zéro réel pour aligner les emplacements de code.

Pour des sliders dans une modale défilante, activez `KitScrollArea.pan-policy: children-first`. Un enfant interactif conserve son appui jusqu’au relâchement ; démarrez sur le fond ou le texte pour défiler. Ce déplacement est direct, sans inertie. Molette, barres et révélation du focus restent actives. La politique `native` par défaut conserve l’arbitrage Slint.

Pour un contrôle sélectionné à fond accentué personnalisé, `KitThemes.selected-inset-focus(theme)` utilise la couleur `on-accent` du thème avec la géométrie partagée d’anneau intérieur. `KitThemes.inset-focus(theme)` conserve la couleur de focus habituelle sur les surfaces neutres.

Les indicateurs d’attente des boutons suivent `theme.motion.fast` et restent fixes si `theme.reduced-motion` vaut true. Donnez à `busy-label` une explication localisée du travail en cours ; elle devient la description accessible du bouton occupé.

Les boutons acceptent `ButtonIntent.success`, `.warning` et `.information`. Chaque palette comprend repos/survol/appui et les encres indépendantes `on-success`, `on-warning`, `on-information`. Les actions destructives lisent `on-danger`, sans hériter de `on-accent`. Fournissez chaque champ de palette dans un littéral `KitTheme` complet ; copier un thème livré puis modifier ses champs évite les valeurs nulles implicites.

`ButtonIntent.link` est une action textuelle avec encre accentuée, soulignement et fonds hover/pressed/disabled transparents. La sémantique et l’activation restent celles d’un bouton : votre handler activated décide de toute navigation. Slint1.18.1 ne fournit pas de rôle accessible link distinct. `ButtonRecipe.underline` est un champ de décoration de recette ; les recettes personnalisées complètes doivent le définir.

`KitTheme.geometry` possède les tailles d’icônes/choix, épaisseurs/marges/limites des barres, plafonds de contenu, indicateurs occupés, taille d’avatar, marqueurs d’étape et bornes de dépôt. Les valeurs conservent la présentation par défaut et deviennent personnalisables. `KitIcon`, `KitScrollBar`, `KitSurface` et `KitCard` exposent des entrées `recipe` complètes ; les cartes ajoutent `level`, les conteneurs `max-content-width`. Partez de KitThemes puis modifiez les champs voulus. L’ombre principale suit la recette, sauf si une élévation explicite choisit une autre ombre du thème.

Lors de la migration d’un littéral de thème complet, ajoutez chaque champ `KitGeometry` dans les deux modes. `KitThemes.choice-box-size(theme, size)` et `KitLayoutMetrics.width-limit(theme, value)` prennent désormais le thème explicitement pour respecter les thèmes locaux.

`KitSurface` et `KitCard` conservent la peinture de leur Rectangle hérité. Leur `SurfaceRecipe` applique le fond, la bordure, le rayon et l’élévation, puis seulement `shadow.offset-y`, `shadow.blur` et `shadow.color`. Ces deux composants ne peignent ni `ambient-shadow` ni `shadow.spread`. Une valeur explicite de `elevation` différente de `recipe.elevation` choisit l’ombre principale dans le thème. `KitDialogPanel` utilise la composition élevée qui peint les deux couches d’ombre et leurs spreads.

La `size` explicite de l’avatar est exacte et les initiales suivent son diamètre. Les préréglages `AvatarSize.compact/medium/large` suivent le texte ; leurs tailles et le ratio des initiales appartiennent à geometry. Un avatar vide lit theme.icons.avatar avec un nom de repli localisable. Les décorations de statut utilisent @children.

Le `tone` de l’avatar choisit une palette neutre/accent/statut, avec accent par défaut. Les tons de statut combinent fonds subtils et encres lisibles.


`KitSteps.orientation` propose des dispositions verticales et horizontales, avec liaisons facultatives et descriptions par étape. Activez explicitement l’interaction et nommez le groupe. `KitTimeline` utilise le ton sémantique de la ligne pour son repère ou glyphe décoratif ; le ton par défaut est neutre. Les textes doivent porter le sens indépendamment des couleurs ou icônes.


KitSteps conserve spacing comme entrée publique ; son conteneur Rectangle utilise des dispositions horizontale/verticale internes. Déplacez l’ancien padding hérité vers une disposition parente.


## Présentation de droite à gauche

Définissez `theme.direction: ChoiceFlow.right-to-left` pour les icônes leading/trailing, les badges, les gouttières de défilement et les côtés start/end du chat. Les repères d’étapes suivent cette direction. Les choix en héritent comme `flow` par défaut, toujours remplaçable localement. Les thèmes fournis restent de gauche à droite ; les littéraux complets de `KitTheme` doivent inclure direction.

Slint 1.18.1 ne propose pas de direction générale ni de reflet automatique des dispositions. Ce réglage reflète les éléments positionnés par le kit ; il ne réécrit pas les dispositions de vos `@children`, n’inverse pas les données, ne modifie pas les coordonnées du défilement horizontal et ne transforme pas les glyphes fournis. Disposez les enfants explicitement et choisissez des icônes directionnelles adaptées. La composition, la sélection, le curseur et le comportement bidirectionnel des textes natifs restent ceux de Slint ; qualifiez la saisie arabe/hébraïque et les technologies d’assistance sur le backend cible.

La direction du thème n’ajoute pas de service général de reflet Slint : textes et parcours de focus natifs conservent le comportement du moteur, les dispositions enfants restent explicites et les coordonnées de défilement gardent la convention contenu positif/décalage négatif documentée.


`ScrollFrame` est une primitive publique `@kit` pour composer un viewport fini à deux axes. Fournissez `content-width` et `content-height` ; les enfants gardent leur édition native. Observez `content-x/y` négatifs, `visible-width/height` et l’origine physique relative `viewport-x` pour aligner les en-têtes externes. Elle fournit gouttières, politiques et `scrolled`, sans rôle clavier de viewport. `enabled` contrôle barres et déplacement direct ; la molette native reste disponible. `KitScrollArea` reste le conteneur vertical complet. Les en-têtes et la révélation des cellules du tableau RTL utilisent cette même origine mesurée.

`KitButtonGroup` nomme les commandes apparentées, avec orientation et retour à la ligne facultatif. `attached: true` présente les actions du modèle dans une bande sans retour à la ligne, avec contour partagé, séparateurs et focus intérieur ; spacing/wrap sont ignorés. Les enfants personnalisés gardent leur style explicite. Le conteneur utilise désormais un Rectangle et un FlexboxLayout interne ; spacing/alignment restent publics, tandis que l’ancien padding hérité de HorizontalLayout doit passer à la disposition parente.

`KitColorWheel` lit son minimum, son diamètre préféré et les dimensions du marqueur dans `KitTheme.geometry.color-wheel-*`, multipliés par `text-scale`. Les littéraux complets doivent inclure `color-wheel-minimum`, `color-wheel-diameter`, `color-wheel-indicator-size` et `color-wheel-indicator-border-width`. Le spectre représente les données couleur et ne dépend pas du thème. Les remplissages unis des sliders conservent leurs coins arrondis sur le moteur logiciel ; les dégradés gardent leur espace de remplissage partiel et sa limite de clipping amont documentée.

KitButton conserve par défaut son libellé monoligne élidé. Définissez `text-wrap: word-wrap` (ou `char-wrap`) pour une action multiligne, puis bornez la largeur avec la disposition ou `control-width: ControlWidth.fill`. Gardez une hauteur intrinsèque pour que le texte agrandi et les sauts de ligne explicites puissent agrandir l’action. Les icônes restent centrées de chaque côté du bloc de texte ; les états occupé et désactivé conservent cette géométrie. Toutes les lignes activent la même action et gardent un seul nom accessible.

Utilisez `KitAvatarGroup` pour un nombre borné de membres superposés. Fournissez `AvatarEntry` via `KeyedModel<K, AvatarEntry>`, résolvez jetons/générations avec `accept` et traitez l’action du compteur séparément. Réglez `max-visible`, `overlap` et la taille ; seules les entrées visibles sont montées. `KitStrings` nomme le groupe/compteur et les surcharges d’instance permettent un format personnalisé.

Composez les icônes intrinsèques dans un conteneur de la hauteur de ligne et centrez-les dans ce conteneur ; fixer leur taille ne centre pas leur cellule. Les chips de multi-select et tag-input mesurent leur hauteur de retour à la largeur disponible, en conservant les éditeurs natifs lorsque les modèles changent. Menu-bar et breadcrumbs réservent une gouttière stable de défilement horizontal en plus de leur hauteur de ligne complète. Les étapes indisponibles gardent une action transparente et le texte désactivé du thème ; les lignes de choix conservent leur propre fond désactivé. Une vue de texte virtuelle devrait remplir un cadre parent qui remplit l’espace disponible ; n’imposez un viewport fini explicite que pour obtenir cette taille fixe.
