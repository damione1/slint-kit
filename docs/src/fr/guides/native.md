---
layout: base.njk
locale: fr
title: "Intégration native"
translationKey: "guides/native/"
permalink: "{{ 'guides/native/' | localeUrl(locale) }}index.html"
---

# Intégration native

Activez explicitement les services du système. Le domaine du cœur et les composants visuels de fichiers ne nécessitent pas de dépendance de sélecteur natif. Ajoutez la fonctionnalité `native` dans l’application qui en a besoin :

```toml
slint-kit = { version = "={{ project.version }}", features = ["native"] }
```

## Bindings typés des sélecteurs

Utilisez `FilePickerBinding<W>` ou `DirectoryPickerBinding<W>` pour le flux ordinaire du composant. Leurs ports explicites désignent les propriétés de l’application sans convention de noms sur la racine ; le binding gère référence faible, requêtes asynchrones et générations. `on_selection` reçoit les ressources acceptées.

```rust
let picker = slint_kit::FilePickerBinding::new(&app, slint_kit::FilePickerPorts {
    entries: App::set_entries, state: App::set_request_state, error: App::set_error,
});
picker.set_mode(slint_kit::FileSelectionMode::Single);
picker.set_access(slint_kit::FieldAccess::Editable);
picker.on_selection(|files| println!("{} ressources sélectionnées", files.len()));
app.on_browse(picker.browse_handler());
app.on_remove(picker.remove_handler());
```

Initialisez `slint_kit::install(&app)` avant l’affichage. Consultez les consommateurs complets [sélecteur de fichiers](/fr/components/file-picker/) et [sélecteur de répertoire](/fr/components/directory-picker/). L’annulation conserve les ressources acceptées ; une fenêtre détruite ou une requête remplacée ne reçoit aucun résultat périmé.

## Requêtes et résultats

`slint_kit::native::pick_files(FileRequest)` et `pick_directory(DirectoryRequest)` sont asynchrones. Construisez et exécutez le sélecteur sur l’exécuteur de l’interface, par exemple `slint::spawn_local`. Revenez à une référence faible de fenêtre pour publier le résultat. Gardez la boucle d’événements disponible pendant l’attente du dialogue.

```rust
use slint_kit::{FileRequest, FileSelectionMode, SelectionOutcome};

async fn choose_document() -> SelectionOutcome<Vec<slint_kit::FileHandle>> {
    slint_kit::native::pick_files(FileRequest {
        selection: FileSelectionMode::Single,
        filters: Vec::new(),
        suggested_name: None,
        ..Default::default()
    }).await
}
```

Traitez chaque variante de `SelectionOutcome` : `Selected(value)`, `Cancelled` ou `Failed(HostError)`. Une annulation préserve la sélection acceptée précédente. `HostError::Unsupported` exprime une capacité indisponible, distincte d’une sélection vide réussie. Une API native incapable de distinguer échec et annulation n’invente pas d’erreur de permission.

`FileFilter::new(label, extensions)` valide le filtre. `FileHandle` et `DirectoryHandle` exposent un nom et une `FileLocation` ; cette dernière vaut `Path` ou `Uri`. Choisir une ressource ne lit pas son contenu et ne garantit pas un chemin de système de fichiers. La taille peut être inconnue.

## Capacités et requêtes en cours

Consultez `slint_kit::native::capabilities()` et projetez ses résultats typés dans [KitFilePicker](/fr/components/file-picker/), [KitDirectoryPicker](/fr/components/directory-picker/) et [KitFileDropZone](/fr/components/file-drop-zone/). Le sélecteur et le dépôt externe sont deux capacités distinctes. L’adaptateur natif générique déclare le dépôt non pris en charge. Pour une fenêtre winit, activez `native-winit` et appelez `FilePickerBinding::install_winit_drop` avec les ports de capacité et de survol. Un DropArea Slint ne prouve ni le support système du dépôt ni la qualification d’un périphérique.

`HostRequestController<T>` émet des tickets pour rejeter les réponses asynchrones obsolètes. Présentez explicitement attente, échec et annulation. Consultez le [contrôleur de fichiers du showcase](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_files.rs) et l’[adaptateur système](https://github.com/damione1/slint-kit/blob/main/crates/kit-native/src/lib.rs) pour le comportement par plateforme.

Synchronisez `set_mode` et `set_access` avec le sélecteur lors de leurs changements. Les dépôts natifs appliquent ces règles même via le pont winit de la fenêtre : sélection simple conserve le dernier fichier déposé, sélection multiple ajoute les emplacements uniques, lecture seule et désactivation ignorent le dépôt.

## Textes intégrés localisés

Définissez `KitStrings` une fois pour la fenêtre afin de traduire parcours, suppression, états des sélecteurs et états des étapes. Les propriétés publiques restent remplaçables par instance ; les modèles de suppression acceptent `{action}` et `{name}` dans l’ordre voulu.

`FilePickerBinding::set_error_mapper` et `DirectoryPickerBinding::set_error_mapper` reçoivent le `HostError` typé et retournent un `SharedString` localisé. `HostError::Platform(detail)` conserve le détail système, y compris avec le mapper par défaut. L’absence de capacité passe par l’état de requête et n’ajoute jamais d’erreur de champ, même avec un mapper personnalisé.

Les bindings fournissent la fenêtre Slint comme parent de la boîte de dialogue native. Une intégration directe utilise pick_files_with_parent ou pick_directory_with_parent avec un fournisseur raw-window-handle. Affichez d’abord la fenêtre ; le support dépend du système/backend (macOS, Windows et Linux XDG). Les handles sont lus avant de retourner le futur pour préserver la durée de vie faible ; le comportement modal natif reste à qualifier sur la cible.

`KitFileDropZone` sert aussi au parcours : raccordez `browse-requested` au même binding que le bouton du sélecteur. Clic/Entrée/Espace/action d’accessibilité fonctionnent indépendamment de la capacité de dépôt système. Définissez `description` et `leading-icon`, ou composez `@children` ; lecture seule et désactivation bloquent les deux chemins.

## Options des requêtes

`FileRequest` porte les filtres validés, le titre, le dossier initial et un nom suggéré facultatif. `DirectoryRequest` porte le titre et le dossier initial. Dans chaque binding, `set_request` configure les prochaines demandes, `request` retourne la configuration typée et `browse_with` la configure puis l’ouvre. Le callback fichier actualise encore le mode de sélection depuis le composant, en conservant les autres options. Elles sont capturées avant le lancement asynchrone : modifier la configuration ne modifie pas une boîte ouverte. Les backends natifs peuvent interpréter différemment le dossier initial ; ces options n’accordent aucun accès au système de fichiers.

```rust
picker.set_request(slint_kit::FileRequest {
    filters: vec![slint_kit::FileFilter::new("Documents".into(), vec!["pdf".into()])?],
    title: Some("Joindre un document".into()),
    directory: Some(std::path::PathBuf::from("/home/me/Documents")),
    ..Default::default()
});
```


## Destinations d’enregistrement, métadonnées et contraintes de fichiers

`SaveFileRequest` est distinct d’un `FileRequest` d’ouverture et de `DirectoryRequest`. `native::save_file` ou `save_file_with_parent` choisit une destination ; le kit retourne `SelectionOutcome<FileHandle>` sans écrire le document ni créer son contenu. Filtres, nom suggéré, titre et dossier initial sont conservés. Vérifiez `capabilities().save_files` ; une cible non prise en charge retourne `Failed(HostError::Unsupported)`. Utilisez les gardes existantes de fenêtre faible et de ticket `HostRequestController` pour publier le résultat d’une opération brute d’enregistrement.

L’ouverture et `native::inspect_file_paths` obtiennent les métadonnées de chemins sur un worker sans lire le contenu. Une taille connue devient `Some(u64)` ; une métadonnée indisponible reste `None`, y compris pour une ressource de fournisseur sans chemin. Le binding affiche les octets exacts par défaut. `set_detail_formatter` reçoit le handle typé pour traduire les unités ou indiquer une taille indisponible ; il actualise les lignes existantes sans remplacer leur identité stable.

Définissez les mêmes règles intrinsèques dans le composant et le binding. `KitFilePicker.accept` reçoit des extensions ou indications MIME séparées par virgules ; `max-files: -1` ne limite pas le nombre et `max-size: ""` ne limite pas les octets. Limites et `FileViewEntry.byte-length` sont des chaînes décimales à la frontière du renderer pour conserver tous les u64 ; une métadonnée vide signifie inconnue. Configurez `FileConstraints` une fois en Rust avant d’accepter une sélection native, un dépôt ou des ressources de fournisseur :

```rust
picker.set_constraints(slint_kit::FileConstraints {
    accept: slint_kit::FileAccept::parse(".pdf,image/*")?,
    max_files: Some(4),
    max_size: Some(10_000_000),
});
picker.set_detail_formatter(|file| file.byte_length.map_or_else(
    || "Taille indisponible".into(), |bytes| format!("{bytes} octets").into()));
```

La même frontière HTML partagée est disponible avec `HtmlConstraints::validate_file_selection`, qui combine obligation/accès/validité personnalisée et `FileConstraints` typées. Conservez cette vérification Rust à la soumission. La sélection doit respecter type, nombre et chaque taille connue ; une taille inconnue échoue explicitement si une limite est configurée. Un refus conserve les ressources acceptées précédemment. `set_constraint_mapper` traduit `FileViolation` typée ; `KitStrings.file-constraints` et `file-messages` localisent les erreurs du formulaire.

Les indications MIME utilisent la correspondance d’extensions épinglée, sans vérification du contenu ni de sécurité. Les indications connues produisent des filtres natifs quand la requête n’en fournit pas explicitement ; une correspondance inconnue retourne Unsupported. Les extensions suivent la grammaire validée de FileFilter, y compris les suffixes composés tels que `.tar.gz`. Voir [les limites documentées de la bibliothèque](https://docs.rs/mime_guess/2.0.5/mime_guess/). Le système de fichiers ou fournisseur reste responsable du contenu réel et des permissions.

`accept_drop` résout désormais les métadonnées de façon asynchrone avec les mêmes gardes de fenêtre faible, d’accès et de ticket que browse. Une suppression, un remplacement, un blocage ou une nouvelle requête clôture l’ancienne réponse. `accept_handles` reçoit des métadonnées déjà résolues par un fournisseur et applique les mêmes contraintes. Annulation native et résultats indisponible/unsupported restent distincts. Ces contrats et tests headless ne qualifient ni dialogue d’appareil ni lecteur d’écran.
