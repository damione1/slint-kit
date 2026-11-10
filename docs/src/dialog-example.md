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

Cette application Rust et Slint complète compose deux couches modales à l’intérieur d’une fenêtre avec `KitDialogSurface`, `KitDialogPanel`, `KitDialogHeader`, `KitDialogBody`, `KitDialogFooter` et `KitFocusTarget`. Rust possède la pile de couches (`OverlayStack`) et le registre de focus (`FocusRegistry`) ; Slint affiche l’état publié et relaie les gestes. Ce sont des surfaces composées dans la fenêtre de l’application : le kit ne fournit pas de fenêtre modale système.

Ce que l’exemple démontre :

- **Plusieurs déclencheurs, retour exact.** Deux boutons ouvrent le même dialogue. À la fermeture, le focus revient au bouton qui l’a ouvert, pas à un emplacement par défaut.
- **Couches imbriquées.** « Archiver… » ouvre une confirmation au-dessus du dialogue. Échap et Retour système ferment une couche à la fois ; chaque couche rend le focus à son propre déclencheur.
- **Tab et Maj+Tab** restent dans la couche du dessus et bouclent, en ignorant les contrôles désactivés. Un champ natif est une cible comme une autre.
- **État occupé.** Pendant l’archivage, Échap, Retour système et le voile sont consommés sans fermer ; le bouton d’action affiche l’état occupé et garde le focus dans la couche. À la fin, les deux couches se ferment et le focus revient au premier déclencheur.
- **Arrière-plan neutralisé, pas seulement les clics.** Les contrôles couverts sont inéligibles dans le registre et désactivés dans Slint à partir du même indicateur : pointeur, clavier et action d’accessibilité sont neutres. Le voile seul ne suffit pas, car il n’arrête pas les actions sémantiques.

## Essayer l’application

Créez un projet neuf, puis remplacez ses fichiers par les quatre fichiers complets ci-dessous. Les paquets viennent de crates.io ; aucun checkout du kit n’est nécessaire. Les [prérequis natifs](/fr/guides/getting-started/) sont ceux du guide de démarrage.
{% else %}
# Compose dialogs with focus return

This complete Rust and Slint application composes two modal layers inside one window with `KitDialogSurface`, `KitDialogPanel`, `KitDialogHeader`, `KitDialogBody`, `KitDialogFooter` and `KitFocusTarget`. Rust owns the layer stack (`OverlayStack`) and the focus registry (`FocusRegistry`); Slint shows the published state and relays gestures. These are surfaces composed in the application window: the kit does not provide a system modal window.

What the example demonstrates:

- **Several triggers, exact return.** Two buttons open the same dialog. On close, focus returns to the button that opened it, not to a default location.
- **Nested layers.** “Archive…” opens a confirmation above the dialog. Escape and system Back close one layer at a time; each layer returns focus to its own trigger.
- **Tab and Shift+Tab** stay inside the topmost layer and wrap, skipping disabled controls. A native text field is a target like any other.
- **Busy state.** While archiving runs, Escape, system Back and the backdrop are consumed without closing; the action button shows the busy state and keeps focus inside the layer. On completion both layers close and focus returns to the first trigger.
- **Background neutralized, not only clicks.** Covered controls are ineligible in the registry and disabled in Slint from the same flag: pointer, keyboard and accessibility activation are all neutral. The backdrop alone is not enough, because it does not stop semantic actions.

## Try the application

Create a fresh project, then replace its files with the four complete files below. Packages come from crates.io; no kit checkout is needed. The [native prerequisites](/guides/getting-started/) match the getting-started guide.
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

- `OverlayStack` reçoit un `OverlaySpec` par couche ouverte, avec son parent et le `FocusHandle` du déclencheur dans `restore_focus`. `close(id, reason)` applique la politique de fermeture (`busy` bloque Échap, Retour et voile, jamais une fermeture programmatique), retire la couche et ses descendantes, et rend les entrées fermées : la dernière porte la cible de restauration.
- `FocusRegistry` possède une portée par couche. `push_trap(scope)` borne le cycle de Tab à la couche du dessus ; `request(handle)` refuse toute cible inéligible ou hors du piège actif ; `restore(preferred, fallback)` choisit la première cible encore valide. Les handles portent une génération : une cible démontée puis remontée ne peut pas recevoir une requête périmée.
- Slint reçoit trois projections : un `FocusSlot` par jeton (génération et éligibilité), un `DialogState` par couche (`open`, `topmost`, `busy`) et la `FocusRequest` à appliquer. `KitFocusTarget` applique la requête une seule fois et relaie `focus-requested` vers `take-focus()` du contrôle réel. `KitDialogSurface` ne traite Échap, Retour et le voile que lorsque son état est `topmost`, et capture Tab/Maj+Tab vers `focus-next(bool)` en politique `host`.
- L’hôte publie après avoir relâché son emprunt : appliquer une requête de focus rappelle `focus-changed`, qui réemprunte la session.

L’éligibilité est calculée une fois, dans Rust, puis publiée ; Slint n’en possède pas de copie. Dans cet exemple, l’action occupée reste éligible pendant le travail pour que la couche conserve le focus et consomme Échap et Retour ; ses voisines sont désactivées.
{% else %}
1. Activate **Archive from the project card**. The “Project name” field takes focus. Tab visits Cancel, then Archive…, then returns to the field; Shift+Tab goes the other way.
2. Press Escape: the dialog closes and focus returns to **Archive from the project card**. Open it from the toolbar: Escape returns focus to that button instead.
3. Open the dialog, then **Archive…**: the confirmation appears above it. Tab only cycles between Keep project and Archive now. Escape closes the confirmation alone and returns focus to **Archive…**; a second Escape closes the dialog.
4. In the confirmation, activate **Archive now**: the button enters the busy state and Escape no longer closes anything. After a moment both layers close and focus returns to the original trigger.

## How the pieces fit

- `OverlayStack` receives one `OverlaySpec` per open layer, with its parent and the trigger’s `FocusHandle` in `restore_focus`. `close(id, reason)` applies the dismissal policy (`busy` blocks Escape, Back and the backdrop, never a programmatic close), removes the layer and its descendants, and returns the closed entries: the last one carries the restore target.
- `FocusRegistry` owns one scope per layer. `push_trap(scope)` bounds the Tab cycle to the topmost layer; `request(handle)` refuses any target that is ineligible or outside the active trap; `restore(preferred, fallback)` picks the first target that is still valid. Handles carry a generation: a target unmounted and mounted again cannot receive a stale request.
- Slint receives three projections: one `FocusSlot` per token (generation and eligibility), one `DialogState` per layer (`open`, `topmost`, `busy`) and the `FocusRequest` to apply. `KitFocusTarget` applies the request once and relays `focus-requested` to the real control’s `take-focus()`. `KitDialogSurface` handles Escape, Back and the backdrop only while its state is `topmost`, and captures Tab/Shift+Tab into `focus-next(bool)` under the `host` policy.
- The host publishes after releasing its borrow: applying a focus request calls back into `focus-changed`, which borrows the session again.

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

## {% if locale == 'fr' %}Exécuter et adapter{% else %}Run and adapt{% endif %}

```sh
cargo run{% if locale == 'fr' %} -- --fr{% endif %}
```

{% if locale == 'fr' %}
Remplacez la temporisation par votre travail asynchrone réel : marquez la couche `busy` au départ, puis fermez-la par `DismissReason::Programmatic` à l’arrivée du résultat. Ajoutez une couche en déclarant son overlay, sa portée, son parent et son premier contrôle ; ajoutez un contrôle en déclarant son jeton et sa couche. Le compilateur signale chaque `match` à compléter. Avec `tab-policy: DialogTabPolicy.native`, Slint parcourt lui-même les contrôles montés ; l’hôte doit alors désactiver tout l’arrière-plan de la même façon.

Slint {{ project.slint }} ne fournit ni rôle de dialogue, ni état modal, ni propriété permettant de masquer un sous-arbre accessible tout en le gardant visible. Le panneau est annoncé comme `groupbox` avec son nom ; les contrôles couverts restent lisibles dans l’ordre de parcours du lecteur d’écran malgré leur désactivation. Le contrat « arrière-plan neutralisé » concerne les actions, pas l’isolement du parcours accessible.

Les tests du kit prouvent ce contrat sans écran : retour du focus au déclencheur exact, couches imbriquées, Tab/Maj+Tab, Échap/Retour, état occupé et arrière-plan inerte. Ils ne qualifient ni les lecteurs d’écran ni les méthodes de saisie : vérifiez-les sur la plateforme cible. Consultez la [référence KitDialogSurface](/fr/components/dialog-surface/) et la [référence KitFocusTarget](/fr/components/focus-target/) pour l’API complète.
{% else %}
Replace the timer with your real asynchronous work: mark the layer `busy` when it starts, then close it with `DismissReason::Programmatic` when the result arrives. Add a layer by declaring its overlay, scope, parent and first control; add a control by declaring its token and layer. The compiler reports every `match` to complete. With `tab-policy: DialogTabPolicy.native`, Slint traverses the mounted controls itself; the host must then disable the whole background in the same way.

Slint {{ project.slint }} exposes no dialog role, modal state, or property that hides an accessible subtree while keeping it visible. The panel is exposed as a named `groupbox`; covered controls remain readable in the screen reader browse order despite being disabled. The inert-background contract covers actions, not isolation of the accessibility browse tree.

The kit’s tests prove this contract headlessly: focus return to the exact trigger, nested layers, Tab/Shift+Tab, Escape/Back, busy state and an inert background. They do not qualify screen readers or input methods: verify those on the target platform. See the [KitDialogSurface reference](/components/dialog-surface/) and the [KitFocusTarget reference](/components/focus-target/) for the full API.
{% endif %}
