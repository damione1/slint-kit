---
layout: base.njk
locale: fr
title: "Pourquoi Slint Kit"
translationKey: "guides/why-slint-kit/"
permalink: "{{ 'guides/why-slint-kit/' | localeUrl(locale) }}index.html"
---

# Ce que Slint Kit apporte à une application Rust et Slint

**Slint Kit fournit des composants UI et une couche de comportements partagés pour les applications Rust construites avec Slint.** Il associe édition native du texte, acceptation typée des champs, identité stable des collections, valeurs métier exactes et thème personnalisable. Les {{ components.length }} composants publics comprennent des contrôles complets et des briques de composition. Votre application reste propriétaire des règles métier, des données et des commandes.

Choisissez-le pour des formulaires, des valeurs éditables, des choix avec recherche et des vues de données au comportement cohérent. Commencez par l’[installation](/fr/guides/getting-started/), ou examinez le [catalogue](/fr/components/) et les exemples exécutables avant de l’adopter.

## Slint, Slint Kit et votre application

[Slint](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/overview/) fournit le langage déclaratif, le rendu, les backends de fenêtre, l’éditeur natif et les widgets standard. Slint Kit est une bibliothèque indépendante construite sur Slint, distribuée par Cargo et importée avec `@kit`. Ses services Rust et ses composants Slint fonctionnent ensemble ; les démonstrations navigateur exécutent ces mêmes composants et leur code Rust compilés en WebAssembly.

| Besoin | Slint Kit fournit | Votre application définit |
| --- | --- | --- |
| Formulaires validés | Contraintes HTML partagées, brouillons natifs, transitions acceptation/annulation et acceptation par `FieldSpec<T, E>` | Analyseurs et règles métier, persistance et commande de soumission protégée |
| Choix avec recherche | `Choices<K>`, navigation clavier, options désactivées et jetons de sélection stables | Votre type de clé, les choix disponibles et l’action résultante |
| Tableaux et listes | Vues virtualisées et `KeyedModel<K, Row>` avec identité stable et vérifications de génération | Chargement, filtrage et comparateurs de tri |
| Nombres et dates exacts | `Money`, `DecimalValue`, dates civiles et résolution explicite des fuseaux | Devise, précision, disponibilités et politique de fuseau |
| Apparence | Thèmes clair/sombre complets, recettes d’état, dimensions et rôles d’icônes | Identité visuelle, textes localisés et thèmes des enfants composés |
| Fichiers et répertoires | Liaisons natives asynchrones optionnelles distinguant sélection, annulation, échec et absence de support | Traitement des fichiers et prise en compte des capacités de la plateforme cible |

## Des comportements à vérifier dans un exemple

### Un brouillon incomplet reste distinct de la donnée métier

Ouvrez le [champ courriel](/fr/components/email-field/), remplacez l’adresse par `not-an-address` et essayez de l’appliquer. Le moteur de contraintes partagé refuse ce candidat. Un candidat valide passe par `FieldSpec::accept` à la frontière Rust, qui retourne une valeur typée. Caret, sélection, annulation et composition restent dans l’éditeur Slint.

Le [guide des formulaires](/fr/guides/forms/) détaille contraintes intrinsèques, règles métier et soumission protégée. L’[application typée complète](/fr/guides/typed-example/) ajoute un type de nom de projet personnalisé et un choix d’espace de travail par clé. Un callback et un bouton désactivé ne suffisent pas à valider le chemin de soumission d’une application.

### Une ligne triée conserve son identité

Dans l’[application de tableau triable](/fr/guides/table-example/), sélectionnez Atlas, puis triez Tâches. Le projet sélectionné reste Atlas même si sa position change. Avec `KeyedModel`, les clés métier stables survivent au réordonnancement et les générations rejettent un geste commencé sur une ancienne projection. Votre comparateur définit l’ordre ; votre application choisit les données à charger.

### Formatage et rendu préservent les valeurs métier

Le [champ monétaire](/fr/components/money-field/) relie l’édition à des valeurs `Money` exactes et à des règles de devise. Le [champ date, heure et fuseau](/fr/components/zoned-date-time-field/) refuse une heure locale inexistante et exige un choix explicite lorsqu’elle se produit deux fois. Ces règles sont implémentées en Rust et partagées avec l’interaction des composants. Flottants de rendu et libellés formatés restent des projections.

## Personnaliser la présentation et réutiliser les comportements

[KitTheme et les briques de composition publiques](/fr/guides/themes/) séparent apparence et interaction. Couleurs, espacements, géométrie, anneaux de focus et durées viennent du thème. Les champs prêts à l’emploi intègrent libellés et messages de validation ; les briques plus petites permettent des mises en page personnalisées. Transmettez explicitement le thème aux enfants fournis par l’application dans une région thémée.

`slint_kit::install(&window)` installe les services partagés une fois par fenêtre. Chaque champ ou sélecteur conserve son état d’interaction local. Choisissez le mode de gestion contrôlé lorsque l’hôte doit accuser réception d’un changement proposé. Le [guide des contrôleurs](/fr/guides/controllers/) détaille frontières typées et modèles de collection.

## Preuves et limites d’adoption

Les [tests publics de comportement](https://github.com/damione1/slint-kit/tree/main/tests/src) couvrent clavier, pointeur, focus, identité et géométrie. Les [tests de champs](https://github.com/damione1/slint-kit/blob/main/tests/src/autonomous.rs) exercent notamment les acceptations invalides et les instances indépendantes ; les [tests de modèles](https://github.com/damione1/slint-kit/blob/main/tests/src/keyed_model.rs) couvrent réordonnancement et gestes périmés. Les [résultats CI](https://github.com/damione1/slint-kit/actions/workflows/check.yml) exposent les vérifications. Tests sans affichage et captures ne qualifient ni IME, ni lecteur d’écran, ni appareil physique.

L’apparence livrée est un système visuel inspiré de Tailwind, en modes clair et sombre. `KitDataTable` affiche actuellement des cellules texte ; une grille de saisie éditable demande une composition supplémentaire. Les surfaces de dialogue sont des modales internes à leur fenêtre hôte : l’application neutralise les interactions en arrière-plan et raccorde le focus selon la [référence des dialogues](/fr/components/dialog-surface/).

La CI publiée exécute la vérification complète sur macOS et les vérifications de compilation et du helper sur Windows. Elle n’établit pas une qualification équivalente à l’exécution sur Linux ou mobile ; consultez la [configuration CI publique](https://github.com/damione1/slint-kit/blob/main/.github/workflows/check.yml) et vérifiez votre cible.

La série actuelle est la **{{ project.version }}{% if project.preOne %}, avant la 1.0{% endif %}** ; des changements cassants restent possibles. Les pull requests sont suspendues jusqu’à la 1.0 ; les questions et discussions restent ouvertes. Évaluez la capacité de maintenance du projet et votre budget de mises à jour pour une application durable. Utilisez des paquets runtime/build correspondants. Le kit accepte les versions correctives de Slint 1.18 à partir de la 1.18.1 (`~1.18.1`) ; les exemples téléchargeables verrouillent leurs dépendances résolues. Le helper build utilise la compilation expérimentale de modules de Slint ; consultez la [compatibilité du build](https://docs.rs/crate/slint-kit-build/latest) et les [limites des aperçus éditeur](/fr/guides/editor/#apercus-statiques-et-services-rust).

**Gratuits pour toujours pour l’usage open source.** Les composants du kit de base restent gratuits pour les projets open source. Le code et les ressources incluses sont sous [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE), avec conservation des notices des dépendances. Les options de licence de Slint ne changent pas la licence de ce kit. Lors de la distribution d’une application liée à une bibliothèque GPL, la GPL s’applique au programme combiné ; l’usage privé dans une même organisation n’impose pas à lui seul une publication publique. Consultez la FAQ GNU sur les [bibliothèques GPL](https://www.gnu.org/licenses/gpl-faq.en.html#IfLibraryIsGPL) et l’[usage interne](https://www.gnu.org/licenses/gpl-faq.en.html#InternalDistribution). **Une licence commerciale est à venir, sur demande** pour les applications propriétaires : [contactez l’auteur](/fr/contact/) pour discuter de vos besoins et de sa disponibilité. **Slint Kit Pro**, un ensemble distinct de composants avancés, est également à venir. Le [guide des plateformes](/fr/guides/project-setup/#desktop-and-mobile-targets) distingue la configuration bureau et mobile. Les dialogues natifs de fichiers optionnels sont des services de bureau et retournent un résultat non pris en charge sur mobile.

Les widgets standard de Slint restent un point de départ raisonnable lorsqu’ils couvrent vos besoins. Évaluez Slint Kit pour réutiliser la couche de formulaires typés, d’identité et d’interaction décrite ici. Prototypez votre écran critique avec les exemples documentés et qualifiez son comportement sur vos backends et appareils réels.
