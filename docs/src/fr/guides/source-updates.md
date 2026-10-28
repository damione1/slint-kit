---
layout: base.njk
locale: fr
title: "Nouveautés et compatibilité"
translationKey: "guides/source-updates/"
permalink: "{{ 'guides/source-updates/' | localeUrl(locale) }}index.html"
---

# Mises à jour du kit

La référence des composants, les exemples Slint téléchargeables et les captures natives suivent la branche `main` du dépôt. Les ajouts ci-dessous sont des **changements des sources après la version crates.io 0.1.0**. Le numéro du paquet reste 0.1.0 jusqu’à une nouvelle publication. L’[application de démarrage](/fr/guides/getting-started/), l’[application typée](/fr/guides/typed-example/) et le [tableau triable](/fr/guides/table-example/) conservent des dépendances compatibles avec le registre.

## Utiliser les sources actuelles

Clonez le dépôt à côté de votre application, puis remplacez les deux dépendances du kit par des chemins correspondants. Gardez Slint en version exacte 1.18.1 et Rust 1.92 ou ultérieur.

```sh
git clone https://github.com/damione1/slint-kit.git
```

```toml
[dependencies]
slint-kit = { path = "../slint-kit/crates/kit" }
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = { path = "../slint-kit/crates/kit-build" }
```

Adaptez les chemins à vos dossiers ; conservez le [même build.rs et le point d’entrée](/fr/guides/getting-started/). Notez `git -C ../slint-kit rev-parse HEAD` avec la révision testée de votre application. Les ajouts de cette page nécessitent les sources ; la dépendance `"0.1"` seule ne les sélectionne pas.

## Traduire les dates et les états vides

| Composant | Textes fournis par l’hôte |
| --- | --- |
| [KitCalendar](/fr/components/calendar/) | `previous-month-label`, `next-month-label` ; continuez à fournir le titre, les jours de semaine et les noms des jours dans le modèle. |
| [KitDatePicker](/fr/components/date-picker/) | `placeholder`, `description`, `calendar-button-label`, `calendar-accessible-name` et les deux libellés de navigation mensuelle. Une description vide masque l’aide. |
| [KitDateTimeField](/fr/components/date-time-field/) | `date-label`, `time-label`, `date-placeholder`, `date-description`, `time-description`. |
| [KitZonedDateTimeField](/fr/components/zoned-date-time-field/) | Les textes de date et d’heure ci-dessus, puis `zone-label`, `zone-placeholder`, `zone-description`, `earlier-label`, `later-label`. |
| [KitDataTable](/fr/components/data-table/), [KitContentList](/fr/components/content-list/) | `empty-label`, affiché quand le modèle ne contient aucune ligne ou aucun élément. |

Ces textes changent la présentation et les noms accessibles. Ils ne modifient ni l’analyse des dates, ni la résolution IANA, ni les valeurs acceptées par l’hôte. Par exemple, le placeholder français `AAAA-MM-JJ` décrit toujours l’ordre ISO année-mois-jour existant. Traduisez l’aide de [KitDateField](/fr/components/date-field/) et de [KitTimeField](/fr/components/time-field/) avec leur propriété héritée `description` déjà disponible.

## Observer le focus des éditeurs composés

`KitDateTimeField` et `KitZonedDateTimeField` exposent `editor-focused` et `focus-observed(bool)` pour leurs éditeurs natifs. L’hôte peut utiliser ces observations dans son registre de focus. Ce ne sont pas des demandes de validation ; un passage entre éditeurs peut produire des observations.

`KitZonedDateTimeField.ambiguity-started()` signale le début d’une action de choix d’occurrence avant le callback terminé `ambiguity-chosen(AmbiguousTimeChoice)`. Préservez le contexte de résolution pendant les changements de focus ; acceptez une occurrence seulement après le choix terminé. Conservez les trois brouillons et la génération de résolution ensemble dans l’hôte.

## Choisir le parcours des modales

[KitDialogSurface](/fr/components/dialog-surface/) expose désormais `DialogTabPolicy`. La politique `.host`, par défaut, capture Tab/Maj+Tab et appelle `focus-next(bool)` pour le registre de l’hôte. La politique `.native` laisse Slint parcourir les contrôles montés.

Avec le parcours natif, l’hôte de fenêtre doit rendre **toutes les cibles de focus d’arrière-plan inéligibles** pendant la modale. Utilisez `FieldAccess.disabled` pour les champs, `disabled: true` pour les boutons, et `interactive: false` pour [KitMenu](/fr/components/menu/) et `KitDataTable`, selon le composant. Restaurez les droits et le déclencheur exact à la fermeture. La politique native seule n’établit pas cette limite.

`KitMenu.has-focus` et `focus-observed(bool)` incluent une ligne recevant le focus à l’appui du pointeur, avant son activation. Préservez ce focus pendant l’appui pour permettre au clic de se terminer.

Seul le dialogue au premier plan traite Échap ou Retour système (`Key.Back`) ; un dialogue occupé consomme ces touches sans se fermer. Un sélecteur ouvert demande `CloseReason.system-back` via [KitSelectKeyboard](/fr/components/select-keyboard/). Sa propriété `dismiss-on-system-back` vaut `true` par défaut ; les menus persistants, barres de menus et onglets la désactivent pour laisser l’overlay contenant recevoir Retour. La gestion de la touche est implémentée ; la qualification sur appareil Android reste une vérification distincte.

## Laisser les layouts mesurer le contenu

Les boutons étirés acceptent leur largeur allouée même avec un long libellé. Les actions d’effacement et de visibilité des champs confortables incluent le padding dans leur cible ; un appui sur le padding du champ place le focus dans l’éditeur existant. Le parcours clavier révèle le contour complet du bouton dans les zones de défilement ancêtres.

[KitFieldLabel](/fr/components/field-label/) propage sa hauteur repliée dans les layouts, y compris à 200 % de texte. [KitAlert](/fr/components/alert/) mesure son texte à la largeur allouée dans les layouts conditionnels. Évitez les hauteurs fixes autour de ces composants.

Pour une cible plus haute que le viewport, [KitScrollArea.reveal(top, bottom)](/fr/components/scroll-area/) conserve la position enregistrée si le bord supérieur est déjà visible ; sinon, il révèle ce bord. Les restaurations répétées du focus restent stables.

Homeward est la première preuve d’intégration concrète. Les défauts réutilisables confirmés du kit se corrigent dans ce dépôt avec une régression ; le transport applicatif, l’identité d’installation et l’autorisation serveur restent dans l’hôte. Conservez le contrôle officiel `cargo run -p xtask --locked -- check`, puis inspectez les rendus natifs clair/sombre, le texte agrandi et les interactions réelles. Les captures et tests sans affichage ne qualifient ni l’IME, ni un lecteur d’écran, ni toutes les plateformes.
