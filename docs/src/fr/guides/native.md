---
layout: base.njk
locale: fr
title: "Intégration native"
translationKey: "guides/native/"
permalink: /fr/guides/native/index.html
---

# Intégration native

Activez explicitement les services du système. Le domaine du cœur et les composants visuels de fichiers ne nécessitent pas de dépendance de sélecteur natif. Ajoutez la fonctionnalité `native` dans l’application qui en a besoin :

```toml
slint-kit = { version = "0.1", features = ["native"] }
```

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

Consultez `slint_kit::native::capabilities()` et projetez ses résultats typés dans [KitFilePicker](/fr/components/file-picker/), [KitDirectoryPicker](/fr/components/directory-picker/) et [KitFileDropZone](/fr/components/file-drop-zone/). Le sélecteur et le dépôt externe sont deux capacités distinctes. L’adaptateur natif fourni déclare le dépôt non pris en charge ; cette intégration doit être fournie séparément par le backend de fenêtre.

`HostRequestController<T>` émet des tickets pour rejeter les réponses asynchrones obsolètes. Présentez explicitement attente, échec et annulation. Consultez le [contrôleur de fichiers du showcase](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_files.rs) et l’[adaptateur système](https://github.com/damione1/slint-kit/blob/main/crates/kit-native/src/lib.rs) pour le comportement par plateforme.
