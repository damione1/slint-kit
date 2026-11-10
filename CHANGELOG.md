# Changelog

## Unreleased

- Preserve completed macOS compiler caches when a later CI documentation check fails.
- Conserve les caches de compilation macOS terminés lorsqu’un contrôle documentaire ultérieur de CI échoue.

- Add optional roving table-header navigation, localized header context and public sort/resize commands, including empty tables.
- Ajoute une navigation optionnelle par curseur d’en-tête de tableau, un contexte localisable et des commandes de tri/redimensionnement, y compris sans lignes.

- Place sortable table headers and resize handles before the body in keyboard tab order, matching their visual order.
- Place les en-têtes triables et les poignées de tableau avant le corps dans l’ordre de tabulation, comme à l’écran.

- Keep textarea scrollbar hit areas inside their themed gutter so mouse drags reach the native editor and select text.
- Contient la zone de la barre du textarea dans sa gouttière thémée pour que les glissements sélectionnent le texte natif.

- Add ButtonIntent.link with underlined accent text, inline geometry and transparent states while retaining shared button semantics and guards. Breaking: exhaustive ButtonIntent matches require Link and complete ButtonRecipe literals require underline.

- Add success, warning and information button intents with complete palettes and independent status/destructive foreground tokens. Breaking: complete KitTheme literals must provide on-danger and the new on-*/hover/pressed status fields; exhaustive ButtonIntent matches must handle the three new variants.

- Fetch release history in documentation CI so immutable released examples remain verifiable from a fresh checkout.
- Récupère l’historique des versions en CI pour vérifier les exemples publiés immuables depuis un checkout neuf.
- Clear disabled action focus and its outline once, preserving busy focus and preventing focus from returning on reenabling.
- Retire le focus et son halo à la désactivation d’une action, sans restauration à la réactivation ; une action occupée conserve le focus.

- Validate intrinsic table field constraints with the shared HTML engine and let hosts reject a commit while retaining its draft. **Breaking:** `CellContent::Field` gains optional `constraints`.
- Valide les contraintes intrinsèques des cellules avec le moteur HTML partagé et permet de refuser une validation en conservant le brouillon. **Rupture :** `CellContent::Field` reçoit un champ optionnel `constraints`.

- Preserve selected, striped and text presentation when table interaction is blocked; unavailable records retain their own disabled appearance.

- Replace stock text editing menus with themed kit popups compatible with renderer-free builds, preserving native commands and hidden-password/paste guards. **Behavior change:** field focus includes its owned editing menu; document Slint outside-click, accessibility-role and touch limits.
- Remplace les menus d’édition standard par des popups thémés compatibles sans renderer imposé, avec commandes natives et gardes de mot de passe masqué/collage. **Changement de comportement :** le focus du champ inclut son menu ; documente les limites Slint de clic extérieur, rôles accessibles et tactile.

- Initialize selection controllers created after window installation, including conditional controls and popup content, without requiring an options mutation.
- Initialise les contrôleurs de sélection créés après installation de la fenêtre, y compris les contrôles conditionnels et popups, sans exiger de mutation des options.

- Expose one public `KitKeyResult` enum for keyboard hooks so Rust handlers can accept or reject events without private Slint types. Convert only at native event boundaries. **Breaking:** text-field `key-pressed`, table/pressable `key-event`, selector `tab-navigation` and public selector/navigation `handle-key` return `KitKeyResult`; Slint handlers use qualified `.accept`/`.reject` variants.


- Check complete downloadable applications against registry crates before a release documentation build and against local sources in CI. Development guides disclose unreleased APIs and link the immutable 0.7.2 registry example.

- Make file drop zones browseable by click, Enter, Space and assistive activation. Add description, icon and child composition while preserving disabled/read-only guards.

- Parent native file and directory dialogs to their Slint window, with framework-independent parent-provider APIs that retain no window reference while awaiting results.

- Localize file actions/status and step names through window KitStrings with instance overrides and removal-name templates. Native picker bindings accept typed error mappers and preserve platform details.

- Animate button busy indicators using theme timing, honor reduced motion, and expose busy-label for an accessible explanation of pending work.

- Protect form-button validation from overridden activated handlers. Breaking: submit buttons now require a validation-requested callback returning true, or explicit form-no-validate; form-action remains the submit/reset notification.

- Document Slint 1.18.1 modal accessibility limits: disabled background actions remain readable, and panels expose a named groupbox rather than a dialog/modal role.

- Render success, warning and information stat tones, applying the chosen ink to the value and icon in both themes.

- Apply file selection mode and editable access to native drops, including window-wide winit events. Expose binding setters; single-file drops replace the selection and disabled/read-only drops are ignored.

- Stop forcing Slint element debug metadata into consumer builds. Runtime inspection follows Slint defaults and can be enabled explicitly; kit interaction fixtures opt in.

- Record the successful Ubuntu 24.04 headless component/workspace tests and clippy run at [59ece023](https://github.com/damione1/slint-kit/actions/runs/42769690387/job/129674322933), keeping native Linux, mobile, IME and screen-reader qualification separate.

- Run the Linux component suite once, exclude it from the remaining workspace tests, and bound the CI job to 45 minutes.

- Keep the dialog consumer on the workspace release and check documentation dependency versions in CI; integration dependencies now inherit workspace versions.

- Present unavailable native file/directory pickers through their capability state without a duplicate field error.

- Document text-field callback payloads, native editing functions, focus, visibility, localization and accessibility in English and French; retain generated API tables and inherited reference links.
- Documente en français et anglais les callbacks, fonctions natives, focus, visibilité, localisation et accessibilité des champs ; conserve les tableaux API générés et liens hérités.

- Preserve text-field help alongside accessible errors, expose placeholder/required/count/pending descriptions and localisable visible required text; document Slint accessibility and mobile input limits.
- Préserve l’aide avec les erreurs accessibles, expose espace réservé/obligation/compteur/vérification et l’indication d’obligation localisable ; documente les limites Slint d’accessibilité et saisie mobile.

- Count text graphemes automatically and offer localisable count/maximum/remaining displays with HTML-compatible UTF-16 limits and a warning threshold.
- Compte automatiquement les graphèmes et propose les formats compte/maximum/reste localisables avec les limites UTF-16 HTML et un seuil d’avertissement.

- Add an IME-aware text key hook, explicit Escape policies and optional modified-Enter multiline submission.
- Ajoute un hook clavier respectant la préédition IME, des politiques Échap et l’envoi multiligne optionnel par Entrée modifiée.

- Add window-local `KitStrings` defaults, plural count labels and templates; localise codec/phone failures and `FieldBinding` messages. Breaking: pure field/phone service callbacks receive message data.
- Ajoute les messages de fenêtre `KitStrings`, les compteurs pluriels et les modèles ; localise les erreurs codec/téléphone et `FieldBinding`. Rupture : les callbacks purs de champ/téléphone reçoivent les messages.

- Preserve native undo and commit dispatch when clearing text or search fields. Reset/cancel/number steps remain explicit baseline replacements.
- Préserve l’annulation native et l’engagement lors de l’effacement texte/recherche. Réinitialisation, annulation et pas numériques remplacent explicitement la référence.

- Expose native text selection, insertion, editing history and focus APIs and a localisable context menu.
- Expose la sélection, l’insertion, l’historique d’édition et le focus natifs ainsi qu’un menu contextuel localisable.

- Keep mouse drag selection in overflowing text areas and provide the themed vertical scrollbar.
- Préserve la sélection par glissement souris dans les zones de texte débordantes et ajoute la barre verticale thémée.


- Accept the same visual phone separators during typing and paste while preserving canonical digits.
- Accepte les mêmes séparateurs téléphoniques en saisie et au collage en conservant les chiffres canoniques.

- Breaking: text-field `report-validity()` reports without taking focus; use `focus-if-invalid()` for explicit focus control.
- Rupture : `report-validity()` des champs texte signale sans prendre le focus ; utilisez `focus-if-invalid()` pour le déplacer explicitement.

- Avoid unchanged blur commits and honor HTML dirty state and validation timing for text fields. Breaking: `KitFieldService.reduce` now takes the actual user-edit flag.
- Évite les engagements au flou sans changement et respecte l’état modifié HTML et le moment de validation. Rupture : `KitFieldService.reduce` reçoit désormais l’indicateur réel de saisie.

- Commit the completed native text edit for `on-change-when-valid`; ignore navigation keys and include search clear actions.
- Engage la saisie native terminée pour `on-change-when-valid`, ignore les touches de navigation et inclut l’effacement de recherche.

- Expose localized editor, placeholder, Save and Copy labels on `KitVirtualTextView`; translate the French example.

- Document table cell construction, callback argument order, editing functions and localized accessibility hints in English and French; show callable descriptions in the generated component reference.

- Expose a table key-event callback before built-in navigation so hosts can implement shortcuts.

- Refuse table field editing before the field service is installed, including plain-text cells, instead of opening an editor that cannot commit.

- Reveal the active row before opening its cell editor and scroll the active cell horizontally using resolved layout geometry.

- Hit-test the table cell cursor from resolved pointer geometry, open fields on double-click or F2 and expose edit/commit/cancel commands and an accessible edit action.

- Preserve collection keyboard navigation after a row press is cancelled. Add `KitPressable.focus-on-pointer` so composite rows focus their shared keyboard scope on press.

- Expose the active table cell value and localized editability to accessibility, remove duplicate header text names and retain a container focus indicator when a collection has no visible active row.

- Lay out every table row against all declared columns: missing cells stay empty and extra cells do not create columns or shift the header grid.

- Ignore stationary column resize clicks, cancel drag previews with Escape or secondary press, enlarge scaled resize targets and expose `autosize-requested(column-token)` on double-click.

- Keep a resizable fixed column above the table minimum so dragging cannot hide its resize handle or change its floor after the host applies a width.

- Give table cells the full row height so native editors and their focus borders fit at every control size and text scale.

- Keep keyboard focus on the destination control when a table cell commits on blur; return focus to the table only for an editor that still owns it.


- Preserve active editor drafts when other collection rows change or unchanged rows reorder. Add key-based record access and updates that retain the generation and notify one row; navigation no longer reconstructs the model. Explicit replacement republishes the active record.

- Add an English/French contact form with server-side Turnstile verification, bounded input validation, rate limiting and private email delivery through the existing documentation Worker. Preserve drafts when sending fails and link contact from the homepage and guides.

- Replace homepage screenshots with lightweight, theme-aware HTML illustrations and demonstrate the documented ascending/descending table sort while retaining selection by key. Respect reduced motion, pause offscreen playback and allow manual control. Keep the open-source section dark in dark mode and preserve displayed/copied snippet indentation. No component API changes.

- Redesign the bilingual homepage with real component captures, an on-demand Slint demo, integration excerpts and clear installation and licensing paths.
- Explain Slint Kit's shared interaction and typed domain layer across the bilingual homepage, adoption guide, component catalogue, README and crate entry points. Link runnable behavior examples, distinguish complete controls from composition primitives, document adoption limits and clarify registry installation in the typed example. Correct outdated email/URL validation descriptions and clarify that base components stay free for open-source use, with commercial licensing on request and Slint Kit Pro coming. No public API changes.
- Component reference: the property tables of every component gain a "Default" column read from the declared initializer in the Slint source (`—` for the zero value of the type, a pointer to the source for a composite expression, nothing for a computed `out` property). The central properties of the fields (`KitTextField` and its inheritors, `KitTelephoneField`, `KitOtpField`, `KitNumberStepper`), the selections (`KitSelect`, `KitCombobox`, `KitMultiSelect`, `KitRadioGroup`, `KitSegmentedControl`, `KitToggleGroup`, `KitCheckbox`, `KitSwitch`) and the data views (`KitDataTable`, `KitContentList`) get a "Property details" section in English and French: effect, default, states, constraints and a short example that uses public mechanisms, with a link to the complete project of the family. The explanations live once in `docs/content/components.json` on the component that declares the property; inherited properties repeat the owner's explanation, and a built-in option (`label`, `description`, `error`) links its composition primitive only when the owner renders it. The documentation build refuses an explanation for a property that is neither declared nor inherited, a mismatch between the two languages or an example that does not use the property. No public API changes.
- Platform proofs: the Check workflow gains a `linux` job (GitHub-hosted `ubuntu-24.04`, x86_64) that installs the Linux development libraries, builds the workspace, runs the headless test suite on the Slint testing backend with the software renderer and runs clippy with warnings denied on the Linux configuration paths; the macOS full check and the Windows build checks are unchanged. The getting-started guide gains "Announced and qualified targets": the announced desktop platforms (macOS, Windows, Linux with winit and Skia) are distinguished from the qualified ones, each proof names its OS, backend and renderer, and headless tests and native captures are stated not to be a native, IME or screen-reader qualification. The README, crate README, themes guide and integration skill point at it. No public API changes.
- Dialog composition guide: a complete Rust and Slint application (`/guides/dialog-example/`, `docs/dialog-consumer/`) composes two nested in-window modal layers from the existing `KitDialogSurface`, `KitDialogPanel`, `KitDialogHeader`, `KitDialogBody`, `KitDialogFooter` and `KitFocusTarget` parts on `OverlayStack` and `FocusRegistry`: several triggers with exact focus return, Tab/Shift+Tab cycling, Escape and system Back one layer at a time, a busy layer that consumes dismissal, and background controls neutralized from one published eligibility flag (pointer, keyboard and accessibility actions alike). The guide, the `KitDialogSurface` and `KitFocusTarget` pages and the integration skill point at it; `tests/` proves the contract on the real window. No public API changes.
- The sortable table guide and its downloadable application now take the short path: `KeyedModel<ProjectId, TableRow>` with `bind`, `navigation()`, `activation(handler)`, `sorting_by` and `replace` (an archived-rows switch) replaces the hand-written keys, generations, sorting and model publication; cells use `CellContent::text`, and the sort labels are localized. The mechanism explanation stays as an advanced section. No public API changes.
- `KitDataTable` field cells are edited in place. `TableCellKind` gains `field`, and `TableCell` gains `access: FieldAccess` and `spec` (the key of a `FieldSpec` installed with `install_field_specs`, `""` for free text); in Rust, `CellContent::field(text, FieldAccess)` and `CellContent::typed_field(key, &spec, Option<&T>, FieldAccess)` (new `CellCodec`) format the accepted value once through its codec, which also parses and validates the draft inside the editor and guards the Rust path through `spec.accept(commit)`. Left/Right move a cell cursor in the active row (new `active-column`, named in the table's accessible label); Enter on an editable field opens the table's single `KitTextField` with the value selected, Enter commits, Escape cancels, Tab/Shift+Tab commit and move the cursor, and focus loss commits an acceptable draft or drops an unacceptable one. An invalid draft stays in the editor with its message in the new `edit-error` (new `editing` reports an open editor); read-only cells open no editor, disabled cells are dimmed, and `interactive: false` closes one. The commit is reported by `cell-committed(row token, column token, generation, FieldCommit)` with the identity armed when editing began, never positions: a new `generation` closes the editor without committing, and the new `KeyedModel::cell_commits(handler)` rejects a commit armed under an old generation before handing the row key, the column token and the commit to the application. Only the edited viewport cell mounts an editor. `KitPressable` gains `pointer-enabled` so a row hands the pointer to its open editor. **Breaking for Rust:** `CellContent` has a new `Field` variant, so exhaustive matches need an arm, and `TableCell` struct literals need `access` and `spec` (`..Default::default()` keeps the previous meaning); Slint literals may omit them. Headless tests cover the cursor, Enter/Escape/Tab, the draft/accepted distinction, invalid/read-only/disabled states, generation fencing, focus loss and the single viewport editor; IME, screen readers and light/dark rendering at 200% text still need manual qualification, and no performance or memory figure is claimed.
- `KitDataTable` rows accept typed presentation cells. `TableRow` gains `content: [TableCell]`, one cell per column in column order, next to the unchanged `cells: [string]`; `TableCell { kind: TableCellKind (text | badge | check), text, tone: Tone, check: CheckState }` is the flat projection of the new Rust `CellContent` enum (`text`, `badge(text, Tone)`, `check(CheckState, text)`), converted once by `From` or `table_cells(...)`. `badge` paints a `KitBadge` with the cell's tone, `check` a read-only check mark shared with `KitCheckbox` (new `CheckMark`) for unchecked, checked and mixed, and the cell's `text` is the accessible label of both; `content` wins when a row carries both forms, and the row's accessible name is its first cell's text either way. The cells are painted by the kit inside the virtualized row because Slint 1.18.1 rejects `@children` and slot placeholders in a repeated element and keeps dynamic components experimental; caller-supplied cell templates, cell toggling and cell editing are not provided. **Breaking for Rust:** `TableRow` struct literals need the new `content` field (`ModelRc::default()` keeps the string behaviour); Slint literals may omit it. Headless tests cover rendering by kind, header alignment, precedence, virtualization and the projection; light/dark rendering at 200% text and screen readers still need manual qualification.
- `KitDataTable` columns have a typed width policy. `TableColumn` gains `width: ColumnWidth { sizing: ColumnSizing (fill | fixed), basis, minimum, ceiling: ColumnCeiling (unbounded | bounded), maximum, resizable }`, carried by the column so it follows the column's stable token through reordering. `fill` (the default) starts at `basis`, never drops below `minimum-column-width` or its own `minimum`, and shares the remaining space equally; `fixed` is exactly `basis` within its bounds and never grows; a bounded `maximum` caps growth and resizing, and space left after every column reached its maximum stays empty. Header and row cells share one set of constraints, so they stay aligned at every width, and the header follows the horizontal scroll (new `scroll-x`). A `resizable` column shows a handle at its trailing edge: pointer drags preview live and emit one clamped `resize-requested(token, width)` on release; the handle is a tab stop and an accessible `spinbox` (`label, resize-label`) stepping by the new `KitThemes.column-handle-recipe` step with Left/Right, Home/End and the increment/decrement actions. The host applies the request by changing the column's policy. **Breaking for Rust:** `TableColumn` struct literals need the new `width` field (`ColumnWidth::default()` keeps the previous behaviour); Slint literals may omit it. Geometry, pointer, keyboard and identity tests cover the contract; light/dark rendering at 200% text, platform cursors and screen readers still need manual qualification.
- `KitDataTable` accessible text is localizable: new `sort-ascending-label`, `sort-descending-label` and `unnamed-row-label` properties replace the hard-coded English sort suffixes and row fallback. A header with `sortable: false` is now exposed as static `text` instead of a disabled `button`, and the table's accessible label names the active row (`label: first cell`) so focus, active row and row selection stay distinct. Headless tests prove the sort, activation and selection contract across keyboard, pointer and accessible actions; they are not a screen-reader qualification. No breaking change: defaults keep the previous English names.

### Français

- Ajoute ButtonIntent.link : texte accentué souligné, hauteur de ligne et états transparents avec les gardes du bouton. Rupture : les correspondances ButtonIntent exigent Link et les littéraux ButtonRecipe exigent underline.

- Ajoute les intentions de bouton success, warning et information avec palettes complètes et encres de statut/destruction indépendantes. Rupture : les littéraux KitTheme complets doivent fournir on-danger et les champs on-*/hover/pressed de statut ; les correspondances ButtonIntent exhaustives doivent traiter les trois variantes ajoutées.

- Vérifie les applications complètes téléchargeables contre les crates publiées avant de construire la documentation de release et contre les sources locales en CI. Les guides de développement signalent les API non publiées et relient l’exemple crates.io immuable 0.7.2.

- Rend les zones de dépôt activables au clic, avec Entrée/Espace et par les technologies d’assistance. Ajoute description, icône et composition d’enfants, en conservant les gardes de lecture seule et de désactivation.

- Attache les boîtes de fichiers et dossiers natives à leur fenêtre Slint, avec des API de parent indépendantes du framework qui ne conservent pas de référence à la fenêtre pendant l’attente.

- Localise actions/états des fichiers et noms d’étapes via KitStrings de la fenêtre, avec surcharges et modèles de suppression. Les bindings natifs acceptent des mappers d’erreurs typées et conservent les détails système.

- Vérifie les applications complètes téléchargeables contre les crates publiées avant de construire la documentation de release et contre les sources locales en CI. Les guides de développement signalent les API non publiées et relient l’exemple crates.io immuable 0.7.2.




- Anime les indicateurs d’attente des boutons selon le thème, respecte la réduction des mouvements et ajoute busy-label pour expliquer le travail en cours aux technologies d’assistance.

- Protège la validation des boutons de formulaire lorsque activated est remplacé. Rupture : les boutons de soumission exigent désormais validation-requested retournant true, ou form-no-validate explicite ; form-action signale toujours soumission/réinitialisation.

- Documente les limites d’accessibilité modale de Slint 1.18.1 : les actions désactivées restent lisibles et les panneaux exposent un groupbox nommé plutôt qu’un rôle dialogue/modal.

- Rend les tons success, warning et information des statistiques et applique le ton à la valeur et à l’icône dans les deux thèmes.

- Applique le mode de sélection et l’accès aux dépôts natifs, y compris les événements winit de la fenêtre. Ajoute les setters du binding ; un dépôt simple remplace la sélection et les dépôts désactivés ou en lecture seule sont ignorés.

- N’impose plus les métadonnées de débogage Slint aux applications. L’inspection suit les réglages Slint et s’active explicitement ; les fixtures d’interaction du kit l’activent.

- Consigne le succès des tests headless composants/workspace et de clippy sur Ubuntu 24.04 à [59ece023](https://github.com/damione1/slint-kit/actions/runs/42769690387/job/129674322933), sans le confondre avec une qualification Linux native, mobile, IME ou lecteur d’écran.

- Exécute une seule fois la suite des composants sur Linux, l’exclut des autres tests du workspace et limite le job CI à 45 minutes.

- Aligne l’exemple de dialogue sur la version du workspace et vérifie les versions documentées en CI ; l’intégration hérite désormais des dépendances du workspace.

- Présente les sélecteurs natifs indisponibles par leur état de capacité, sans erreur de champ en double.

- Expose les libellés localisés de l’éditeur, du placeholder et des actions Enregistrer/Copier de `KitVirtualTextView` ; traduit l’exemple français.

- Documente la construction des cellules, l’ordre des arguments, les fonctions d’édition et les indications accessibles en français et anglais ; affiche les descriptions des fonctions et callbacks dans la référence.

- Expose un callback key-event avant la navigation intégrée pour les raccourcis de l’hôte.

- Refuse l’édition de cellule avant installation du service, même en texte libre, au lieu d’ouvrir un éditeur incapable de confirmer.

- Révèle la ligne active avant ouverture de l’éditeur et défile horizontalement vers la cellule active à partir de la géométrie résolue.

- Positionne le curseur de cellule par la géométrie du pointeur, ouvre les champs au double-clic ou F2 et expose les commandes d’édition ainsi qu’une action accessible.

- Conserve la navigation clavier des collections après annulation d’un appui. Ajoute `KitPressable.focus-on-pointer` pour donner le focus à la portée clavier commune des lignes.

- Expose la valeur de cellule active et son état modifiable localisé à l’accessibilité, retire les noms d’en-tête dupliqués et conserve un indicateur de focus sans ligne active visible.

- Aligne chaque ligne sur toutes les colonnes déclarées : les cellules absentes restent vides et les cellules supplémentaires ne décalent plus la grille.

- Ignore les clics sans mouvement sur les poignées, annule le glisser avec Échap ou un appui secondaire, agrandit les cibles et expose `autosize-requested(column-token)` au double-clic.

- Applique le minimum du tableau aux colonnes fixes redimensionnables pour conserver leur poignée et leur plancher après application d’une largeur par l’hôte.

- Donne aux cellules toute la hauteur de ligne afin que les éditeurs natifs et leurs bordures de focus restent visibles à chaque densité et échelle de texte.

- Conserve le focus sur le contrôle de destination lorsqu’une cellule est confirmée à la perte de focus ; rend le focus au tableau uniquement si l’éditeur le possède encore.


- Conserve le brouillon actif lorsque d’autres lignes changent ou sont triées. Ajoute un accès métier par clé et des mises à jour conservant la génération et notifiant une ligne ; la navigation ne reconstruit plus le modèle. Un remplacement explicite republie la ligne active.

- Ajoute un formulaire de contact français/anglais avec vérification Turnstile côté serveur, validation bornée, limitation des tentatives et envoi privé par le Worker existant de la documentation. Conserve le texte en cas d’échec et relie le contact depuis l’accueil et les guides.

- Remplace les captures de l’accueil par des illustrations HTML légères adaptées au thème et démontre le cycle de tri documenté du tableau en conservant la sélection par clé. Respecte la réduction des mouvements, suspend l’animation hors écran et permet le contrôle manuel. Corrige le thème sombre de la section open source et l’indentation des extraits affichés/copiés. Aucun changement d’API de composant.

- Revoit l’accueil bilingue avec de vraies captures de composants, une démo Slint à la demande, des extraits d’intégration et des parcours clairs d’installation et de licence.
- Présente la couche d’interaction partagée et les valeurs métier typées de Slint Kit sur l’accueil bilingue, le guide de choix, le catalogue, le README et les entrées des crates. Relie les exemples exécutables, distingue contrôles complets et briques de composition, précise les limites d’adoption et l’installation depuis crates.io dans l’exemple typé. Corrige les descriptions obsolètes de validation courriel/URL et précise que les composants de base restent gratuits pour l’usage open source, avec licence commerciale sur demande et Slint Kit Pro à venir. Aucun changement d’API publique.
- Référence des composants : les tableaux de propriétés de chaque composant gagnent une colonne « Défaut » lue depuis l’initialisation déclarée dans la source Slint (`—` pour la valeur zéro du type, un renvoi à la source pour une expression composée, rien pour une propriété `out` calculée). Les propriétés centrales des champs (`KitTextField` et ses héritiers, `KitTelephoneField`, `KitOtpField`, `KitNumberStepper`), des sélections (`KitSelect`, `KitCombobox`, `KitMultiSelect`, `KitRadioGroup`, `KitSegmentedControl`, `KitToggleGroup`, `KitCheckbox`, `KitSwitch`) et des vues de données (`KitDataTable`, `KitContentList`) reçoivent une section « Détail des propriétés » en anglais et en français : effet, défaut, états, contraintes et un court exemple qui emploie les mécanismes publics, avec un lien vers le projet complet de la famille. Les explications vivent une seule fois dans `docs/content/components.json`, sur le composant qui déclare la propriété ; les propriétés héritées reprennent l’explication du propriétaire, et une option intégrée (`label`, `description`, `error`) ne renvoie à sa primitive de composition que lorsque le propriétaire la rend. La construction de la documentation refuse une explication pour une propriété ni déclarée ni héritée, un écart entre les deux langues ou un exemple qui n’emploie pas la propriété. Aucun changement d’API publique.
- Preuves par plateforme : le workflow Check gagne un job `linux` (runner GitHub `ubuntu-24.04`, x86_64) qui installe les bibliothèques de développement Linux, compile l’espace de travail, exécute la suite de tests headless sur le backend de test Slint avec le moteur de rendu logiciel et lance clippy avec avertissements refusés sur les chemins de configuration Linux ; la vérification complète macOS et les vérifications de compilation Windows sont inchangées. Le guide Bien démarrer gagne « Cibles annoncées et cibles qualifiées » : les plateformes de bureau annoncées (macOS, Windows, Linux avec winit et Skia) sont distinguées des cibles qualifiées, chaque preuve nomme son système, son backend et son moteur de rendu, et les tests headless comme les captures natives sont déclarés ne pas être une qualification native, IME ou lecteur d’écran. Le README, le README de la crate, le guide des thèmes et le skill d’intégration y renvoient. Aucun changement d’API publique.
- Guide de composition des dialogues : une application Rust et Slint complète (`/fr/guides/dialog-example/`, `docs/dialog-consumer/`) compose deux couches modales imbriquées dans la fenêtre à partir des parties existantes `KitDialogSurface`, `KitDialogPanel`, `KitDialogHeader`, `KitDialogBody`, `KitDialogFooter` et `KitFocusTarget` sur `OverlayStack` et `FocusRegistry` : plusieurs déclencheurs avec retour exact du focus, cycle Tab/Maj+Tab, Échap et Retour système une couche à la fois, couche occupée qui consomme la fermeture, et contrôles d’arrière-plan neutralisés depuis un seul indicateur d’éligibilité publié (pointeur, clavier et actions d’accessibilité). Le guide, les pages `KitDialogSurface` et `KitFocusTarget` et le skill d’intégration y renvoient ; `tests/` prouve le contrat sur la fenêtre réelle. Aucun changement d’API publique.
- Le guide du tableau triable et son application téléchargeable prennent le chemin court : `KeyedModel<ProjectId, TableRow>` avec `bind`, `navigation()`, `activation(handler)`, `sorting_by` et `replace` (un interrupteur masque les lignes archivées) remplace les clés, générations, tri et publication du modèle écrits à la main ; les cellules utilisent `CellContent::text` et les libellés de tri sont localisés. L’explication du mécanisme devient une section avancée. Aucun changement d’API publique.
- Les cellules `field` de `KitDataTable` s’éditent sur place. `TableCellKind` gagne `field`, et `TableCell` gagne `access: FieldAccess` et `spec` (la clé d’un `FieldSpec` installé par `install_field_specs`, `""` pour du texte libre) ; en Rust, `CellContent::field(text, FieldAccess)` et `CellContent::typed_field(key, &spec, Option<&T>, FieldAccess)` (nouveau `CellCodec`) formatent la valeur acceptée une seule fois par son codec, qui analyse et valide aussi le brouillon dans l’éditeur et garde le chemin Rust via `spec.accept(commit)`. Gauche/Droite déplacent un curseur de cellule dans la ligne active (nouveau `active-column`, nommé dans le libellé accessible du tableau) ; Entrée sur un champ modifiable ouvre l’unique `KitTextField` du tableau avec la valeur sélectionnée, Entrée valide, Échap annule, Tab/Maj+Tab valident et déplacent le curseur, et la perte du focus valide un brouillon acceptable ou abandonne un brouillon inacceptable. Un brouillon invalide reste dans l’éditeur avec son message dans le nouveau `edit-error` (le nouveau `editing` signale un éditeur ouvert) ; les cellules en lecture seule n’ouvrent pas d’éditeur, les cellules désactivées sont atténuées, et `interactive: false` ferme l’éditeur. Le commit est signalé par `cell-committed(jeton de ligne, jeton de colonne, génération, FieldCommit)` avec l’identité armée au début de l’édition, jamais des positions : une nouvelle `generation` ferme l’éditeur sans valider, et le nouveau `KeyedModel::cell_commits(handler)` rejette un commit armé sous une ancienne génération avant de transmettre la clé de ligne, le jeton de colonne et le commit à l’application. Seule la cellule éditée du viewport monte un éditeur. `KitPressable` gagne `pointer-enabled` pour qu’une ligne cède le pointeur à son éditeur ouvert. **Rupture côté Rust :** `CellContent` a une nouvelle variante `Field`, les `match` exhaustifs ont donc besoin d’une branche, et les littéraux de struct `TableCell` doivent fournir `access` et `spec` (`..Default::default()` conserve le sens précédent) ; les littéraux Slint peuvent les omettre. Des tests headless couvrent le curseur, Entrée/Échap/Tab, la distinction brouillon/valeur acceptée, les états invalide/lecture seule/désactivé, le verrou de génération, la perte du focus et l’éditeur unique du viewport ; IME, lecteurs d’écran et rendu clair/sombre à 200 % restent à qualifier manuellement, et aucun chiffre de performance ou de mémoire n’est promis.
- Les lignes de `KitDataTable` acceptent des cellules de présentation typées. `TableRow` gagne `content: [TableCell]`, une cellule par colonne dans l’ordre des colonnes, à côté de `cells: [string]` inchangé ; `TableCell { kind: TableCellKind (text | badge | check), text, tone: Tone, check: CheckState }` est la projection plate du nouvel enum Rust `CellContent` (`text`, `badge(text, Tone)`, `check(CheckState, text)`), convertie une seule fois par `From` ou `table_cells(...)`. `badge` peint un `KitBadge` avec le ton de la cellule, `check` une marque de case en lecture seule partagée avec `KitCheckbox` (nouveau `CheckMark`) pour les états décoché, coché et mixte, et le `text` de la cellule est le libellé accessible des deux ; `content` l’emporte lorsqu’une ligne fournit les deux formes, et le nom accessible de la ligne est le texte de sa première cellule dans les deux cas. Les cellules sont peintes par le kit dans la ligne virtualisée, car Slint 1.18.1 refuse `@children` et les emplacements de slot dans un élément répété et garde les composants dynamiques expérimentaux ; gabarits de cellule fournis par l’appelant, basculement et édition par cellule ne sont pas fournis. **Rupture côté Rust :** les littéraux de struct `TableRow` doivent fournir le nouveau champ `content` (`ModelRc::default()` conserve le comportement des chaînes) ; les littéraux Slint peuvent l’omettre. Des tests headless couvrent le rendu par type, l’alignement avec l’en-tête, la priorité, la virtualisation et la projection ; le rendu clair/sombre à 200 % et les lecteurs d’écran restent à qualifier manuellement.
- Les colonnes de `KitDataTable` ont une politique de largeur typée. `TableColumn` gagne `width: ColumnWidth { sizing: ColumnSizing (fill | fixed), basis, minimum, ceiling: ColumnCeiling (unbounded | bounded), maximum, resizable }`, portée par la colonne pour suivre son jeton stable lors d’un réordonnancement. `fill` (par défaut) part de `basis`, ne descend jamais sous `minimum-column-width` ni sous son propre `minimum`, et partage l’espace restant à parts égales ; `fixed` vaut exactement `basis` dans ses bornes et ne grandit jamais ; un `maximum` borné plafonne croissance et redimensionnement, et l’espace restant une fois chaque colonne à son maximum reste vide. En-têtes et cellules partagent les mêmes contraintes et restent alignés à toute largeur ; l’en-tête suit le défilement horizontal (nouveau `scroll-x`). Une colonne `resizable` affiche une poignée à son bord de fin : le glissement au pointeur prévisualise en direct et émet au relâchement un seul `resize-requested(token, width)` borné ; la poignée est un arrêt de tabulation et un `spinbox` accessible (`libellé, resize-label`) qui avance du pas de la nouvelle recette `KitThemes.column-handle-recipe` avec Gauche/Droite, Début/Fin et les actions incrémenter/décrémenter. L’hôte applique la demande en modifiant la politique de la colonne. **Rupture côté Rust :** les littéraux de struct `TableColumn` doivent fournir le nouveau champ `width` (`ColumnWidth::default()` conserve le comportement précédent) ; les littéraux Slint peuvent l’omettre. Des tests de géométrie, pointeur, clavier et identité couvrent le contrat ; le rendu clair/sombre à 200 %, les curseurs de plateforme et les lecteurs d’écran restent à qualifier manuellement.
- Le texte accessible de `KitDataTable` est localisable : les nouvelles propriétés `sort-ascending-label`, `sort-descending-label` et `unnamed-row-label` remplacent les suffixes de tri anglais et le libellé de ligne codés en dur. Un en-tête avec `sortable: false` est maintenant exposé comme `text` statique plutôt que comme `button` désactivé, et le libellé accessible du tableau nomme la ligne active (`libellé : première cellule`) pour que focus, ligne active et sélection restent distincts. Des tests headless prouvent le contrat de tri, d’activation et de sélection au clavier, au pointeur et par action accessible ; ils ne sont pas une qualification de lecteur d’écran. Aucun changement cassant : les valeurs par défaut gardent les noms anglais précédents.

## 0.7.2

- Fix empty pure-Slint previews across the selection family: render declarative options in menus, tabs, breadcrumbs, carousels, radio/segmented/toggle groups and accordion headers; render arbitrary valid rating scales. Keep service-backed interaction guarded until `install(&window)`. Preserve explicit row projections and document static projection limits for dropdowns, multi-selection, calendars and typed fields.
- Report an actionable build error when `SLINT_LIVE_PREVIEW` is set, instead of unresolved shared globals from Slint 1.18.1's live-preview generator (upstream slint-ui/slint#14812). Unset the variable, including assignments in `build.rs`; `slint/mcp` and `SLINT_EMIT_DEBUG_INFO` remain usable. No public API changes.

- Present desktop and mobile Rust UI use cases consistently across GitHub, crates.io, docs.rs and the bilingual site. Document Android/iOS setup, desktop-only file dialogs and device qualification limits; clarify quality evidence and verify the generated AI documentation index.

- Slint requirement relaxed from `=1.18.1` to `~1.18.1`: applications can use any Slint 1.18 patch release from 1.18.1. Keep `slint-kit` and `slint-kit-build` on the same release. No public API changes.
- The documentation site publishes `/llms.txt`, generated from the component inventory, for AI assistants and search tools.
- README, crates.io page, docs.rs crate documentation and crate metadata now present the component catalogue, screenshots, engineering guarantees and licensing plans.

### Français

- Corrige les aperçus Slint seuls vides de la famille sélection : options déclarées visibles dans menus, onglets, fils d’Ariane, carrousels, groupes radio/segments/toggles et en-têtes d’accordéon ; échelles de notation valides visibles. Les interactions restent protégées jusqu’à `install(&window)`. Conserve les projections de lignes explicites et documente les limites statiques des listes déroulantes, sélections multiples, calendriers et champs typés.
- Signale une erreur de compilation explicite lorsque `SLINT_LIVE_PREVIEW` est définie, au lieu des globals partagés introuvables du générateur live-preview de Slint 1.18.1 (slint-ui/slint#14812). Supprimez cette variable, y compris dans `build.rs` ; `slint/mcp` et `SLINT_EMIT_DEBUG_INFO` restent utilisables. Aucun changement d’API publique.

- Présente les usages UI Rust de bureau et mobiles sur GitHub, crates.io, docs.rs et le site bilingue. Documente la configuration Android/iOS, les sélecteurs de fichiers réservés au bureau et les limites de qualification ; précise les preuves de qualité et vérifie l’index généré pour les assistants IA.

- L’exigence Slint passe de `=1.18.1` à `~1.18.1` : les applications peuvent utiliser toute version corrective de Slint 1.18 à partir de 1.18.1. Gardez `slint-kit` et `slint-kit-build` sur la même version. Aucun changement d’API publique.
- Le site de documentation publie `/llms.txt`, généré à partir de l’inventaire des composants, pour les assistants IA et les outils de recherche.
- Le README, la page crates.io, la documentation docs.rs et les métadonnées des crates présentent le catalogue, des captures, les garanties d’ingénierie et les plans de licence.

## 0.7.1

- Fix Windows source imports generated by `slint_kit_build::compile` and `compile_with_config`: canonical paths are written as Slint import paths without Rust Debug escaping. Preserve application library aliases, include paths and automatic window service exports. No public API changes.

### Français

- Corrige les imports Windows générés par `slint_kit_build::compile` et `compile_with_config` : les chemins canonicalisés sont écrits comme des chemins d’import Slint, sans échappement Debug Rust. Les alias de bibliothèques, chemins d’inclusion et exports automatiques des services de fenêtre sont conservés. Aucun changement d’API publique.

## 0.7.0

- Documentation uses one shared browser bundle for sequential component captures, including localized fixtures and dark posters. A visible light/dark selector keeps the page, poster and demo startup consistent; the typed example uses `Choices` and shared field conversions.

- Add `KitSelectKeyboard.move-active` and `type-ahead` for custom navigation surfaces sharing one selection session. / Ajout des méthodes de navigation et de recherche par frappe pour les surfaces composées partageant une session de sélection.

- **Breaking integration change:** shared Slint contracts are now compiled once in the kit using Slint 1.18.1's experimental module build. Import components/types/globals from `@kit`, build through `slint_kit_build::compile`, and keep the Cargo runtime dependency named `slint-kit`. Application-generated copies of kit types and imports through family files are no longer interchangeable with the canonical `slint_kit::ui` types. Supplemental icon and font imports retain their paths.
- Call `slint_kit::install(&window)` once before showing each window. It installs HTML validation and the calendar, field, selection and time services. The `init_form_validation!` and `bind_select!` macros are documented as deprecated compatibility APIs for one migration cycle.
- **Behavior change:** checkbox, switch, collapse and pagination accept local interaction state by default. Use `Ownership.controlled` when the host must acknowledge changes. Autonomous selectors, fields and temporal controls likewise expose explicit local/controlled ownership instead of requiring per-instance window callback names.
- Calendars and date/range pickers own navigation, focused dates, bounds and projections, and emit `chosen` values. Multiple instances and windows remain independent. `Choices<K>` centralizes stable tokens, filtered selection and generation checks, including independent sources and multi-selection through `selections(&[K])`; radio/segmented arrows choose locally while disabled/read-only states retain their documented restrictions.
- `FieldState`, `FieldSpec<T,E>` and typed `FieldCommit` centralize native draft/commit/blur/cancel handling and HTML constraints. Custom codecs are installed through `install_field_specs`; Rust acceptance checks remain required. Native caret, selection, undo and IME stay with Slint's editor; headless checks do not qualify OS IME or a screen reader.
- `KeyedModel` supports stable-key sorting (`sort_by`/`sorting_by`) and active editor projections (`active_row`/`bind_active`/`update_active`).
- Add `KeyedModel<K,Row>`, `ToastBinding<T>`, weak-window `action`/`event`/`event2`/`event3` handlers and native file/directory bindings. `native-winit` explicitly enables the winit file-drop bridge; unsupported, cancelled and failed requests remain distinct. Decimal sliders keep exact domain values and use floats only for renderer projection.
- Every component quickstart now uses at most 15 Rust source lines, with no weak-window plumbing or UI-enum conversion. Component pages show Slint and business callbacks and link to bilingual Project setup; complete downloads retain Cargo, build and lock files. A source guard in `xtask check` enforces this contract. Use matching 0.7 runtime/build packages; source dependencies remain available for local development.

### Français

- La documentation utilise un bundle navigateur commun pour capturer successivement les composants, leurs fixtures localisées et leurs posters sombres. Un sélecteur jour/nuit visible synchronise page, poster et démarrage de la démo ; l’exemple typé utilise `Choices` et les conversions partagées des champs.

- **Changement d’intégration cassant :** les contrats Slint sont compilés une seule fois dans le kit avec les modules expérimentaux de Slint 1.18.1. Importez composants/types/globals depuis `@kit`, compilez avec `slint_kit_build::compile` et gardez la dépendance Cargo nommée `slint-kit`. Les copies de types générées dans l’application et les imports de modules de famille ne sont plus interchangeables avec `slint_kit::ui`. Les imports supplémentaires de polices/icônes gardent leurs chemins.
- Appelez `slint_kit::install(&window)` une fois avant l’affichage de chaque fenêtre. La validation HTML et les services calendrier/champ/sélection/temps sont installés ensemble. `init_form_validation!` et `bind_select!` restent des API de compatibilité, documentées comme dépréciées pour un cycle de migration.
- **Changement de comportement :** checkbox, switch, collapse et pagination acceptent leur état local par défaut. Utilisez `Ownership.controlled` pour attendre l’acquittement de l’hôte. Sélecteurs, champs et contrôles temporels autonomes exposent leur propriété d’état au lieu d’imposer des noms de callbacks par instance sur la fenêtre.
- Calendriers et sélecteurs de date/période possèdent navigation, focus, bornes et projections, et émettent `chosen`. Plusieurs instances et fenêtres restent indépendantes. `Choices<K>` centralise jetons stables, filtrage et générations, avec des sources indépendantes et `selections(&[K])` pour la sélection multiple ; les flèches des groupes radio/segments sélectionnent localement, avec les restrictions lecture seule/désactivé documentées.
- `FieldState`, `FieldSpec<T,E>` et `FieldCommit` centralisent brouillon natif, validation, commit, blur et annulation. Les codecs personnalisés passent par `install_field_specs` ; l’acceptation Rust garde ses contraintes. Caret, sélection, undo et IME restent natifs ; les contrôles headless ne qualifient ni l’IME système ni un lecteur d’écran.
- `KeyedModel` fournit le tri par clé stable (`sort_by`/`sorting_by`) et les projections de l’éditeur actif (`active_row`/`bind_active`/`update_active`).
- Ajout de `KeyedModel<K,Row>`, `ToastBinding<T>`, des callbacks faibles `action`/`event`/`event2`/`event3` et des bindings natifs fichiers/répertoire. `native-winit` active explicitement le pont de dépôt winit ; absence de support, annulation et échec restent distincts. Les curseurs décimaux gardent des valeurs exactes, projetées en flottants seulement pour le rendu.
- Chaque quickstart tient en 15 lignes de source Rust, sans plomberie de fenêtre faible ni conversion d’enum UI. Les fiches montrent Slint et les callbacks métier et pointent vers Configurer le projet en deux langues ; les téléchargements complets gardent Cargo, build et verrouillage. Le gate `xtask check` vérifie ce contrat. Utilisez des paquets runtime/build 0.7 correspondants ; les dépendances sources restent possibles en développement local.

## 0.6.1

No public signatures were removed and `ChoiceRow` is unchanged.

- **OTP behavior change:** `KitOtpField` now uses `TextLengthPolicy.limit` to refuse edits exceeding `max-length` (default `slots`) in the actual native draft. Overlong paste and selection replacements are refused in full; existing text is retained. UTF-16 lengths never split a Unicode scalar. Native undo precedes restoration to avoid stale history offsets in Slint 1.18.1; refusal can discard undo/redo history and collapse selection. Edits within the limit retain native history. Strict input waits for `init_form_validation!`. `TextLengthPolicy.validate` restores validation-only behavior and remains the `KitTextField` default.

- **Geometry change:** themed scroll viewports reserve stable scrollbar gutters by default, including hidden `as-needed` bars. `visible-width` and `visible-height` exclude the gutters; fit content to those dimensions. `KitScrollArea.overlay-scrollbars: true` restores overlay behavior.
- Add opt-in `KitScrollArea.pan-policy: children-first` (`ScrollPanPolicy`) so nested drag controls retain gestures. Background/text drags pan directly without inertia; wheel, bars and focus reveal remain available. Default native arbitration is unchanged.
- Dismiss dialogs on the first backdrop click or tap without leaking the press to background controls; preserve panel clicks and modal dismissal guards.
- Ignore native keyboard events on disabled `KitSelectKeyboard`, including after focused controls become disabled. Preserve explicit `handle-key` delegation.

- `KitSegmentedControl` gains token-keyed `ChoiceIcon` glyphs through `icons` and configurable horizontal `columns`, including 2×2. Selected focus uses contrasting on-accent ink; read-only selection uses neutral colors. Keyboard activation rejects absent, disabled and duplicate active tokens and fails closed before the required window initialization. The Rust `choice_token_enabled` projection helper exposes the same uniqueness/eligibility check for custom controls.
- `KitButton` keeps its label centered with a leading or trailing icon, reserving equal side space in intrinsic sizing.
- `KitSlider` honors explicit track, fill and handle brushes when disabled; uncustomized defaults still use disabled theme colors.
- `KitSwitch` renders read-only tracks with neutral colors and a visible edge while preserving its checked position and keyboard inspection.
- `KitKeypad` allows rectangular keys within 320px at 200% text and adds `activation-started()` for feedback before pointer/Space commit. Existing action callbacks keep their timing; a cancelled press may produce feedback without an action.

## 0.6.0

0.6.0 adds home-automation primitives and changes the `Tone` and `KitIconSet` contracts; the compiler reports every site that must change.

- **Breaking:** `Tone` gains `information`. `KitBadge` and the new `KitChipButton` map it to the `information-subtle` fill and `information` edge. Rust code that matches `Tone` exhaustively must add the `Tone::Information` arm; the compiler reports each site.
- **Breaking:** `KitIconSet` gains `delete`, the backspace glyph of the keypad's delete key. The shipped themes fill it with Lucide `delete`; a hand-built theme must add it.
- `KitCheckbox`, `KitSwitch`, `KitRadioGroup`, `KitSegmentedControl` and `KitToggleGroup` now set `accessible-checkable`, so native screen readers announce their checked state; before, the AccessKit adapter dropped it.
- Add `KitChipButton`, a pressable status, filter or shortcut chip with a label, an optional `glyph: IconGlyph`, a `tone: Tone` and a host-owned `selected` state. A selected chip takes its tone's subtle fill and edge, a strong edge and a semibold label, so selection never rests on colour alone. `token` and `generation` identify the row: `activate(int, int)` reports the identity the press started on, and a press is cancelled when either changes. A `checkable` chip (the default) is announced as pressed or not pressed; a shortcut chip clears `checkable`. A static chip may handle the inherited `activated()` instead, which replaces the identity report. The chip follows `control-size`; a touch host raises `min-height` to its target.
- Add `direction: ChoiceDirection` and `presentation: SliderPresentation` (`line`, `filled`) to `KitSlider`, named like the direction of the radio group, toggle group and segmented control. A vertical slider puts the minimum at the bottom; Up and Right step forward, Down and Left step back. A filled slider paints the whole control as its track with a strong edge and a bar handle, and its minimum thickness is twelve spacing units in either direction. Its default paints (a solid accent fill on the pressed track, an on-accent handle) keep the value at least 3:1 against the track and the page in both themes. `track-brush`, `fill-brush` and `handle-brush` accept injected paints, such as a hue or color-temperature gradient. `accessible-orientation` follows the direction.
- Add `direction: ChoiceDirection` to `KitSegmentedControl`. Vertical segments stack in source order and navigate with Up and Down; `flow` applies only to the horizontal direction.
- Add `KitKeypad`, a numeric keypad (1 to 9, then delete, 0 and confirm) that emits `digit(int)`, `delete-requested()` (remove the last digit) and `confirm()` and never holds the entered digits. While `access` is editable, typed 0 to 9 and Backspace reach the same callbacks whenever a key or the keypad has focus. Return confirms on the keypad; Return and Space first activate a key that shows keyboard focus, as on any button. A key pressed with the pointer hands focus back to the keypad without a ring, so Return after tapped digits confirms. `can-confirm` gates confirm, and a held Return confirms once. Each key is at least twelve spacing units square and grows with enlarged text. `digit-labels`, `delete-label`, `delete-glyph` (by default `theme.icons.delete`), `confirm-label`, `confirm-glyph` and `confirm-intent` customize the keys.
- Add `KitCodeIndicator`, one mark per expected digit with the first `filled` marks filled and a host-supplied accessible `label`; it shows how many digits are entered, never which.

## 0.5.1

- Document project-relative `@kit` library paths for VS Code and Zed, including locked Cargo vendoring for crates.io consumers, local checkout paths and refresh instructions.

## 0.5.0

- Review all 124 public components against shared design, UX, accessibility and alignment criteria; unify showcase typography, spacing, navigation and control density.
- Center switches and radio marks beside enlarged labels, align labelled dividers and timeline markers, restore data-row hover/press feedback, and draw the toast dismiss icon.
- Match segmented-control visual order to RTL keyboard navigation. Search clearing emits the semantic value edit; RGB channel fields use the shared numeric constraints.
- Make the virtual-text editor footer intrinsic and responsive. Scale busy indicators with enlarged text, restore a stopped skeleton's opacity, and keep selected text legible in the teal demo theme.
- Preserve all OTP placeholder cells at enlarged text sizes and hide text/select placeholders during native preedit. This visual correction does not qualify IME behavior.
- Keep simple drawer content together below its heading and show a fallback icon for content items without a thumbnail. Clarify the AppShell and Section examples.
- Keep complete fields at their natural height in stretching layouts, expose the catalogue's selected page to accessibility, and enlarge custom status badges with text.
- Keep numeric step actions square beside the editor at 200% text. Add `interactive` to navigation composites so modal hosts can disable background focus and actions while preserving selection and data.
- Disabled and busy pressables reject semantic activation before emitting `activation-started`, matching their pointer and keyboard paths. Modal backdrops contain wheel events and the showcase disables its background navigation and actions.
- Keep long localized showcase action labels inside their reading column and expose their complete text through hover and keyboard help.

## 0.4.0

0.4.0 changes the theme and icon contracts. The Slint compiler reports every site that must change, so upgrade by following its errors. This list names each break.

### Portable code typography

- The default themes register the original JetBrains Mono 2.304 Regular font (SIL OFL 1.1) for `typography.family-monospace`, replacing the generic system family. Wide OTP letters and password mask glyphs retain the same advance in native and WebAssembly builds. Inter remains optional. Custom code fonts must be registered, monospaced and cover the code alphabet and mask glyphs.

### OTP patterns and native masking

- Add `OtpRules` for fixed-length code acceptance with the shared `HtmlPattern` engine. It uses complete-draft patterns and UTF-16 length units. `OtpValue::parse` and `OtpAlphabet` remain available.
- **Validation change:** `KitOtpField` now defaults to the digits pattern `[0-9]*` and minimum/maximum lengths matching `slots`. Set `pattern` explicitly for existing non-numeric codes. Lengths validate rather than truncate: Slint 1.18.1 has no public before-insert hook that preserves all native editing paths.
- Masked native fields no longer expose their raw value through the kit accessibility override. Examples include an empty underlined field, regex-validated alphanumeric codes and a native masked read-only view, with acceptance in Rust. An independent seven-character code stores `FG76657` and projects `FG‑76657` through inherited `display-text` only at rest; editing and paste use the raw code. Editable masking is not qualified: Slint 1.18.1 can panic on undo after a shorter selection replacement.

### Component polish and representative examples

- Add `AvatarInteraction.static`/`.action`, `KitAvatar.activated`, `disabled`, `has-focus` and `take-focus()`. Portraits now scale to their actual bounds; initial avatars have a contrasted surface and action states. The example bundles an original fictional portrait and demonstrates image, initials and shape variants.
- Add `FieldDecoration.boxed`/`.underlined` and `underline-segments` to native text fields and `KitInputGroup`; `KitOtpField.slots` controls its visual six-slot layout and default length validation while Rust remains responsible for acceptance. Native editing stays in one `TextInput`; `text-font-family`, `text-inset` and measured `text-cell-width` let the OTP align partial native drafts with their first slot. **Theme contract:** `KitTypography.family-monospace` supplies the code font in both shipped themes.
- Add optional decorative `leading-icon` to badges and stats, and semantic `tone` to stats. Improve semantic badge fills, steps, file rows and loading placeholders; skeletons remain visible on the light canvas.
- Center checkbox indicators beside wrapped labels and content-list thumbnails beside copy. Color-picker popup actions fit the intrinsically measured content at enlarged text sizes.
- Strengthen active control boundaries to at least 3:1 against both shipped canvases and surfaces, including the empty checkbox in dark mode.
- Choice groups track keyboard versus pointer focus, removing stale rings after clicks while retaining keyboard indicators. Tooltip examples use `focus-visible`; hover and keyboard help no longer draw simultaneously. Dialog examples open on their heading and preserve normal Tab navigation.
- Progress examples start at 65%, offer an explicit Animate/Pause action and respect reduced motion. Radial progress uses a continuous rounded arc. Reset buttons default to a visible secondary surface.
- Expand catalogue examples with meaningful collections, original images, native multiline text, clear multi-selection and composed public Kit controls. Selected multi-select choices leave the available list until their chip is removed.
- Component previews support light and dark presentation without losing the current draft or controller selection. Downloadable examples expose the same per-window `KitThemeContext` and explicit child theme bindings.

### Calendar formats and availability

- Add validated `DatePattern` for explicit Gregorian date formats, including compact, ordinal and ISO week dates, plus `DateNames` for host-localized month/weekday names and a French preset. Ambiguous adjacent variable-width numbers, two-digit years and time/time-zone directives are rejected. HTML date fields retain their canonical ISO value.
- Add `DateConstraints` for inclusive bounds, unavailable weekdays and blackout ranges; `DateRangeRules` validates the entire period and optional night bounds. `DateSetController` manages multiple dates by `CivilDate`, and `grid_for_month` supports composing several visible months.
- **Breaking for exhaustive matches:** `WeekStart` gains Tuesday through Saturday. `CalendarController::new` clears an initial selection outside its bounds and displays the clamped focused month; `show_month` keeps focus visible. Month/year navigation buttons now report whether the displayed month can actually change.
- Calendar wrappers expose localized weekday/navigation/range labels and previous/next availability. Read-only calendars remain browsable without accepting a date. Picker popovers reserve seven full columns at 200% text.
- Booking examples share their availability rules between typing and calendar choice, preserve rejected range drafts, and demonstrate Apply/Cancel, localized formats, Lucide icons and composed public cards.

### Built-in presentation and product composition

- **Breaking:** Tailwind is the single built-in presentation through `KitThemes.light` and `.dark`. Components use it by default through the existing `@kit` imports and `slint-kit-build` integration; the separate theme package and Base/Tailwind selectors are removed.
- **Breaking:** `KitTheme.icons: KitIconSet` is required in complete themes. Built-in controls use Lucide roles. Inter is separately registered by `InterFont` from `@kit/fonts/inter.slint`; set the Window’s `default-font-family: InterFont.family`.
- Buttons gain typed leading/trailing icons and recipe-derived local radius, padding and typography. Icon buttons honor all intents and reserve their geometry while busy. Fields gain icon affixes and local geometry/typography while preserving native editors, validation and access states. Cards and surfaces gain local radius/elevation; alerts gain local radius/inset.
- Tailwind medium controls now actually fit a 36 px line at normal text scale; body/subtitle hierarchy uses 14/18 px. The built-in theme supplies distinct semantic icons, subtle selected rows, a slate workspace canvas, light borders, elevation and optional Inter.
- Correct theme scope documentation: the global is per window/component tree; supplied children require explicit theme bindings. There is no implicit ancestor cascade.
- The showcase adds reusable account, project collection and editing dialog compositions, with filtered host models, stable identity, in-memory persistence, guarded submissions and cancellation/focus restoration. Download the same Slint/Rust sources and run the WASM demonstration at `/examples/workspace/`.
- Incoming chat bubbles use a bordered theme surface so messages remain visible on both the canvas and same-color panels.
- Correct leading/trailing icon placement inside padded buttons, including navigation actions and locally customized padding.
- Documentation gains a fuzzy search across guides and component references, representative chat/list examples and corrected canvas mockups; migration-only pages are removed.
- Fix a visible gap at the center of an unlabelled divider.

### Theme

- **`KitTheme`** grows typed token groups: `shape` (`KitShape`: radii by role, border width, icon stroke), `elevation` (`KitElevation` of `KitShadow { key, ambient }`), `focus` (`KitFocusRing`), `typography` (`KitTypography`), `spacing` (`KitSpacing` with per-`ControlSize` `KitControlDensity`) and `motion` (`KitMotion`). It also gains these colors: `placeholder`, `accent-hover`, `accent-pressed`, `accent-subtle`, `selection`, `on-selection`, `danger-hover`, `danger-pressed`, `danger-subtle`, `success-subtle`, `warning-subtle`, `information`, `information-subtle` and `border-strong`. Write a custom theme as a complete literal: Slint sets an omitted field to zero or transparent, without a warning.
- The flat **`focus-ring`** color is removed; use `focus.color`.
- **`KitThemeContext`** is a new global. Every component's `theme` now defaults to `KitThemeContext.current` instead of `KitThemes.light`. Assign it once, or from Rust after re-exporting `KitThemeContext`, so a component you did not pass a theme no longer renders light in a dark application.
- **`KitMetrics`**: `label-gap`, `help-gap`, `field-gap`, `section-gap` and `footer-gap` become functions of the theme. `font-size(role, scale)` becomes `font-size(theme, role)`, and `control-height(size, scale)` becomes `control-height(theme, size)`. `KitLayoutMetrics.space` now takes the theme. New: `density`, `inline-gap`, `row-height` and `icon-size`.
- **`ButtonRecipe`**: `focus-color`, `focus-width` and `focus-offset` are replaced by `focus: KitFocusRing`. It also gains `busy-mark-size`, `busy-mark-stroke`, `font-weight` (button labels use `typography.weight-medium`) and `elevation` with its `shadow` layer: primary, secondary and destructive buttons lift by `elevation.control` (`xs` in the built-in light and dark modes), painted by a child so the shadow never changes a button's measured height.
- **`OptionRecipe`** drops `active-border` and `border-width` and gains `pressed-fill`, `selected-fill` and `selected-foreground`. Active and selected rows of lists, menus and tabs take the `accent-subtle` tint, and the active row paints the theme's inset `FocusOutline` instead of a 1 px border. Menu and navigation rows read the recipe's label color and size; `KitOption` now uses the quiet button geometry.
- **`KitThemes.choice-recipe(theme)`** becomes **`choice-recipe(theme, size)`**: checkbox and radio boxes are 16, 20 or 24 px by `ControlSize`, and switch, slider and marks scale with them; the box corner is at most a quarter of the box, and choice groups nest `radius-control` inside their padding (11 px in the default themes, was 10). `KitSlider` and `KitRangeSlider` gain `control-size`.
- Field labels, file and folder picker labels and data-table headers use `typography.weight-medium` instead of semibold. `KitIconButton` uses `radius-control` (was 6 px); the data table reads its surface recipe (container radius and shadow); badges, skeletons and tooltips use `radius-control`, the hero the container radius, and tooltips the popover shadow.
- **`KitThemes.dark`**: `surface-raised` is slate-900 (was slate-800), so hover, pressed and `border-subtle` stay distinct on popovers and dialogs. `on-accent` is slate-950 (was indigo-950), raising labels on the indigo-400 accent from 5.4:1 to 6.8:1.
- **`ButtonIntent.outline`** is added between `secondary` and `quiet`: a transparent fill with the strong border. Rust code matching `ButtonIntent` exhaustively must handle it.
- `SurfaceLevel` (now with `overlay` and `popover`) and `AlertSeverity` are declared in `theme.slint`. They are still re-exported from their former files.
- Every component family reads recipes from `KitThemes` (button, field, trigger, choice, option, surface, alert, table row, scroll bar, icon) instead of literals. `KitDialogPanel` and `KitSelectSurface` paint a stacked shadow under a transparent host, so a `background` set on the instance is now hidden; `border-radius` still applies.
- **`TooltipRecipe`** and **`KitThemes.tooltip-recipe(theme)`** are new. A tooltip is now as wide as one line of its text, up to `max-width` (80 spacing units, 320 px at 100% text, scaled with `text-scale`), with `padding-inline` and `padding-block` of 3 and 1.5 spacing units. It used to take a fixed 300 px width (600 px at 200% text) with 8 px padding on every side.
- Inter 4.1 is available in four optional static weights through `@kit/fonts/inter.slint`. Importing the components does not embed the fonts.

### Icons

- **`IconKind` is removed**, along with its single ternary chain. Glyphs are `IconGlyph { stroke, fill, viewbox }` values returned by named functions: the kit's 28 original glyphs are in **`KitGlyphs`** (`IconKind.chevron-down` becomes `KitGlyphs.chevron-down()`), and all 1,999 Lucide 1.64.0 icons are in `@kit/icons/lucide/all.slint` (`LucideIcons`) and in one file per category. An icon you do not call adds nothing to the binary.
- **`KitIcon`**: `kind` becomes `glyph`, which has no default. `KitIcon` now inherits `Rectangle` instead of `Path`, so the `Path` properties are no longer public. New properties: `control-size` (14, 16 or 20 px, multiplied by `text-scale`), `purpose: IconPurpose` with `label` (an informative icon is announced as an image), `stroke-mode: IconStroke` (`scaled` or `absolute`), and the read-only `stroke-width`.
- **`KitIconButton.icon`** becomes **`glyph: IconGlyph`**. Icons inside controls follow their host's `control-size`: icon buttons, field actions, select and combobox chevrons and checks, stepper buttons, disclosure, breadcrumb and notification icons. A field's clear or reveal button grows with its icon, so it still fits at 200% text.
- An information `KitAlert` draws `KitGlyphs.information()` instead of the plus glyph.
- `IconRecipe` becomes `{ size, color, disabled-color, stroke }`. `KitThemes.icon-recipe` takes a `ControlSize`, and the new `KitThemes.icon-stroke-width(recipe, glyph, mode, extent)` computes the stroke.
- The kit glyphs move to Lucide's 24-unit grid, with round caps and joins and the theme's `shape.icon-stroke` token (2). At 16 px they now draw a 1.33 px stroke, where they drew 1.7 px before.
- Selects, comboboxes, disclosure rows and sorted table headers turn their chevron over `motion.fast`, or instantly with `reduced-motion`. At rest each draws the glyph of its state unrotated, so renderers that do not rotate paths still point the right way.
- **`KitChip`** inherits `KitPressable` instead of `KitButton`: it no longer has a `text` property, and it marks removal with the close glyph instead of a `" ×"` suffix.

### Controls

- **`KitSelect.disabled: bool`** becomes **`access: FieldAccess`**, and `KitSelect` gains `validation`. A read-only select keeps focus but never opens. `KitSelectTrigger` gains `access` and `validation` and no longer reads `intent`.
- **`KitScrollBar`** is new: a themed scroll bar that replaces the std-widgets bars inside the kit. **`KitScrollArea`** now inherits `Rectangle`. It keeps `theme`, `gap`, `inset`, `enabled`, `vertical-scrollbar-policy`, `mouse-drag-pan-enabled`, the `visible-*` and content sizes, `content-y`, `scrolled`, `reveal()` and `scroll-to-start()`. The rest of the std `ScrollView` API is gone (`content-x`, the horizontal policy and the deprecated `viewport-*`).
- `KitPressable` gains `focus-indicated` and `focus-ring`, and `take-focus()` does nothing while it is disabled. `KitInputGroup` gains `out hovered`.
- New: `theme` on `KitField`, `KitFieldGroup`, `KitInputGroupStart`/`End`, `KitContainer`, `KitStack`, `KitInline`, `KitWrap`, `KitGrid`, `KitScrollArea`, `KitButtonGroup`, `KitCardBody` and `KitCardFooter`; `palette` on `KitColorField`; `failed-label` on `KitFilePicker` and `KitDirectoryPicker`.
- Segmented-control and toggle-group labels use the button recipe's `font-weight` (medium). Disabled checkboxes, radios and switches draw their mark's edge in `disabled-text`, so the mark stays visible on the disabled row fill; it used to vanish wherever `border-subtle` equalled `disabled-fill`, as in `KitThemes.dark`.
- `KitDataTable` header cells use the flat, square table-row recipe and start their label at the same inset as the cells below. A column with `sortable: false` keeps the header tone; it used to paint the disabled fill with rounded corners.
- States that used to render nothing now have a look: read-only, hover, checking (with a spinner) and invalid fields, distinct alert tones, the selected tab's underline, the selected palette swatch and a disabled drop zone.

### Build and notices

- `rust-toolchain.toml` pins Rust **1.99.1** for development and CI. The supported minimum (`rust-version`) stays 1.92, and Slint stays at `=1.18.1`.
- `THIRD-PARTY-NOTICES.md` (Tailwind CSS MIT, Inter OFL 1.1, Lucide ISC and Feather MIT) ships with `slint-kit` and `slint-kit-slint`.
- `cargo run -p xtask -- generate-icons` regenerates the Lucide files from `vendor/lucide/<version>/`. `xtask check` rejects stale icon files and, through `lint-tokens`, any literal in the kit sources that is not a theme token or listed in `xtask/token-allowlist.txt`.
