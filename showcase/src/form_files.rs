// SPDX-License-Identifier: GPL-3.0-or-later
//! Window-local native resource requests and exact sRGB editing.
use super::ui;
use kit::{
    AlphaPolicy, DirectoryHandle, DirectoryRequest, DraftValue, FileHandle, FileLocation,
    FileRequest, HostCapability, HostError, HostRequestController, HostRequestTicket, NumberLocale,
    NumericBounds, SelectionOutcome, SrgbColor, parse_integer,
};
use slint::{
    ComponentHandle, ModelRc, VecModel,
    language::{DragAction, DropEvent},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileId(i32);
struct SelectedFile {
    id: FileId,
    handle: FileHandle,
}
struct Resources {
    files: Vec<SelectedFile>,
    generation: i32,
    next_file_id: i32,
    file_requests: HostRequestController<Vec<FileHandle>>,
    folder_requests: HostRequestController<DirectoryHandle>,
    folder: Option<DirectoryHandle>,
    file_state: ui::HostRequestPresentation,
    folder_state: ui::HostRequestPresentation,
    file_error: String,
    folder_error: String,
}
impl Resources {
    fn new() -> Self {
        let capabilities = kit::native::capabilities();
        Self {
            files: Vec::new(),
            generation: 1,
            next_file_id: 1,
            file_requests: HostRequestController::default(),
            folder_requests: HostRequestController::default(),
            folder: None,
            file_state: capability_presentation(capabilities.files),
            folder_state: capability_presentation(capabilities.directories),
            file_error: String::new(),
            folder_error: String::new(),
        }
    }
    fn replace_files(&mut self, files: Vec<FileHandle>) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("file view generation exhausted");
        self.files = files
            .into_iter()
            .map(|handle| {
                let id = FileId(self.next_file_id);
                self.next_file_id = self
                    .next_file_id
                    .checked_add(1)
                    .expect("file identity exhausted");
                SelectedFile { id, handle }
            })
            .collect();
    }
    fn finish_files(
        &mut self,
        ticket: HostRequestTicket,
        outcome: SelectionOutcome<Vec<FileHandle>>,
    ) {
        if !self.file_requests.finish(ticket, outcome) {
            return;
        }
        match self
            .file_requests
            .take_result()
            .expect("just completed request")
        {
            SelectionOutcome::Selected(files) => {
                self.replace_files(files);
                self.file_state = ui::HostRequestPresentation::Idle;
                self.file_error.clear();
            }
            SelectionOutcome::Cancelled => {
                self.file_state = ui::HostRequestPresentation::Cancelled;
                self.file_error.clear();
            }
            SelectionOutcome::Failed(error) => {
                self.file_state = error_presentation(&error);
                self.file_error = host_error(&error).into();
            }
        }
    }
    fn finish_folder(
        &mut self,
        ticket: HostRequestTicket,
        outcome: SelectionOutcome<DirectoryHandle>,
    ) {
        if !self.folder_requests.finish(ticket, outcome) {
            return;
        }
        match self
            .folder_requests
            .take_result()
            .expect("just completed request")
        {
            SelectionOutcome::Selected(folder) => {
                self.folder = Some(folder);
                self.folder_state = ui::HostRequestPresentation::Idle;
                self.folder_error.clear();
            }
            SelectionOutcome::Cancelled => {
                self.folder_state = ui::HostRequestPresentation::Cancelled;
                self.folder_error.clear();
            }
            SelectionOutcome::Failed(error) => {
                self.folder_state = error_presentation(&error);
                self.folder_error = host_error(&error).into();
            }
        }
    }
    fn project(&self, window: &ui::Showcase) {
        let forms = window.global::<ui::FormExamples>();
        forms.set_selected_files(ModelRc::new(VecModel::from(
            self.files
                .iter()
                .map(|file| ui::FileViewEntry {
                    token: file.id.0,
                    generation: self.generation,
                    name: file.handle.display_name.as_str().into(),
                    detail: file
                        .handle
                        .byte_length
                        .map_or_else(
                            || match &file.handle.location {
                                FileLocation::Path(_) => "Local file".into(),
                                FileLocation::Uri(_) => "Resource URI".into(),
                            },
                            |bytes| format!("{bytes} bytes"),
                        )
                        .into(),
                    removable: true,
                })
                .collect::<Vec<_>>(),
        )));
        forms.set_file_state(self.file_state);
        forms.set_folder_state(self.folder_state);
        forms.set_file_error(self.file_error.as_str().into());
        forms.set_folder_error(self.folder_error.as_str().into());
        forms.set_folder_name(
            self.folder
                .as_ref()
                .map_or("", |folder| folder.display_name.as_str())
                .into(),
        );
    }
}
fn capability_presentation(capability: HostCapability) -> ui::HostRequestPresentation {
    match capability {
        HostCapability::Available => ui::HostRequestPresentation::Idle,
        HostCapability::Unsupported => ui::HostRequestPresentation::Unsupported,
    }
}
fn error_presentation(error: &HostError) -> ui::HostRequestPresentation {
    match error {
        HostError::Unsupported => ui::HostRequestPresentation::Unsupported,
        HostError::PermissionDenied
        | HostError::Unavailable
        | HostError::InvalidResource
        | HostError::InvalidFilter
        | HostError::Platform(_) => ui::HostRequestPresentation::Failed,
    }
}
fn host_error(error: &HostError) -> &str {
    match error {
        HostError::Unsupported => "Selection is unavailable on this platform.",
        HostError::PermissionDenied => "Permission to select this resource was denied.",
        HostError::Unavailable => "The native selection service is unavailable.",
        HostError::InvalidResource => "The selected resource is invalid.",
        HostError::InvalidFilter => "The file filter is invalid.",
        HostError::Platform(message) => message,
    }
}

#[derive(Clone)]
struct ColorDrafts {
    hex: String,
    red: String,
    green: String,
    blue: String,
    alpha: String,
}
impl ColorDrafts {
    fn from_color(color: SrgbColor) -> Self {
        Self {
            hex: color.hex(AlphaPolicy::Editable),
            red: color.red.to_string(),
            green: color.green.to_string(),
            blue: color.blue.to_string(),
            alpha: color.alpha.to_string(),
        }
    }
    fn channel_mut(&mut self, channel: ui::ColorChannel) -> &mut String {
        match channel {
            ui::ColorChannel::Red => &mut self.red,
            ui::ColorChannel::Green => &mut self.green,
            ui::ColorChannel::Blue => &mut self.blue,
            ui::ColorChannel::Alpha => &mut self.alpha,
        }
    }
}
#[derive(Clone, Default)]
struct ColorChannelErrors {
    red: String,
    green: String,
    blue: String,
    alpha: String,
}
impl ColorChannelErrors {
    fn channel_mut(&mut self, channel: ui::ColorChannel) -> &mut String {
        match channel {
            ui::ColorChannel::Red => &mut self.red,
            ui::ColorChannel::Green => &mut self.green,
            ui::ColorChannel::Blue => &mut self.blue,
            ui::ColorChannel::Alpha => &mut self.alpha,
        }
    }
}
struct ColorSnapshot {
    channel: ui::ColorChannel,
    color: SrgbColor,
    drafts: ColorDrafts,
    hex_error: String,
    channel_errors: ColorChannelErrors,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorEditIntent {
    Edited,
    CommitRequested,
}
struct ColorEditor {
    accepted: SrgbColor,
    drafts: ColorDrafts,
    error: String,
    channel_errors: ColorChannelErrors,
    snapshot: Option<ColorSnapshot>,
}
impl ColorEditor {
    fn new() -> Self {
        let accepted = SrgbColor {
            red: 79,
            green: 70,
            blue: 229,
            alpha: 255,
        };
        Self {
            accepted,
            drafts: ColorDrafts::from_color(accepted),
            error: String::new(),
            channel_errors: ColorChannelErrors::default(),
            snapshot: None,
        }
    }
    fn channel(&self, channel: ui::ColorChannel) -> u8 {
        match channel {
            ui::ColorChannel::Red => self.accepted.red,
            ui::ColorChannel::Green => self.accepted.green,
            ui::ColorChannel::Blue => self.accepted.blue,
            ui::ColorChannel::Alpha => self.accepted.alpha,
        }
    }
    fn set_channel(&mut self, channel: ui::ColorChannel, value: u8) {
        match channel {
            ui::ColorChannel::Red => self.accepted.red = value,
            ui::ColorChannel::Green => self.accepted.green = value,
            ui::ColorChannel::Blue => self.accepted.blue = value,
            ui::ColorChannel::Alpha => self.accepted.alpha = value,
        }
        *self.drafts.channel_mut(channel) = value.to_string();
        self.drafts.hex = self.accepted.hex(AlphaPolicy::Editable);
        self.channel_errors.channel_mut(channel).clear();
        self.error.clear();
    }
    fn edit_hex(&mut self, text: String) {
        self.drafts.hex = text;
        self.error = match SrgbColor::parse_hex(&self.drafts.hex, AlphaPolicy::Editable) {
            DraftValue::Invalid(_) => "Use an sRGB hex color with alpha, such as #4F46E5FF.".into(),
            DraftValue::Empty | DraftValue::Incomplete | DraftValue::Parsed(_) => String::new(),
        };
    }
    fn commit_hex(&mut self, text: String) {
        self.drafts.hex = text;
        match SrgbColor::parse_hex(&self.drafts.hex, AlphaPolicy::Editable) {
            DraftValue::Parsed(color) => {
                self.accepted = color;
                self.drafts = ColorDrafts::from_color(color);
                self.channel_errors = ColorChannelErrors::default();
                self.error.clear();
            }
            DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => {
                self.error =
                    "Complete the sRGB color before accepting it (for example #4F46E5FF).".into()
            }
        }
    }
    fn edit_channel(&mut self, channel: ui::ColorChannel, text: String, intent: ColorEditIntent) {
        *self.drafts.channel_mut(channel) = text.clone();
        match parse_integer(
            &text,
            NumberLocale::EnUs,
            Some(NumericBounds::new(0, 255).expect("byte bounds")),
        ) {
            DraftValue::Parsed(value) => {
                if intent == ColorEditIntent::CommitRequested {
                    self.set_channel(channel, u8::try_from(value).expect("bounded byte"));
                } else {
                    self.channel_errors.channel_mut(channel).clear();
                }
            }
            DraftValue::Empty | DraftValue::Incomplete => {
                *self.channel_errors.channel_mut(channel) =
                    if intent == ColorEditIntent::CommitRequested {
                        "Enter a complete channel value from 0 to 255.".into()
                    } else {
                        String::new()
                    };
            }
            DraftValue::Invalid(_) => {
                *self.channel_errors.channel_mut(channel) =
                    "Each sRGB channel must be an integer from 0 to 255.".into()
            }
        }
    }
    fn navigate(&mut self, channel: ui::ColorChannel, step: ui::NavigationStep) {
        let value = self.channel(channel);
        let next = match step {
            ui::NavigationStep::First => 0,
            ui::NavigationStep::Last => 255,
            ui::NavigationStep::Next => value.saturating_add(1),
            ui::NavigationStep::Previous => value.saturating_sub(1),
            ui::NavigationStep::PageNext => value.saturating_add(16),
            ui::NavigationStep::PagePrevious => value.saturating_sub(16),
        };
        self.set_channel(channel, next);
    }
    fn position(&mut self, channel: ui::ColorChannel, position: f32) {
        if !position.is_finite() {
            *self.channel_errors.channel_mut(channel) = "The channel position is invalid.".into();
            return;
        }
        let byte = (position.clamp(0., 1.) * 255.).round() as u8;
        self.set_channel(channel, byte);
    }
    fn gesture(&mut self, channel: ui::ColorChannel, gesture: ui::SliderGesture) {
        match gesture {
            ui::SliderGesture::Started => {
                self.snapshot = Some(ColorSnapshot {
                    channel,
                    color: self.accepted,
                    drafts: self.drafts.clone(),
                    hex_error: self.error.clone(),
                    channel_errors: self.channel_errors.clone(),
                })
            }
            ui::SliderGesture::Committed => {
                if self
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.channel == channel)
                {
                    self.snapshot = None;
                }
            }
            ui::SliderGesture::Cancelled => {
                if self
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.channel == channel)
                {
                    let snapshot = self.snapshot.take().expect("matching snapshot");
                    self.accepted = snapshot.color;
                    self.drafts = snapshot.drafts;
                    self.error = snapshot.hex_error;
                    self.channel_errors = snapshot.channel_errors;
                }
            }
        }
    }
    fn project(&self, window: &ui::Showcase) {
        let forms = window.global::<ui::FormExamples>();
        forms.set_color_hex(self.drafts.hex.as_str().into());
        forms.set_color_red(self.drafts.red.as_str().into());
        forms.set_color_green(self.drafts.green.as_str().into());
        forms.set_color_blue(self.drafts.blue.as_str().into());
        forms.set_color_alpha(self.drafts.alpha.as_str().into());
        forms.set_color_error(self.error.as_str().into());
        forms.set_color_red_error(self.channel_errors.red.as_str().into());
        forms.set_color_green_error(self.channel_errors.green.as_str().into());
        forms.set_color_blue_error(self.channel_errors.blue.as_str().into());
        forms.set_color_alpha_error(self.channel_errors.alpha.as_str().into());
        forms.set_accepted_color(ui::SrgbColorView {
            red: i32::from(self.accepted.red),
            green: i32::from(self.accepted.green),
            blue: i32::from(self.accepted.blue),
            alpha: i32::from(self.accepted.alpha),
        });
        forms.set_color_swatch(slint::Color::from_argb_u8(
            self.accepted.alpha,
            self.accepted.red,
            self.accepted.green,
            self.accepted.blue,
        ));
    }
}

pub fn install(window: &ui::Showcase) {
    let resources = Rc::new(RefCell::new(Resources::new()));
    resources.borrow().project(window);
    let forms = window.global::<ui::FormExamples>();
    // The pinned Winit backend has no qualified native drop bridge. A host can
    // enable this public contract once it supplies native DropEvent file paths.
    forms.set_drop_capability(match kit::native::capabilities().drop {
        HostCapability::Available => ui::DropCapability::Available,
        HostCapability::Unsupported => ui::DropCapability::Unsupported,
    });
    {
        let resources = resources.clone();
        let weak = window.as_weak();
        forms.on_file_browse(move |mode| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let ticket = {
                let mut state = resources.borrow_mut();
                let ticket = state.file_requests.begin();
                state.file_state = ui::HostRequestPresentation::Pending;
                state.file_error.clear();
                state.project(&window);
                ticket
            };
            let selection = match mode {
                ui::FileSelectionMode::Single => kit::FileSelectionMode::Single,
                ui::FileSelectionMode::Multiple => kit::FileSelectionMode::Multiple,
            };
            let resources_async = resources.clone();
            let weak_async = weak.clone();
            if slint::spawn_local(async move {
                let outcome = kit::native::pick_files(FileRequest {
                    selection,
                    filters: Vec::new(),
                    suggested_name: None,
                })
                .await;
                if let Some(window) = weak_async.upgrade() {
                    let mut state = resources_async.borrow_mut();
                    state.finish_files(ticket, outcome);
                    state.project(&window);
                }
            })
            .is_err()
            {
                let mut state = resources.borrow_mut();
                state.finish_files(ticket, SelectionOutcome::Failed(HostError::Unavailable));
                state.project(&window);
            }
        });
    }
    {
        let resources = resources.clone();
        let weak = window.as_weak();
        forms.on_file_remove(move |token, generation| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = resources.borrow_mut();
            if state.generation != generation {
                return;
            }
            if !state.files.iter().any(|file| file.id == FileId(token)) {
                return;
            }
            state.file_requests.cancel();
            state.files.retain(|file| file.id != FileId(token));
            state.generation = state
                .generation
                .checked_add(1)
                .expect("file view generation exhausted");
            state.file_state = ui::HostRequestPresentation::Idle;
            state.file_error.clear();
            state.project(&window);
        });
    }
    {
        let resources = resources.clone();
        let weak = window.as_weak();
        forms.on_folder_browse(move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let ticket = {
                let mut state = resources.borrow_mut();
                let ticket = state.folder_requests.begin();
                state.folder_state = ui::HostRequestPresentation::Pending;
                state.folder_error.clear();
                state.project(&window);
                ticket
            };
            let resources_async = resources.clone();
            let weak_async = weak.clone();
            if slint::spawn_local(async move {
                let outcome = kit::native::pick_directory(DirectoryRequest {
                    title: Some("Choose a project folder".into()),
                })
                .await;
                if let Some(window) = weak_async.upgrade() {
                    let mut state = resources_async.borrow_mut();
                    state.finish_folder(ticket, outcome);
                    state.project(&window);
                }
            })
            .is_err()
            {
                let mut state = resources.borrow_mut();
                state.finish_folder(ticket, SelectionOutcome::Failed(HostError::Unavailable));
                state.project(&window);
            }
        });
    }
    {
        let resources = resources.clone();
        let weak = window.as_weak();
        forms.on_folder_clear(move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = resources.borrow_mut();
            state.folder_requests.cancel();
            state.folder = None;
            state.folder_state = ui::HostRequestPresentation::Idle;
            state.folder_error.clear();
            state.project(&window);
        });
    }
    {
        let weak = window.as_weak();
        forms.on_can_drop(move |event| {
            let Some(window) = weak.upgrade() else {
                return DragAction::None;
            };
            if window.global::<ui::FormExamples>().get_drop_capability()
                == ui::DropCapability::Available
                && event.data.has_file_paths()
            {
                DragAction::Copy
            } else {
                DragAction::None
            }
        });
    }
    {
        let resources = resources.clone();
        let weak = window.as_weak();
        forms.on_dropped(move |event: DropEvent| {
            let Some(window) = weak.upgrade() else {
                return DragAction::None;
            };
            if window.global::<ui::FormExamples>().get_drop_capability()
                != ui::DropCapability::Available
            {
                return DragAction::None;
            }
            // Slint1.18.1 stores these paths in the transfer already; this does
            // not stat files, resolve resources, or read their contents.
            let files = match event.data.file_paths() {
                Ok(paths) => paths
                    .map(|path| FileHandle {
                        location: FileLocation::Path(path.to_owned()),
                        display_name: path.file_name().map_or_else(
                            || path.to_string_lossy().into_owned(),
                            |name| name.to_string_lossy().into_owned(),
                        ),
                        byte_length: None,
                    })
                    .collect::<Vec<_>>(),
                Err(_) => {
                    let mut state = resources.borrow_mut();
                    state.file_state = ui::HostRequestPresentation::Failed;
                    state.file_error = "The drop did not contain readable file references.".into();
                    state.project(&window);
                    return DragAction::None;
                }
            };
            if files.is_empty() {
                return DragAction::None;
            }
            let mut state = resources.borrow_mut();
            state.file_requests.cancel();
            state.replace_files(files);
            state.file_state = ui::HostRequestPresentation::Idle;
            state.file_error.clear();
            state.project(&window);
            DragAction::Copy
        });
    }
    let color = Rc::new(RefCell::new(ColorEditor::new()));
    color.borrow().project(window);
    macro_rules! color_callback {($callback:ident,|$state:ident,$($argument:ident),*|$body:block)=>{{let color=color.clone();let weak=window.as_weak();forms.$callback(move|$($argument),*|{let Some(window)=weak.upgrade()else{return;};let mut $state=color.borrow_mut();$body $state.project(&window);});}};}
    color_callback!(on_color_hex_edited, |state, text| {
        state.edit_hex(text.to_string());
    });
    color_callback!(on_color_hex_commit, |state, text| {
        state.commit_hex(text.to_string());
    });
    color_callback!(on_color_channel_edited, |state, channel, text| {
        state.edit_channel(channel, text.to_string(), ColorEditIntent::Edited);
    });
    color_callback!(on_color_channel_commit, |state, channel, text| {
        state.edit_channel(channel, text.to_string(), ColorEditIntent::CommitRequested);
    });
    color_callback!(on_color_position, |state, channel, position| {
        state.position(channel, position);
    });
    color_callback!(on_color_navigate, |state, channel, step| {
        state.navigate(channel, step);
    });
    color_callback!(on_color_gesture, |state, channel, gesture| {
        state.gesture(channel, gesture);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_invalid_draft_retains_accepted_and_gesture_cancel_restores_all_drafts() {
        let mut color = ColorEditor::new();
        let original = color.accepted;
        color.edit_channel(
            ui::ColorChannel::Red,
            "999".into(),
            ColorEditIntent::CommitRequested,
        );
        assert_eq!(color.accepted, original);
        assert_eq!(color.drafts.red, "999");
        assert!(!color.channel_errors.red.is_empty());
        assert!(color.error.is_empty());
        color.edit_hex("#4F".into());
        assert_eq!(color.accepted, original);
        color.gesture(ui::ColorChannel::Red, ui::SliderGesture::Started);
        color.position(ui::ColorChannel::Red, 1.);
        assert_eq!(color.accepted.red, 255);
        color.gesture(ui::ColorChannel::Red, ui::SliderGesture::Cancelled);
        assert_eq!(color.accepted, original);
        assert_eq!(color.drafts.hex, "#4F");
        assert_eq!(color.drafts.red, "999");
        assert!(!color.channel_errors.red.is_empty());
    }
    #[test]
    fn selecting_the_same_resource_again_gets_fresh_view_tokens() {
        let mut state = Resources::new();
        let file = FileHandle {
            location: FileLocation::Path("/tmp/repeated.txt".into()),
            display_name: "repeated.txt".into(),
            byte_length: None,
        };
        state.replace_files(vec![file.clone()]);
        let token = state.files[0].id;
        let generation = state.generation;
        state.replace_files(vec![file]);
        assert_eq!(state.files.len(), 1);
        assert_ne!(state.files[0].id, token);
        assert!(state.generation > generation);
    }
    #[test]
    fn stale_picker_result_cannot_replace_current_resource_selection() {
        let mut state = Resources::new();
        let stale = state.file_requests.begin();
        let current = state.file_requests.begin();
        state.finish_files(stale, SelectionOutcome::Cancelled);
        assert!(state.file_requests.pending());
        state.finish_files(current, SelectionOutcome::Cancelled);
        assert!(!state.file_requests.pending());
        assert_eq!(state.file_state, ui::HostRequestPresentation::Cancelled);
    }
}
