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
    }).await
}
```

Traitez chaque variante de `SelectionOutcome` : `Selected(value)`, `Cancelled` ou `Failed(HostError)`. Une annulation préserve la sélection acceptée précédente. `HostError::Unsupported` exprime une capacité indisponible, distincte d’une sélection vide réussie. Une API native incapable de distinguer échec et annulation n’invente pas d’erreur de permission.

`FileFilter::new(label, extensions)` valide le filtre. `FileHandle` et `DirectoryHandle` exposent un nom et une `FileLocation` ; cette dernière vaut `Path` ou `Uri`. Choisir une ressource ne lit pas son contenu et ne garantit pas un chemin de système de fichiers. La taille peut être inconnue.

## Capacités et requêtes en cours

Consultez `slint_kit::native::capabilities()` et projetez ses résultats typés dans [KitFilePicker](/fr/components/file-picker/), [KitDirectoryPicker](/fr/components/directory-picker/) et [KitFileDropZone](/fr/components/file-drop-zone/). Le sélecteur et le dépôt externe sont deux capacités distinctes. L’adaptateur natif générique déclare le dépôt non pris en charge. Pour une fenêtre winit, activez `native-winit` et appelez `FilePickerBinding::install_winit_drop` avec les ports de capacité et de survol. Un DropArea Slint ne prouve ni le support système du dépôt ni la qualification d’un périphérique.

`HostRequestController<T>` émet des tickets pour rejeter les réponses asynchrones obsolètes. Présentez explicitement attente, échec et annulation. Consultez le [contrôleur de fichiers du showcase](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_files.rs) et l’[adaptateur système](https://github.com/damione1/slint-kit/blob/main/crates/kit-native/src/lib.rs) pour le comportement par plateforme.
