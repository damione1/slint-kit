// SPDX-License-Identifier: GPL-3.0-or-later
use kit::{
    CloseResult, DismissReason, FocusDirection, FocusHandle, FocusId, FocusRegistry, FocusScopeId,
    OverlayId, OverlaySpec, OverlayStack, Ownership, SelectOption,
};
use slint::{ComponentHandle, Model};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, time::Duration};

mod base_data;
mod base_display;
mod base_navigation;
mod form_choices;
mod form_fields;
mod form_files;
mod form_schedule;

mod ui {
    slint::include_modules!();
}
use ui::*;

#[derive(Default)]
enum PreviewPosition {
    #[default]
    Top,
    Middle,
    End,
}

#[derive(Clone, Copy, Debug)]
enum PreviewScene {
    NotificationsUnread,
    NotificationsRead,
    NotificationsEmpty,
    NotificationsDetail,
    DrawerEditing,
    DrawerSaved,
    DrawerSecondary,
    ProgressRunning,
    ProgressCancelled,
    ProgressCompleted,
    ToastError,
    ToastRetry,
    ArticlePreview,
    AccordionMultiple,
    TableEmpty,
    CarouselPreview,
    PaginationLast,
    DialogInvalid,
}
impl PreviewScene {
    fn parse(value: &str) -> Result<Self, String> {
        Ok(match value {
            "notifications-unread" => Self::NotificationsUnread,
            "notifications-read" => Self::NotificationsRead,
            "notifications-empty" => Self::NotificationsEmpty,
            "notifications-detail" => Self::NotificationsDetail,
            "drawer-editing" => Self::DrawerEditing,
            "drawer-saved" => Self::DrawerSaved,
            "drawer-secondary" => Self::DrawerSecondary,
            "progress-running" => Self::ProgressRunning,
            "progress-cancelled" => Self::ProgressCancelled,
            "progress-completed" => Self::ProgressCompleted,
            "toast-error" => Self::ToastError,
            "toast-retry" => Self::ToastRetry,
            "article-preview" => Self::ArticlePreview,
            "accordion-multiple" => Self::AccordionMultiple,
            "table-empty" => Self::TableEmpty,
            "carousel-preview" => Self::CarouselPreview,
            "pagination-last" => Self::PaginationLast,
            "dialog-invalid" => Self::DialogInvalid,
            _ => return Err(format!("Unknown preview scene: {value}")),
        })
    }
    fn location(self) -> (ShowcasePage, PreviewTarget) {
        match self {
            Self::NotificationsUnread
            | Self::NotificationsRead
            | Self::NotificationsEmpty
            | Self::NotificationsDetail => (ShowcasePage::Navigation, PreviewTarget::Notifications),
            Self::DrawerEditing | Self::DrawerSaved => {
                (ShowcasePage::Navigation, PreviewTarget::Drawer)
            }
            Self::DrawerSecondary | Self::AccordionMultiple => {
                (ShowcasePage::Navigation, PreviewTarget::Accordion)
            }
            Self::ProgressRunning | Self::ProgressCancelled | Self::ProgressCompleted => {
                (ShowcasePage::Display, PreviewTarget::Progress)
            }
            Self::ToastError | Self::ToastRetry => (ShowcasePage::Display, PreviewTarget::Toast),
            Self::ArticlePreview => (ShowcasePage::Data, PreviewTarget::Articles),
            Self::TableEmpty => (ShowcasePage::Data, PreviewTarget::Table),
            Self::CarouselPreview => (ShowcasePage::Navigation, PreviewTarget::Carousel),
            Self::PaginationLast => (ShowcasePage::Navigation, PreviewTarget::Pagination),
            Self::DialogInvalid => (ShowcasePage::Gallery, PreviewTarget::Dialog),
        }
    }
    fn capture_delay(self) -> u64 {
        match self {
            Self::ProgressCompleted => 5600,
            Self::DialogInvalid => 1200,
            Self::ProgressRunning | Self::ProgressCancelled => 1400,
            Self::NotificationsUnread
            | Self::NotificationsRead
            | Self::NotificationsEmpty
            | Self::NotificationsDetail
            | Self::DrawerEditing
            | Self::DrawerSaved
            | Self::DrawerSecondary
            | Self::ToastError
            | Self::ToastRetry
            | Self::ArticlePreview
            | Self::AccordionMultiple
            | Self::TableEmpty
            | Self::CarouselPreview
            | Self::PaginationLast => 700,
        }
    }
    fn play(self, window: &Showcase) {
        let nav = window.global::<NavigationExamples>();
        let display = window.global::<DisplayExamples>();
        match self {
            Self::NotificationsUnread
            | Self::NotificationsRead
            | Self::NotificationsEmpty
            | Self::NotificationsDetail => {
                if matches!(self, Self::NotificationsEmpty) {
                    while let Some(row) = nav.get_notifications().row_data(0) {
                        nav.invoke_notice_dismiss(row.item.token, row.item.generation);
                    }
                } else if let Some(row) = nav.get_notifications().row_data(0) {
                    if matches!(self, Self::NotificationsRead) {
                        nav.invoke_notice_read(row.item.token, row.item.generation);
                    }
                    if matches!(self, Self::NotificationsDetail) {
                        nav.invoke_notice_activate(row.item.token, row.item.generation);
                    }
                }
                nav.invoke_notice_open();
            }
            Self::DrawerEditing | Self::DrawerSaved | Self::DrawerSecondary => {
                let launcher = if matches!(self, Self::DrawerSecondary) {
                    nav.set_collapse_open(true);
                    DrawerFocus::CollapseTrigger
                } else {
                    DrawerFocus::Trigger
                };
                nav.invoke_drawer_open(launcher);
                nav.set_drawer_name("Allsafe audit notes".into());
                nav.set_drawer_team(true);
                if matches!(self, Self::DrawerSaved) {
                    nav.invoke_drawer_save("Allsafe audit notes".into(), true, true);
                }
            }
            Self::ProgressRunning | Self::ProgressCompleted => display.invoke_progress_start(),
            Self::ProgressCancelled => {
                display.invoke_progress_start();
                let weak = window.as_weak();
                slint::Timer::single_shot(Duration::from_millis(750), move || {
                    if let Some(window) = weak.upgrade() {
                        window.global::<DisplayExamples>().invoke_progress_cancel();
                    }
                });
            }
            Self::ToastError | Self::ToastRetry => {
                display.invoke_add_toast(ToastExample::Failed);
                if matches!(self, Self::ToastRetry)
                    && let Some(row) = display.get_toast_entries().row_data(0)
                {
                    display.invoke_toast_action(row.token);
                }
            }
            Self::ArticlePreview => window.global::<DataExamples>().invoke_article_open(2, 1),
            Self::AccordionMultiple => {
                nav.invoke_mode_changed(ExpansionMode::Multiple);
                let rows = nav.get_accordion_rows();
                for index in 0..2 {
                    if let Some(row) = rows.row_data(index) {
                        nav.invoke_expand(
                            row.item.token,
                            row.item.generation,
                            true,
                            ExpansionMode::Multiple,
                        );
                    }
                }
            }
            Self::TableEmpty => {
                let data = window.global::<DataExamples>();
                data.set_query("no-results-for-this-query".into());
                data.invoke_query_edited(data.get_query());
            }
            Self::CarouselPreview => {
                let state = nav.get_carousel_state();
                nav.invoke_activate(
                    NavigationDemo::Carousel,
                    state.selected.token,
                    state.generation,
                );
            }
            Self::PaginationLast => nav.invoke_page_navigate(NavigationStep::Last),
            Self::DialogInvalid => {
                window.invoke_dialog_open();
                window.set_project_name("".into());
                window.invoke_dialog_submit(window.get_project_name());
            }
        }
    }
}
#[derive(Default)]
struct Preview {
    scene: Option<PreviewScene>,
    dark: bool,
    enlarged: bool,
    dialog: Option<ShowcaseSkin>,
    select: Option<ShowcaseSkin>,
    snapshot: Option<std::path::PathBuf>,
    density: Option<ControlSize>,
    text_sample: Option<TextSample>,
    content_lab: bool,
    invalid: bool,
    position: PreviewPosition,
    page: Option<ShowcasePage>,
    width: Option<u32>,
    height: Option<u32>,
}
impl Preview {
    fn parse() -> Result<Self, String> {
        let mut result = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--scene" => result.scene = Some(PreviewScene::parse(&args.next().ok_or("--scene needs a name")?)?),
                "--position" => result.position = match args.next().as_deref() {Some("top")=>PreviewPosition::Top,Some("middle")=>PreviewPosition::Middle,Some("end")=>PreviewPosition::End,_=>return Err("--position needs top, middle, or end".into())},
                "--page" => result.page = Some(match args.next().as_deref() {
                    Some("gallery") => ShowcasePage::Gallery,
                    Some("profile") => ShowcasePage::Profile,
                    Some("billing") => ShowcasePage::Billing,
                    Some("choices") => ShowcasePage::Choices,
                    Some("schedule") => ShowcasePage::Schedule,
                    Some("files") => ShowcasePage::Files,
                    Some("display") => ShowcasePage::Display,
                    Some("navigation") => ShowcasePage::Navigation,
                    Some("data") => ShowcasePage::Data,
                    _ => return Err("--page needs gallery, profile, billing, choices, schedule, files, display, navigation, or data".into()),
                }),
                "--width" | "--height" => {
                    let value: u32 = args.next().ok_or("size needs pixels")?.parse().map_err(|_| "size needs positive pixels")?;
                    if !(480..=4096).contains(&value) { return Err("size must be between 480 and 4096 pixels".into()); }
                    if argument == "--width" {result.width=Some(value);} else {result.height=Some(value);}
                }
                "--invalid" => result.invalid = true,
                "--dark" => result.dark = true,
                "--text-200" => result.enlarged = true,
                "--content-lab" => result.content_lab = true,
                "--density" => {
                    result.density = Some(match args.next().as_deref() {
                        Some("compact") => ControlSize::Compact,
                        Some("medium") => ControlSize::Medium,
                        Some("comfortable") => ControlSize::Comfortable,
                        value => return Err(format!("invalid density {value:?}; use compact, medium, or comfortable")),
                    });
                }
                "--text-sample" => {
                    result.text_sample = Some(match args.next().as_deref() {
                        Some("standard") => TextSample::Standard,
                        Some("french") => TextSample::LongFrench,
                        Some("rtl") => TextSample::RightToLeft,
                        value => return Err(format!("invalid text sample {value:?}; use standard, french, or rtl")),
                    });
                }
                "--snapshot" => result.snapshot = Some(args.next().ok_or("--snapshot needs a PNG path")?.into()),
                "--dialog" | "--select" => {
                    let skin = match args.next().as_deref() {
                        Some("mineral") => ShowcaseSkin::Mineral,
                        Some("expressive") => ShowcaseSkin::Expressive,
                        Some(value) => return Err(format!("unknown skin {value}; use mineral or expressive")),
                        None => return Err(format!("{argument} needs a skin")),
                    };
                    match argument.as_str() {
                        "--dialog" => result.dialog = Some(skin),
                        "--select" => result.select = Some(skin),
                        _ => unreachable!("option was matched above"),
                    }
                }
                "--help" => return Err("Usage: kit-showcase [--page gallery|profile|billing|choices|schedule|files|display|navigation|data] [--scene name] [--position top|middle|end] [--width pixels] [--height pixels] [--dark] [--text-200] [--invalid] [--dialog mineral|expressive] [--select mineral|expressive] [--density compact|medium|comfortable] [--text-sample standard|french|rtl] [--content-lab] [--snapshot output.png]".into()),
                value => return Err(format!("unknown argument {value}")),
            }
        }
        Ok(result)
    }
}

fn preview_invalid(window: &Showcase) {
    let form = window.global::<FormExamples>();
    match window.get_page() {
        ShowcasePage::Gallery
        | ShowcasePage::Display
        | ShowcasePage::Navigation
        | ShowcasePage::Data => {}
        ShowcasePage::Profile => {
            form.invoke_text_edited(TextExample::Name, "".into());
            form.invoke_text_edited(TextExample::Email, "unfinished@".into());
            form.invoke_profile_submit();
        }
        ShowcasePage::Billing => {
            form.invoke_numeric_edited(NumericExample::Money, "-".into());
            form.invoke_billing_submit();
        }
        ShowcasePage::Choices => {
            form.invoke_tag_create("native".into());
        }
        ShowcasePage::Schedule => {
            form.set_date_text("2026-02-29".into());
            form.invoke_date_commit(DateExample::Date, "2026-02-29".into());
            form.set_zone_date_text("2026-03-08".into());
            form.set_zone_time_text("02:30".into());
            form.invoke_zone_resolve(ZonedDateTimeDraft {
                date: "2026-03-08".into(),
                time: "02:30".into(),
                zone: "America/Toronto".into(),
            });
        }
        ShowcasePage::Files => {
            form.invoke_color_hex_commit("#4F".into());
            form.invoke_color_channel_commit(ColorChannel::Red, "999".into());
        }
    }
}

fn save_snapshot(
    window: &Showcase,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let pixels = window.window().take_snapshot()?;
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(file),
        pixels.width(),
        pixels.height(),
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()?
        .write_image_data(pixels.as_bytes())?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PersonId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValidationTicket(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValidationState {
    Idle,
    Pending(ValidationTicket),
    Failed,
    Succeeded,
}

const ROOT: FocusScopeId = FocusScopeId(0);
const DIALOG_SCOPE: FocusScopeId = FocusScopeId(1);
const DIALOG: OverlayId = OverlayId(1);
const SELECT: OverlayId = OverlayId(2);

struct Session {
    focus: FocusRegistry,
    targets: BTreeMap<i32, FocusHandle>,
    current: Option<FocusHandle>,
    overlays: OverlayStack,
    validation: ValidationState,
    next_ticket: u64,
}
impl Session {
    fn new() -> Self {
        let mut focus = FocusRegistry::new();
        focus.register_scope(DIALOG_SCOPE, ROOT).unwrap();
        let targets = [1, 2, 3]
            .into_iter()
            .map(|id| {
                (
                    id,
                    focus.register(FocusId(id as u64), ROOT, id.into(), true),
                )
            })
            .collect();
        Self {
            focus,
            targets,
            current: None,
            overlays: OverlayStack::new(),
            validation: ValidationState::Idle,
            next_ticket: 0,
        }
    }
    fn publish(&self, window: &Showcase) {
        window.set_validation_phase(match self.validation {
            ValidationState::Idle => ValidationPhase::Idle,
            ValidationState::Pending(_) => ValidationPhase::Pending,
            ValidationState::Failed => ValidationPhase::Failed,
            ValidationState::Succeeded => ValidationPhase::Succeeded,
        });
        window.set_dialog_state(DialogState {
            open: self.overlays.contains(DIALOG),
            topmost: self
                .overlays
                .topmost()
                .is_some_and(|entry| entry.id == DIALOG),
            busy: self.overlays.is_busy(DIALOG),
        });
        let mut generations = vec![0; 18];
        for (id, handle) in &self.targets {
            generations[*id as usize] =
                i32::try_from(handle.generation).expect("small demo generation");
        }
        if !window
            .get_focus_generations()
            .iter()
            .eq(generations.iter().copied())
        {
            window.set_focus_generations(slint::ModelRc::new(slint::VecModel::from(generations)));
        }
    }
    fn request(&mut self, window: &Showcase, handle: FocusHandle) {
        if let Some(request) = self.focus.request(handle) {
            self.current = Some(handle);
            window.set_focus_request(FocusRequest {
                present: true,
                token: request.target.id.0 as i32,
                generation: request.target.generation as i32,
                request: request.serial as i32,
            });
        }
    }
    fn close_dialog(&mut self, window: &Showcase, reason: DismissReason) {
        if let CloseResult::Closed(closed) = self.overlays.close(DIALOG, reason) {
            let restore = closed.last().and_then(|entry| entry.restore_focus);
            for id in 11..=17 {
                if let Some(handle) = self.targets.remove(&id) {
                    self.focus.unregister(handle);
                }
            }
            self.focus.pop_trap(DIALOG_SCOPE);
            self.validation = ValidationState::Idle;
            window.set_note_visible(false);
            self.publish(window);
            if let Some(request) = self
                .focus
                .restore(&restore.into_iter().collect::<Vec<_>>(), ROOT)
            {
                self.request(window, request.target);
            }
        }
    }
}

fn person_label(PersonId(id): PersonId) -> String {
    if id == 42 {
        return "Elliot Alderson".into();
    }
    if id % 137 == 0 {
        return "Alex Morgan".into();
    }
    let first = [
        "Amara", "Noah", "Élodie", "Sora", "Maya", "Luca", "Mika", "Ren",
    ];
    let last = [
        "Chen", "Laurent", "Patel", "Rivera", "Kim", "Martin", "Okafor", "Silva",
    ];
    format!(
        "{} {} · {:04}",
        first[id as usize % first.len()],
        last[(id as usize / first.len()) % last.len()],
        id
    )
}
fn dataset() -> Vec<SelectOption<PersonId>> {
    (0..10_000)
        .map(|id| SelectOption {
            key: PersonId(id),
            label: person_label(PersonId(id)),
            description: format!(
                "Person #{id:04} · {}",
                if id % 23 == 0 {
                    "Unavailable"
                } else {
                    "Design & engineering"
                }
            ),
            disabled: id % 23 == 0,
        })
        .collect()
}
fn reason(view: CloseReason) -> DismissReason {
    match view {
        CloseReason::Escape => DismissReason::Escape,
        CloseReason::OutsidePress => DismissReason::OutsidePress,
        CloseReason::SystemBack => DismissReason::SystemBack,
        CloseReason::TriggerToggle => DismissReason::TriggerToggle,
        CloseReason::Programmatic => DismissReason::Programmatic,
        CloseReason::HostClosed => DismissReason::HostClosed,
    }
}
fn diagnostics(
    window: &Showcase,
    session: &Session,
    selected: Option<PersonId>,
    source_count: usize,
) {
    let state = window.get_select_state();
    let key = selected.map_or_else(|| "none".into(), |PersonId(id)| format!("PersonId({id})"));
    let focus = session
        .current
        .map_or_else(|| "native".into(), |h| format!("{}", h.id.0));
    let scope = if session.overlays.contains(DIALOG) {
        "dialog"
    } else {
        "window"
    };
    window.set_diagnostics(format!("Key {key} · active row {} · generation {} · {source_count} source rows · focus {focus}/{scope} · {} overlays · {} targets", state.active_index, state.generation, session.overlays.len(), session.focus.len()).into());
    window.set_selected_label(
        selected
            .map_or_else(|| "Choose a person".into(), person_label)
            .into(),
    );
    window.set_selected_detail(
        selected
            .map_or_else(
                || "Search people by name or stable ID".into(),
                |PersonId(id)| format!("Person #{id:04} · Design & engineering"),
            )
            .into(),
    );
    window.set_row_count(source_count as i32);
}

fn main() -> Result<(), slint::PlatformError> {
    let preview = Preview::parse().unwrap_or_else(|message| {
        eprintln!("{message}");
        std::process::exit(2)
    });
    let window = Showcase::new()?;
    form_fields::install(&window);
    form_choices::install(&window);
    form_schedule::install(&window);
    form_files::install(&window);
    base_data::install(&window);
    base_display::install(&window);
    base_navigation::install(&window);
    if let Some(page) = preview.page {
        window.set_page(page);
    }
    if preview.width.is_some() || preview.height.is_some() {
        window.window().set_size(slint::LogicalSize::new(
            preview.width.unwrap_or(1180) as f32,
            preview.height.unwrap_or(900) as f32,
        ));
    }
    window.on_place_popup(|anchor, viewport| {
        let placement = kit::place_popup(
            kit::Rect {
                x: anchor.x,
                y: anchor.y,
                width: anchor.width,
                height: anchor.height,
            },
            kit::Size {
                width: 438.0,
                height: 380.0,
            },
            kit::Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            kit::Insets {
                top: 24.0,
                right: 24.0,
                bottom: 24.0,
                left: 24.0,
            },
            4.0,
        );
        PopupRect {
            x: placement.bounds.x,
            y: placement.bounds.y,
            width: placement.bounds.width,
            height: placement.bounds.height,
        }
    });
    let session = Rc::new(RefCell::new(Session::new()));
    session.borrow().publish(&window);
    let dialog_selection = Rc::new(RefCell::new(None));
    let select = kit::bind_select!(ui, window, dataset(), Some(PersonId(42)), Ownership::Local);
    diagnostics(&window, &session.borrow(), select.selected(), 10_000);

    let weak = window.as_weak();
    let target = session.clone();
    let selection = select.clone();
    window.on_select_state_changed(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let mut session = target.borrow_mut();
        let open = window.get_select_state().open;
        if open && !session.overlays.contains(SELECT) {
            let mut spec = OverlaySpec::new(
                SELECT,
                session.overlays.contains(DIALOG).then_some(DIALOG),
                false,
            );
            spec.restore_focus = match window.get_popup_skin() {
                ShowcaseSkin::Mineral => None,
                ShowcaseSkin::Expressive => session
                    .targets
                    .get(&if session.overlays.contains(DIALOG) {
                        12
                    } else {
                        3
                    })
                    .copied(),
            };
            session
                .overlays
                .push(spec)
                .expect("one select popup per window");
        } else if !open && session.overlays.contains(SELECT) {
            let result = session.overlays.close(SELECT, DismissReason::Programmatic);
            if let Some(handle) = result.restore_focus() {
                session.request(&window, handle);
            }
        }
        session.publish(&window);
        diagnostics(
            &window,
            &session,
            selection.selected(),
            window.get_row_count() as usize,
        );
    });

    let weak = window.as_weak();
    window.on_activated(move || {
        if let Some(window) = weak.upgrade() {
            let count = window.get_activation_count() + 1;
            window.set_activation_count(count);
        }
    });
    let weak = window.as_weak();
    let selection = select.clone();
    window.on_reverse_options(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let before = selection.selected();
        let mut rows = selection.options();
        rows.reverse();
        selection
            .reconcile(rows)
            .expect("reverse preserves unique keys");
        window.set_status(
            format!(
                "Rows reversed. Selected identity {:?} → {:?}.",
                before,
                selection.selected()
            )
            .into(),
        );
    });
    let weak = window.as_weak();
    let selection = select.clone();
    window.on_remove_selected(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let selected = selection.selected();
        let rows: Vec<_> = selection
            .options()
            .into_iter()
            .filter(|row| Some(row.key) != selected)
            .collect();
        window.set_row_count(rows.len() as i32);
        selection
            .reconcile(rows)
            .expect("removal preserves unique keys");
        window.set_status(
            format!(
                "Removed {selected:?}. Selection cleared explicitly; no adjacent row inherits it."
            )
            .into(),
        );
    });
    let weak = window.as_weak();
    let selection = select.clone();
    window.on_restore_options(move || {
        if let Some(window) = weak.upgrade() {
            window.set_row_count(10_000);
        }
        selection.query("");
        selection
            .reconcile(dataset())
            .expect("deterministic unique keys");
        selection.set_selected(Some(PersonId(42)));
        if let Some(window) = weak.upgrade() {
            window.set_status("Restored all 10,000 people and PersonId(42).".into());
        }
    });

    let weak = window.as_weak();
    let target = session.clone();
    let selection = select.clone();
    let saved_selection = dialog_selection.clone();
    window.on_dialog_open(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let mut session = target.borrow_mut();
        if session.overlays.contains(DIALOG) {
            return;
        }
        *saved_selection.borrow_mut() = selection.selected();
        let trigger = match window.get_dialog_skin() {
            ShowcaseSkin::Mineral => 1,
            ShowcaseSkin::Expressive => 2,
        };
        let mut spec = OverlaySpec::new(DIALOG, None, true);
        spec.restore_focus = session.targets.get(&trigger).copied();
        session.overlays.push(spec).expect("fresh dialog");
        for (order, id) in [11, 12, 15, 17, 13, 14].into_iter().enumerate() {
            let handle =
                session
                    .focus
                    .register(FocusId(id as u64), DIALOG_SCOPE, order as i64 * 2, true);
            session.targets.insert(id, handle);
        }
        session.focus.push_trap(DIALOG_SCOPE).unwrap();
        session.validation = ValidationState::Idle;
        window.set_validation_message("".into());
        session.publish(&window);
        let first = session.targets[&11];
        session.request(&window, first);
    });
    let weak = window.as_weak();
    let target = session.clone();
    let selection = select.clone();
    let saved_selection = dialog_selection.clone();
    window.on_dialog_dismiss(move |view_reason| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        if selection.snapshot().open {
            selection.dismiss(reason(view_reason));
            return;
        }
        target
            .borrow_mut()
            .close_dialog(&window, reason(view_reason));
        selection.set_selected(*saved_selection.borrow());
    });
    let weak = window.as_weak();
    let target = session.clone();
    window.on_focus_next(move |backward| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let mut session = target.borrow_mut();
        let current = session.current;
        let direction = if backward {
            FocusDirection::Backward
        } else {
            FocusDirection::Forward
        };
        if let Some(request) = session.focus.cycle(DIALOG_SCOPE, current, direction) {
            session.request(&window, request.target);
        }
    });
    let weak = window.as_weak();
    let target = session.clone();
    window.on_select_tab(move |backward| {
        let Some(window) = weak.upgrade() else {
            return TabDisposition::NativeNavigation;
        };
        let mut session = target.borrow_mut();
        if !session.overlays.contains(DIALOG) {
            return TabDisposition::NativeNavigation;
        }
        let current = session.current;
        let direction = if backward {
            FocusDirection::Backward
        } else {
            FocusDirection::Forward
        };
        if let Some(request) = session.focus.cycle(DIALOG_SCOPE, current, direction) {
            session.request(&window, request.target);
        }
        TabDisposition::HostHandled
    });
    let target = session.clone();
    window.on_focus_changed(move |id, focused| {
        if focused && let Ok(mut session) = target.try_borrow_mut() {
            session.current = session.targets.get(&id).copied();
        }
    });
    let weak = window.as_weak();
    let target = session.clone();
    window.on_dialog_toggle_note(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let mut session = target.borrow_mut();
        if window.get_note_visible() {
            if let Some(handle) = session.targets.remove(&16) {
                session.focus.unregister(handle);
            }
            window.set_note_visible(false);
            session.publish(&window);
            let fallback = session.targets[&15];
            session.request(&window, fallback);
        } else {
            let handle = session.focus.register(FocusId(16), DIALOG_SCOPE, 5, true);
            session.targets.insert(16, handle);
            session.publish(&window);
            window.set_note_visible(true);
            session.request(&window, handle);
        }
    });
    let weak = window.as_weak();
    let target = session.clone();
    let selection = select.clone();
    let saved_selection = dialog_selection.clone();
    window.on_dialog_submit(move |name| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let ticket = {
            let mut session = target.borrow_mut();
            if matches!(session.validation, ValidationState::Pending(_))
                || !session.overlays.contains(DIALOG)
            {
                return;
            }
            session.next_ticket += 1;
            let ticket = ValidationTicket(session.next_ticket);
            session.validation = ValidationState::Pending(ticket);
            session.overlays.set_busy(DIALOG, true);
            for id in [11, 12, 13, 14, 15, 16, 17] {
                if let Some(handle) = session.targets.get(&id).copied() {
                    session.focus.set_eligible(handle, false);
                }
            }
            session.publish(&window);
            ticket
        };
        window.set_status("Validating asynchronously…".into());
        window.set_validation_message("Checking the project name…".into());
        let weak = window.as_weak();
        let target = target.clone();
        let selection = selection.clone();
        let saved_selection = saved_selection.clone();
        slint::Timer::single_shot(Duration::from_millis(650), move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut session = target.borrow_mut();
            if session.validation != ValidationState::Pending(ticket) {
                return;
            }
            session.overlays.set_busy(DIALOG, false);
            for id in [11, 12, 13, 14, 15, 16, 17] {
                if let Some(handle) = session.targets.get(&id).copied() {
                    session.focus.set_eligible(handle, true);
                }
            }
            if name.trim().chars().count() < 3 {
                session.validation = ValidationState::Failed;
                window.set_validation_message(
                    "Use at least 3 characters. Your edits were kept.".into(),
                );
                session.publish(&window);
                window.set_status(
                    "Use at least 3 characters for the project name. Your edits were kept.".into(),
                );
                let first = session.targets[&11];
                session.request(&window, first);
            } else {
                session.validation = ValidationState::Succeeded;
                window.set_status(format!("Created project: {}", name.trim()).into());
                session.close_dialog(&window, DismissReason::Programmatic);
                drop(session);
                selection.set_selected(*saved_selection.borrow());
            }
        });
    });
    let weak = window.as_weak();
    window
        .global::<FormExamples>()
        .on_reveal_field(move |top, height| {
            if let Some(window) = weak.upgrade() {
                window.invoke_reveal_content(top, height);
            }
        });
    let weak = window.as_weak();
    window
        .global::<PreviewExamples>()
        .on_reveal(move |top, _height| {
            if let Some(window) = weak.upgrade() {
                window.invoke_position_content(top);
            }
        });
    if let Some(scene) = preview.scene {
        let (page, target) = scene.location();
        window.set_page(page);
        let weak = window.as_weak();
        slint::Timer::single_shot(Duration::from_millis(150), move || {
            if let Some(window) = weak.upgrade() {
                let _ = window.window().take_snapshot();
                let preview = window.global::<PreviewExamples>();
                preview.set_target(target);
                preview.set_request(preview.get_request() + 1);
                let weak = window.as_weak();
                slint::Timer::single_shot(Duration::from_millis(100), move || {
                    if let Some(window) = weak.upgrade() {
                        scene.play(&window);
                    }
                });
            }
        });
    }
    if preview.dark {
        window.set_theme_mode(ThemeMode::Dark);
    }
    window.set_enlarged(preview.enlarged);
    if let Some(density) = preview.density {
        window.set_density(density);
    }
    if let Some(sample) = preview.text_sample {
        window.set_text_sample(sample);
    }
    if preview.content_lab {
        let weak = window.as_weak();
        slint::Timer::single_shot(Duration::from_millis(150), move || {
            if let Some(window) = weak.upgrade() {
                window.invoke_show_content_lab();
            }
        });
    }
    if let Some(skin) = preview.dialog {
        window.set_dialog_skin(skin);
        window.invoke_dialog_open();
    }
    if let Some(skin) = preview.select {
        window.set_popup_skin(skin);
        select.open();
    }
    if preview.invalid {
        let weak = window.as_weak();
        slint::Timer::single_shot(Duration::from_millis(80), move || {
            if let Some(window) = weak.upgrade() {
                preview_invalid(&window);
            }
        });
    }
    let scroll_position = match preview.position {
        PreviewPosition::Top => 0.,
        PreviewPosition::Middle => 0.5,
        PreviewPosition::End => 1.,
    };
    if scroll_position > 0. {
        let weak = window.as_weak();
        slint::Timer::single_shot(Duration::from_millis(150), move || {
            if let Some(window) = weak.upgrade() {
                let _ = window.window().take_snapshot();
                window.invoke_scroll_page(scroll_position);
            }
        });
    }
    if let Some(path) = preview.snapshot {
        let weak = window.as_weak();
        slint::Timer::single_shot(
            Duration::from_millis(preview.scene.map_or(350, PreviewScene::capture_delay)),
            move || {
                if let Some(window) = weak.upgrade() {
                    if scroll_position > 0. {
                        let _ = window.window().take_snapshot();
                        window.invoke_scroll_page(scroll_position);
                    }
                    if let Err(error) = save_snapshot(&window, &path) {
                        eprintln!("Snapshot failed: {error}");
                        std::process::exit(1);
                    }
                    println!("Saved {}", path.display());
                }
                let _ = slint::quit_event_loop();
            },
        );
    }
    window.run()
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use i_slint_backend_testing::ElementQuery;
    #[test]
    fn gallery_end_scroll_keeps_navigation_fixed_and_footer_inside_viewport() {
        i_slint_backend_testing::init_no_event_loop();
        let window = Showcase::new().unwrap();
        window.show().unwrap();
        for enlarged in [false, true] {
            window.set_enlarged(enlarged);
            let navigation = ElementQuery::from_root(&window)
                .match_inherits("KitPressable")
                .match_predicate(|element| {
                    element.accessible_label().as_deref() == Some("Buttons & dialogs")
                })
                .find_first()
                .unwrap();
            let before = navigation.absolute_position();
            window.invoke_scroll_page(1.);
            let footer = ElementQuery::from_root(&window)
                .match_predicate(|element| {
                    element.accessible_label().as_deref() == Some("Slint Kit · GPL-3.0-or-later")
                })
                .find_first()
                .unwrap();
            let _ = footer.absolute_position();
            window.invoke_scroll_page(1.);
            assert_eq!(navigation.absolute_position(), before);
            let bottom = footer.absolute_position().y + footer.size().height;
            let viewport_height = window.window().size().height as f32;
            assert!(
                bottom <= viewport_height - 20. && bottom >= viewport_height - 70.,
                "footer bottom {bottom}, viewport {viewport_height}"
            );
        }
    }
}
