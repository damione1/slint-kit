---
layout: base.njk
locale: fr
title: "Changelog"
translationKey: "changelog/"
permalink: "{{ 'changelog/' | localeUrl(locale) }}index.html"
---

# Changelog

## 0.9.0 — Slint 1.19

Slint Kit 0.9.0 passe à Slint 1.19. Utilisez des paquets runtime et build 0.9.0 correspondants ; [Configurer le projet](/fr/guides/project-setup/) fournit le manifeste et les alternatives depuis les sources. Une application qui reste sur une version précédente conserve sa documentation et ses dépendances.

### Intégration de Slint 1.19

- **Outillage :** Rust 1.95+ et Slint 1.19 sont requis. Le kit accepte Slint `~1.19.0` ; les exemples fixent la version 1.19.0. Gardez le compilateur et le runtime alignés.
- **Édition du langage :** le kit possède son propre `slint-project.json` en édition 1.19. Les applications restent en édition 1.18 par défaut ; choisissez explicitement la 1.19 avec `slint_kit_build::config().with_edition((1, 19))`. Les enfants conditionnels et répétés participent désormais à la taille intrinsèque dans le kit, remplaçant les doubles mesures de légendes et de chips.
- **Accessibilité :** les champs texte et numériques exposent un seul propriétaire accessible et délèguent texte et sélection à l’éditeur natif. Cela supprime les nœuds accessibles en double, sans qualifier un lecteur d’écran ou un IME physique.
- **Révélation native :** listes, onglets, menus et champs utilisent les API de défilement de Slint. La restauration des collections reste immédiate, y compris avant le premier placement ; les inversions rapides d’onglet ou de menu gardent le dernier élément actif visible. Les pas clavier des zones de défilement sont immédiats pour conserver toute la distance des répétitions.
- **Réduction des animations :** les blocs d’animation utilisent `animate.enabled` ; le réglage applicatif `theme.reduced-motion` reste disponible. Les plateformes et les responsabilités des timers/ticks sont détaillées dans [thèmes et accessibilité](/fr/guides/themes/#text-scaling-and-motion).

Consultez l’[annonce officielle de Slint 1.19](https://slint.dev/blog/slint-1.19-released) et les [notes de version amont](https://github.com/slint-ui/slint/releases/tag/v1.19.0) pour tous les changements de Slint. Les capacités supplémentaires de Slint sont distinctes des nouveaux composants de Slint Kit.

### Changements de migration

| Intégration précédente | Intégration 0.9.0 |
| --- | --- |
| Sous-imports de composants comme `@kit/button.slint` | Importez le composant public depuis `@kit`. |
| `@kit/icons/lucide/all.slint` ou fichiers de catégories | Utilisez `@kit-icons/all.slint` ou le fichier de catégorie correspondant. |
| `@kit/fonts/inter.slint` | Utilisez `@kit-fonts/inter.slint`. |
| Alias éditeur `kit` pointant vers `kit.slint` | Faites pointer `kit` vers `ui/slint-project.json` ; configurez les alias séparés pour les icônes et polices. Voir [la configuration de l’éditeur](/fr/guides/editor/). |
| `scroll-to-index` aligne toujours une ligne en haut | La fonction révèle le bord le plus proche et laisse une ligne visible en place. Attendez la fin d’une révélation animée avant de lire les décalages finaux ; `set-scroll` restaure immédiatement les décalages signés. Voir [les contrôleurs](/fr/guides/controllers/). |

Les littéraux de thèmes personnalisés doivent inclure les nouvelles couleurs de contrôles et de sélection, la surface en retrait et les jetons de typographie display. Consultez [thèmes et accessibilité](/fr/guides/themes/) et la liste complète ci-dessous avant de migrer un thème personnalisé ou un `match` exhaustif d’enum.

### Créer une application

Le [guide de démarrage](/fr/guides/getting-started/) explique la répartition entre interface Slint et application Rust. La [configuration du projet](/fr/guides/project-setup/#desktop-and-mobile-targets) renvoie désormais vers les guides officiels de packaging desktop, Android, iOS, WebAssembly et de déploiement embarqué. Le support des microcontrôleurs dans Slint ne signifie pas que Slint Kit possède un portage `no_std`.

Les 139 composants publics disposent désormais d’un starter exécutable complet et d’un aperçu navigateur, dont la navigation entre écrans, la barre latérale, les popovers, menus et dialogues.

Pour tous les changements d’API et de comportement du kit, au-delà de la mise à jour de Slint et dans les versions précédentes, consultez le [changelog complet](https://github.com/damione1/slint-kit/blob/v0.9.0/CHANGELOG.md).
