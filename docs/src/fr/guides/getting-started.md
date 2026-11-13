---
layout: base.njk
locale: fr
title: "Bien démarrer"
translationKey: "guides/getting-started/"
permalink: "{{ 'guides/getting-started/' | localeUrl(locale) }}index.html"
---

# Bien démarrer

Slint Kit associe composants visuels et comportements partagés : validation des brouillons, valeurs Rust typées et identités stables. Le [guide Pourquoi Slint Kit](/fr/guides/why-slint-kit/) présente les responsabilités du kit et de votre application, avec des exemples à essayer.


Installez Slint Kit depuis crates.io avec **Rust {{ project.rust }}+** et **Slint {{ project.slint }}**. Aucun clonage du kit n’est nécessaire.

Les applications utilisent l’édition Rust 2024. Le compilateur et le runtime Slint doivent correspondre aux versions du kit.

## Créer une application de bureau

Suivez une fois la [configuration du projet](/fr/guides/project-setup/) pour Cargo et `build.rs`. Utilisez l’import public racine `@kit` et appelez `slint_kit::install(&app)` avant d’afficher chaque fenêtre. Le kit fournit le comportement, le thème et les conversions des contrats typés ; les callbacks de l’application traitent les actions métier.

Cette documentation cible Slint Kit **{{ project.version }}**. Utilisez des versions correspondantes des paquets runtime et build. Configurer le projet décrit aussi les dépendances sources locales. Une application utilisant une ancienne version suit la documentation correspondante.

## Utiliser un exemple documenté

Pour une application fonctionnelle, commencez par le [bouton](/fr/components/button/) ou la [liste avec recherche](/fr/components/combobox/) : chaque fiche montre le composant Slint et ses callbacks métier, avec un lien vers la configuration commune.

Chaque fiche propose un projet complet téléchargeable avec `Cargo.lock`, Rust, Slint et `build.rs`. Décompressez le ZIP, puis lancez `cargo run --locked`. La capture et la preview interactive utilisent ce même exemple. Pour les règles communes des champs, consultez les [formulaires et validations](/fr/guides/forms/).

Les captures montrent l’état déclaré dans l’exemple. Les actions système comme l’ouverture d’un sélecteur de fichiers nécessitent l’[adaptateur natif](/fr/guides/native/). Pour une édition validée et une sélection interactive, consultez l’[application avec contrôleurs typés](/fr/guides/typed-example/).

Importez composants, structs, enums et globals depuis l’unique point d’entrée `@kit`. Les fichiers supplémentaires de polices et d’icônes Lucide gardent leurs chemins documentés. Évitez les composants auxiliaires privés. Transmettez le même thème aux contrôles composés, préservez la taille intrinsèque et choisissez `ControlWidth.fill` quand le parent demande un contrôle étiré.

## Garder le domaine en Rust

Une chaîne Slint est un brouillon éditable. Pour les noms acceptés, montants, dates et choix identifiés par clé, utilisez des newtypes, des enums et les [liaisons typées](/fr/guides/controllers/). Les grandes collections utilisent un modèle et une vue, plutôt qu’une liste de contrôles construite en entier.

## Compiler le showcase

Le showcase est une application fonctionnelle qui présente les composants publics. Compilez-le depuis le dépôt source avec Git et **Rust {{ project.rust }}+**.

Préparez les dépendances de bureau de votre système : outils de ligne de commande Xcode sur macOS, chaîne Rust MSVC et outils de compilation Visual Studio sur Windows, ou bibliothèques de développement requises sur Linux. Le [guide de compilation de Slint {{ project.slint }}](https://github.com/slint-ui/slint/blob/v{{ project.slint }}/docs/building.md#prerequisites) détaille la configuration par plateforme.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

## Cibles annoncées et cibles qualifiées

Slint Kit annonce les plateformes de bureau macOS, Windows et Linux, avec le backend winit et le moteur de rendu Skia de Slint comme pile documentée. Les sélecteurs natifs de fichiers et de répertoires compilent sur macOS, Windows, Linux et FreeBSD et signalent `Unsupported` ailleurs. Android, iOS, le web et les autres backends et moteurs de rendu de Slint ne sont pas des cibles annoncées ; les previews navigateur de ce site compilent le kit en WebAssembly à titre d’illustration seulement.

Une cible annoncée est une cible pour laquelle le kit est construit. Une cible qualifiée est une cible qui a une preuve nommée : une vérification exécutée sur un système, un backend et un moteur de rendu nommés, dont le résultat se lit. Le tableau liste chaque preuve que le kit publie. Les preuves d’intégration continue s’exécutent à chaque push sur `main` et à chaque pull request ; leur résultat courant est la dernière exécution du [workflow Check](https://github.com/damione1/slint-kit/actions/workflows/check.yml), pas une affirmation de cette page. Chaque résultat ci-dessous est une exécution passée enregistrée, pas l’état courant.

| Cible | Backend et moteur de rendu | Ce que la preuve vérifie | Où |
| --- | --- | --- | --- |
| macOS, image GitHub `macos-26-arm64` (exécution passée ; la CI ne tourne plus sur macOS) | Backend de test Slint avec le moteur de rendu logiciel ; aucune fenêtre | Vérification complète `xtask check` (dont 314 tests de composants et 42 du showcase), starters téléchargeables et applications complètes de documentation réussis sur `6f553989` le 2026-11-10 | [Job `desktop` réussi](https://github.com/damione1/slint-kit/actions/runs/42776551241/job/129696271661) |
| Linux, runner GitHub `ubuntu-24.04` (x86_64) | Backend de test Slint avec le moteur de rendu logiciel ; aucune fenêtre | Le workflow Check exécute désormais ici le `xtask check` complet. Dernier résultat enregistré : tests headless des composants, tests du reste du workspace et clippy avec avertissements refusés réussis sur `59ece023` le 2026-11-10 | [Job `linux` réussi](https://github.com/damione1/slint-kit/actions/runs/42769690387/job/129674322933) |
| Windows, runner GitHub `windows-2025` (x86_64) | Aucun : compilation seulement | `cargo check` compile le kit et sa crate de tests ; les tests de chemins du helper de build s’exécutent. Aucun test de composant ne s’exécute et aucune fenêtre ne s’ouvre | Job `windows` du workflow Check |
| macOS arm64, poste du mainteneur (Darwin 27, Apple M4 Pro, facteurs d’échelle 1 et 2) | Backend winit avec le moteur de rendu Skia ; fenêtre native | Captures des composants rendues nativement en thèmes clair et sombre et à 200 % de texte ; clic pointeur et parcours Tab/Espace du bouton vérifiés dans des fenêtres natives neuves, en français et en anglais | Légendes de chaque fiche de composant ; `docs/content/captures.json` enregistre l’environnement et la portée de la qualification |

Non qualifié sur toute cible (les procédures pour le faire sont dans la section suivante) : interaction pointeur et clavier dans une vraie fenêtre winit sur Windows et Linux (X11 et Wayland), composition IME sur tout système, lecteurs d’écran (VoiceOver, Narrateur, NVDA, Orca), moteurs de rendu autres que Skia sur la machine de capture, tactile, mobile et web. Un test headless prouve le contrat du composant sur le backend de test ; une capture prouve le rendu d’un état déclaré dans l’environnement de capture. Ni l’un ni l’autre n’est une qualification native, IME ou lecteur d’écran. Une telle qualification est un passage manuel sur le système, le backend, le moteur de rendu et la technologie d’assistance réels, rapporté avec ces noms ; le changelog indique quelles fonctions en attendent encore une. Quand vous qualifiez une cible pour votre application, consignez les mêmes noms.

### Qualifier l’IME et les lecteurs d’écran

La composition IME et les lecteurs d’écran ont chacun leur procédure manuelle. Aucun job de CI, test headless ni capture ne les remplace. Exécutez-en une sur le système, le backend, le moteur de rendu et l’appareil réels, puis publiez le résultat avec ces noms et la version ou le commit du kit.

**Composition IME.** Utilisez un contrôle de texte (`KitTextField`, `KitTextArea`, `KitMaskedField`) dans une vraie fenêtre winit, avec la méthode de saisie de la plateforme : par exemple le japonais ou le chinois sur macOS et Windows, et IBus ou Fcitx sur Linux sous X11 et Wayland.

1. Donnez le focus au contrôle et démarrez une composition. Vérifiez que le texte en cours s’affiche et que la fenêtre de candidats s’ouvre près du curseur.
2. Validez un candidat. Vérifiez que le champ contient le texte validé une seule fois et déclenche une seule fois son rappel d’acceptation.
3. Appuyez sur Entrée et sur Échap pendant la composition. Vérifiez qu’elles agissent sur la composition et que ni l’une ni l’autre ne soumet le formulaire ni n’annule le brouillon.
4. Composez dans un contrôle multiligne, dans un contrôle masqué et dans un contrôle avec `maxlength`. Notez quelles contraintes s’appliquent au texte validé.

**Lecteurs d’écran.** Testez VoiceOver sur macOS, Narrator et NVDA sur Windows, et Orca sur Linux. Utilisez le même écran pour chaque lecteur.

1. Parcourez l’écran avec Tab. Vérifiez que chaque contrôle annonce son nom, son rôle, son état (coché, déplié, invalide, lecture seule, désactivé) et sa description.
2. Modifiez un champ. Vérifiez que la nouvelle valeur, le message de validation et l’état de progression sont annoncés une seule fois.
3. Ouvrez et fermez un dialogue, une fenêtre contextuelle et un menu. Vérifiez que le focus y entre et revient à l’élément d’ouverture.
4. Utilisez les actions propres au lecteur (par exemple VoiceOver `VO+Espace`) pour activer des boutons et régler des curseurs.

**Compte rendu.** Pour chaque passe, écrivez la cible (système et version, backend, moteur de rendu, appareil ou lecteur et version), les composants testés, la date, le commit, et chaque étape comme réussie, échouée ou non testée. Liez un ticket pour chaque échec. Une étape non exécutée reste « non testée » ; ne la déduisez pas d’une autre cible.

Le code et les ressources incluses sont sous [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE) ; conservez les notices des dépendances.

Les aperçus de l’éditeur affichent les déclarations statiques sans exécuter les services Rust. Consultez les [aperçus statiques et la limite du live-preview de Slint 1.18.1](../editor/#apercus-statiques-et-services-rust) avant d’activer `SLINT_LIVE_PREVIEW`.
