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

Un thème personnalisé fournit canevas, surfaces normale et élevée ; textes primaire, secondaire et indicatif ; accent, texte sur accent, accent atténué et sélection de texte ; danger, succès, avertissement et information avec leurs fonds atténués ; bordures légère et marquée, survol, pression et désactivation ; voile modal ; `reduced-motion` et `text-scale`. Au-delà des couleurs, un thème porte des groupes de jetons typés : `shape` (rayons par rôle pour les contrôles, conteneurs, surcouches et pastilles, et épaisseurs de bordure), `elevation` (un niveau par rôle et une couche d’ombre par niveau), `focus` (style, épaisseur, décalage et couleurs de l’anneau), `typography` (famille, graisses, tailles), `spacing` (une unité de base, et la hauteur et les marges de chaque `ControlSize`) et `motion` (durées rapide, de base et lente). Slint n’a pas de syntaxe de décomposition et un champ de struct omis vaut silencieusement zéro ou transparent : écrivez donc un thème personnalisé en littéral complet, ou copiez un thème livré et modifiez ses champs depuis Rust. Les [déclarations publiques du thème](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/ui/theme.slint) définissent ce contrat.

Le `theme` de chaque composant vaut par défaut `KitThemeContext.current`, initialisé à `KitThemes.light`. Affectez-le une fois pour thémer toute une fenêtre, y compris les composants qui n’ont pas reçu de thème ; la propriété `theme` d’un composant le remplace toujours pour son sous-arbre. Pour le changer depuis Rust, réexportez le global depuis le fichier principal de l’application avec `export { KitThemeContext } from "@kit/kit.slint";`, puis appelez `KitThemeContext::get(&window).set_current(theme)`.

`ButtonIntent` propose `primary`, `secondary`, `quiet` et `destructive`. Le retour utilisateur utilise des enums de sévérité et de ton. Choisissez l’état selon son sens, puis laissez le thème fournir la couleur. Accompagnez une bordure d’erreur d’un message et d’un libellé accessibles.

## Taille et mise en page

`ControlSize` vaut `compact`, `medium` ou `comfortable`. `ControlWidth.intrinsic` conserve la largeur du contenu ; `fill` utilise l’espace attribué par la mise en page. La hauteur est généralement intrinsèque. Donnez un viewport fini aux [composants de données virtualisées](/fr/components/#data) et conservez l’identité du modèle pendant le défilement.

`LayoutSpace` fournit les valeurs partagées de marges internes et d’espacement, notamment pour `KitCard` et `KitHero`. Utilisez [KitStack](/fr/components/stack/), [KitInline](/fr/components/inline/) et les autres conteneurs lorsque leur contrat d’espacement convient à l’application.

## Agrandissement du texte et mouvement

Pour un texte à 200 %, copiez un thème et réglez `text-scale` sur `2`. Pour réduire le mouvement, réglez `reduced-motion` sur `true`. Ce sont des choix du thème applicatif ; l’hôte doit les relier aux préférences réelles du système si nécessaire. Laissez grandir le texte et les contrôles plutôt que de les couper dans un contenu de hauteur fixe.

## Vérifier l’expérience native

Examinez les thèmes clair et sombre, le texte à 200 %, la navigation clavier, le focus visible et les actions désactivées dans l’application réelle. Préservez l’éditeur natif pour curseur, sélection, annulation et IME. Donnez un nom accessible pertinent aux boutons d’icône, surfaces pressables et cibles de focus.

Les captures de ce site sont des rendus Slint natifs utilisant **winit et Skia sur macOS**. Elles montrent l’état déclaré par l’exemple avec des libellés adaptés à la langue de la page. Elles ne qualifient ni lecteur d’écran, ni IME, ni dépôt de fichiers externe, ni autre backend. La qualification des périphériques et technologies d’assistance relève des plateformes ciblées par l’application. La [documentation d’accessibilité de Slint](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) décrit les propriétés et les outils d’inspection native.
