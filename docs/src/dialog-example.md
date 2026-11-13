---
layout: base.njk
pagination:
  data: locales
  size: 1
  addAllPagesToCollections: true
  alias: locale
permalink: "{{ 'guides/dialog-example/' | localeUrl(locale) }}index.html"
translationKey: "guides/dialog-example/"
eleventyComputed:
  title: "{% if locale == 'fr' %}Dialogues et retour du focus{% else %}Dialogs and focus return{% endif %}"
---
{% if locale == 'fr' %}
# Composer des dialogues avec retour du focus

Cette application Rust et Slint complète compose deux couches modales à l’intérieur d’une fenêtre avec `KitDialogSurface`, `KitDialogPanel`, `KitDialogHeader`, `KitDialogBody`, `KitDialogFooter` et `KitFocusTarget`. L’adaptateur `DialogHost` possède la pile de couches (`OverlayStack`) et le registre de focus (`FocusRegistry`) ; Slint affiche l’état publié et relaie les gestes. Ce sont des surfaces composées dans la fenêtre de l’application : le kit ne fournit pas de fenêtre modale système. Les actions de KitDialogFooter passent à la ligne quand la largeur disponible ou le texte agrandi l’exige.

Ce que l’exemple démontre :

- **Plusieurs déclencheurs, retour exact.** Deux boutons ouvrent le même dialogue. À la fermeture, le focus revient au bouton qui l’a ouvert, pas à un emplacement par défaut.
- **Couches imbriquées.** « Archiver… » ouvre une confirmation au-dessus du dialogue. Échap et Retour système ferment une couche à la fois ; chaque couche rend le focus à son propre déclencheur.
- **Tab et Maj+Tab** restent dans la couche du dessus et bouclent, en ignorant les contrôles désactivés. Un champ natif est une cible comme une autre.
- **État occupé.** Pendant l’archivage, Échap, Retour système et le voile sont consommés sans fermer ; le bouton d’action affiche l’état occupé et garde le focus dans la couche. À la fin, les deux couches se ferment et le focus revient au premier déclencheur.
- **Arrière-plan neutralisé, pas seulement les clics.** Les contrôles couverts sont inéligibles dans le registre et désactivés dans Slint à partir du même indicateur : pointeur, clavier et action d’accessibilité sont neutres. Le voile seul ne suffit pas, car il n’arrête pas les actions sémantiques.

## Essayer l’application

Créez un projet neuf, puis remplacez ses fichiers par les cinq fichiers complets ci-dessous. {% if project.registryExamples %}Les paquets viennent de crates.io ; aucun checkout du kit n’est nécessaire.{% else %}Cette édition suit les sources de développement et utilise des API ajoutées après 0.7.2. Les versions 0.7.2 publiées ne compilent pas les sources ci-dessous : utilisez un checkout correspondant du kit et remplacez les dépendances `slint-kit` et `slint-kit-build` du manifeste par des dépendances `path` vers `crates/kit` et `crates/kit-build` de ce checkout. Pour un exemple disponible depuis crates.io, utilisez [les sources immuables du tableau 0.7.2](https://github.com/damione1/slint-kit/tree/v0.7.2/docs/table-consumer), épinglées sur cette version publiée.{% endif %} Les [prérequis natifs](/fr/guides/getting-started/) sont ceux du guide de démarrage.
{% else %}
# Compose dialogs with focus return

This complete Rust and Slint application composes two modal layers inside one window with `KitDialogSurface`, `KitDialogPanel`, `KitDialogHeader`, `KitDialogBody`, `KitDialogFooter` and `KitFocusTarget`. The `DialogHost` adapter owns the layer stack (`OverlayStack`) and focus registry (`FocusRegistry`); Slint shows the published state and relays gestures. These are surfaces composed in the application window: the kit does not provide a system modal window. KitDialogFooter actions wrap when the available width or enlarged text requires it.

What the example demonstrates:

- **Several triggers, exact return.** Two buttons open the same dialog. On close, focus returns to the button that opened it, not to a default location.
- **Nested layers.** “Archive…” opens a confirmation above the dialog. Escape and system Back close one layer at a time; each layer returns focus to its own trigger.
- **Tab and Shift+Tab** stay inside the topmost layer and wrap, skipping disabled controls. A native text field is a target like any other.
- **Busy state.** While archiving runs, Escape, system Back and the backdrop are consumed without closing; the action button shows the busy state and keeps focus inside the layer. On completion both layers close and focus returns to the first trigger.
- **Background neutralized, not only clicks.** Covered controls are ineligible in the registry and disabled in Slint from the same flag: pointer, keyboard and accessibility activation are all neutral. The backdrop alone is not enough, because it does not stop semantic actions.

## Try the application

Create a fresh project, then replace its files with the five complete files below. {% if project.registryExamples %}Packages come from crates.io; no kit checkout is needed.{% else %}This edition tracks development sources and uses APIs added after 0.7.2. Published 0.7.2 packages cannot compile the sources below: use a matching kit checkout and replace the manifest’s `slint-kit` and `slint-kit-build` dependencies with `path` dependencies pointing to that checkout’s `crates/kit` and `crates/kit-build`. For a registry-ready example, use [the immutable 0.7.2 table sources](https://github.com/damione1/slint-kit/tree/v0.7.2/docs/table-consumer), pinned to that published release.{% endif %} The [native prerequisites](/guides/getting-started/) match the getting-started guide.
{% endif %}

```sh
cargo new slint-kit-dialog-example
cd slint-kit-dialog-example
```

{% if locale == 'fr' %}
1. Activez **Archiver depuis la fiche du projet**. Le champ « Nom du projet » prend le focus. Tab parcourt Annuler puis Archiver…, et revient au champ ; Maj+Tab fait l’inverse.
2. Appuyez sur Échap : le dialogue se ferme et le focus revient à **Archiver depuis la fiche du projet**. Ouvrez-le depuis la barre d’outils : Échap rend le focus à ce bouton-là.
3. Ouvrez le dialogue, puis **Archiver…** : la confirmation s’affiche au-dessus. Tab ne circule qu’entre Conserver le projet et Archiver maintenant. Échap ferme la confirmation seule et rend le focus à **Archiver…** ; un second Échap ferme le dialogue.
4. Dans la confirmation, activez **Archiver maintenant** : le bouton passe en état occupé et Échap ne ferme plus rien. Au bout d’un instant, les deux couches se ferment et le focus revient au déclencheur initial.

## Comment les pièces s’assemblent

- `DialogHost<W, L, T>` reçoit des clés typées via `DialogLayerSpec` (parent, premier contrôle) et `DialogFocusSpec` (jeton stable, couche, accès). Déclarez les parents avant les descendants ; une topologie invalide est refusée à la construction.
- `DialogHostPorts` nomme les setters des trois projections : `FocusSlot`, `DialogState` et `FocusRequest`. Le kit publie l’éligibilité puis les états puis le focus, après avoir relâché son emprunt ; les callbacks de focus peuvent donc rappeler l’hôte.
- Raccordez `open`, `close`, `cycle` et `observe_focus` aux événements. `set_activity(layer, DialogActivity::Busy(target))` garde uniquement l’action occupée éligible. La fermeture imbriquée vérifie tout le sous-arbre avant mutation (un descendant occupé ou bloqué par sa politique refuse atomiquement la fermeture utilisateur du parent, tandis que `Programmatic` et `HostClosed` ferment tout le sous-arbre) et restaure une seule fois le déclencheur exact, ou une cible de repli encore éligible. Pour un hôte personnalisé qui combine dialogues modaux et popovers non modaux, `OverlayStack` conserve l’ordre d’enregistrement du bas vers le haut (`iter`, `get`, `topmost_modal`), relie chaque couche à sa portée optionnelle (`OverlaySpec.focus_scope`) et limite les mutations en place à `set_busy` et `set_policy`. Préférez `OverlaySpec::new` (aucune portée de focus par défaut) ; les littéraux complets exigent désormais `focus_scope`. La pile conserve cette métadonnée ; les hôtes personnalisés poussent/retirent encore les pièges de leur `FocusRegistry` et observent le focus réel. `DialogHost` gère ces pièges pour ses couches modales enregistrées.
- `KitFocusTarget` relaie `focus-requested` au `take-focus()` du contrôle réel et n’est pas un arrêt de focus. Le contrôle réel appelle `observe-focus(focused)` ; `focus-observed(token, generation, focused)` va à `observe_focus`, qui ignore une génération périmée. Liez le même `slot.eligible` à son accès/désactivation : le voile seul ne bloque pas les actions d’accessibilité.

L’éligibilité est calculée une fois, dans Rust, puis publiée ; Slint n’en possède pas de copie. Dans cet exemple, l’action occupée reste éligible pendant le travail pour que la couche conserve le focus et consomme Échap et Retour ; ses voisines sont désactivées.
{% else %}
1. Activate **Archive from the project card**. The “Project name” field takes focus. Tab visits Cancel, then Archive…, then returns to the field; Shift+Tab goes the other way.
2. Press Escape: the dialog closes and focus returns to **Archive from the project card**. Open it from the toolbar: Escape returns focus to that button instead.
3. Open the dialog, then **Archive…**: the confirmation appears above it. Tab only cycles between Keep project and Archive now. Escape closes the confirmation alone and returns focus to **Archive…**; a second Escape closes the dialog.
4. In the confirmation, activate **Archive now**: the button enters the busy state and Escape no longer closes anything. After a moment both layers close and focus returns to the original trigger.

## How the pieces fit

- `DialogHost<W, L, T>` accepts typed identities through `DialogLayerSpec` (parent, first control) and `DialogFocusSpec` (stable token, layer, enabled state). Declare parents before descendants; construction rejects invalid topology.
- `DialogHostPorts` names the setters for three projections: `FocusSlot`, `DialogState` and `FocusRequest`. The kit publishes eligibility, then states, then focus after releasing its borrow, so focus callbacks may safely re-enter the host.
- Connect `open`, `close`, `cycle` and `observe_focus` to view events. `set_activity(layer, DialogActivity::Busy(target))` keeps only the busy action eligible. Nested close checks the entire closing subtree before mutating the stack (a busy or policy-blocked descendant atomically rejects user dismissal of its parent, while `Programmatic` and `HostClosed` close the whole subtree) and restores the exact opener once, or an eligible fallback. For a custom host mixing modal dialogs and non-modal popovers, `OverlayStack` preserves bottom-to-top registration order (`iter`, `get`, `topmost_modal`), links each layer to its optional `OverlaySpec.focus_scope`, and restricts in-place mutation to `set_busy` and `set_policy`. Prefer `OverlaySpec::new` (no focus scope by default); complete struct literals now require `focus_scope`. The stack stores scope metadata; custom hosts still push/pop their `FocusRegistry` traps and observe actual focus. `DialogHost` performs modal trap management for its registered layers.
- `KitFocusTarget` relays `focus-requested` to the real control’s `take-focus()` and is not a focus stop. The real control calls `observe-focus(focused)`; `focus-observed(token, generation, focused)` goes to `observe_focus`, which ignores a stale generation. Bind the same `slot.eligible` to its access/disabled state: a backdrop alone cannot block accessibility actions.

Eligibility is computed once, in Rust, then published; Slint keeps no copy of the rule. In this example the busy action stays eligible during the work so the layer keeps focus and consumes Escape and Back; its neighbours are disabled.
{% endif %}

## Cargo.toml

[{{ ui[locale].download }} — Cargo.toml](/dialog-consumer/Cargo.toml)

```toml
{{ 'Cargo.toml' | dialogSource | safe }}
```

## build.rs

[{{ ui[locale].download }} — build.rs](/dialog-consumer/build.rs)

```rust
{{ 'build.rs' | dialogSource | safe }}
```

## ui/dialog.slint

[{{ ui[locale].download }} — ui/dialog.slint](/dialog-consumer/ui/dialog.slint)

```slint
{{ 'ui/dialog.slint' | dialogSource | safe }}
```

## src/main.rs

[{{ ui[locale].download }} — src/main.rs](/dialog-consumer/src/main.rs)

```rust
{{ 'src/main.rs' | dialogSource | safe }}
```

## src/controller.rs

[{{ ui[locale].download }} — src/controller.rs](/dialog-consumer/src/controller.rs)

```rust
{{ 'src/controller.rs' | dialogSource | safe }}
```

## {% if locale == 'fr' %}Exécuter et adapter{% else %}Run and adapt{% endif %}

```sh
cargo run{% if locale == 'fr' %} -- --fr{% endif %}
```

{% if locale == 'fr' %}
Remplacez la temporisation par votre travail asynchrone réel : marquez la couche `busy` au départ, puis fermez-la par `DismissReason::Programmatic` à l’arrivée du résultat. Ajoutez une couche avec `DialogLayerSpec` et un contrôle avec `DialogFocusSpec` ; utilisez `set_enabled` lorsque son état change. Le compilateur signale chaque `match` à compléter. Avec `tab-policy: DialogTabPolicy.native`, Slint parcourt lui-même les contrôles montés. Réglez alors `modal-layer` sur chaque `KitDialogSurface` (nombres positifs distincts pour des couches imbriquées) et donnez le même `modal-layer` aux contrôles Kit de la couche : les autres contrôles Kit deviennent inertes sans jeton. Les contrôles personnalisés, calendriers, sélecteurs de date, d’heure, de couleur et de fichier, tableaux de données et zones défilantes ne lisent pas encore ce contexte ; les onglets, menus, accordéons, carrousels, fils d’Ariane et la pagination le lisent : l’hôte les désactive comme avant.

Slint {{ project.slint }} ne fournit ni rôle de dialogue, ni état modal, ni propriété permettant de masquer un sous-arbre accessible tout en le gardant visible. Le panneau est annoncé comme `groupbox` avec son nom ; les contrôles couverts restent lisibles dans l’ordre de parcours du lecteur d’écran malgré leur désactivation. Le contrat « arrière-plan neutralisé » concerne les actions, pas l’isolement du parcours accessible.

Les tests du kit prouvent ce contrat sans écran : retour du focus au déclencheur exact, couches imbriquées, Tab/Maj+Tab, Échap/Retour, état occupé et arrière-plan inerte. Ils ne qualifient ni les lecteurs d’écran ni les méthodes de saisie : vérifiez-les sur la plateforme cible. Consultez la [référence KitDialogSurface](/fr/components/dialog-surface/) et la [référence KitFocusTarget](/fr/components/focus-target/) pour l’API complète.
{% else %}
Replace the timer with your real asynchronous work: mark the layer `busy` when it starts, then close it with `DismissReason::Programmatic` when the result arrives. Add a layer with `DialogLayerSpec` and a control with `DialogFocusSpec`; call `set_enabled` when its enabled state changes. The compiler reports every `match` to complete. With `tab-policy: DialogTabPolicy.native`, Slint traverses the mounted controls itself. Then set `modal-layer` on each `KitDialogSurface` (distinct positive numbers for nested layers) and give the layer's kit controls the same `modal-layer`: every other kit control becomes inert without a token. Custom controls, calendars, date, time, color and file pickers, data tables and scroll areas do not read this context yet; tabs, menus, accordions, carousels, breadcrumbs and pagination do: the host disables them as before.

Slint {{ project.slint }} exposes no dialog role, modal state, or property that hides an accessible subtree while keeping it visible. The panel is exposed as a named `groupbox`; covered controls remain readable in the screen reader browse order despite being disabled. The inert-background contract covers actions, not isolation of the accessibility browse tree.

The kit’s tests prove this contract headlessly: focus return to the exact trigger, nested layers, Tab/Shift+Tab, Escape/Back, busy state and an inert background. They do not qualify screen readers or input methods: verify those on the target platform. See the [KitDialogSurface reference](/components/dialog-surface/) and the [KitFocusTarget reference](/components/focus-target/) for the full API.
{% endif %}
