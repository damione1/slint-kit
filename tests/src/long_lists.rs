// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, LogicalSize, Model};
use slint_kit::KeyedModel;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

type Tokens = Rc<RefCell<BTreeSet<i32>>>;

fn help(index: usize) -> String {
    match index % 7 {
        0 => String::new(),
        3 => "A long help sentence that explains the consequence of this action in enough detail to wrap on several lines inside a narrow viewport, with accents éèà and an unbroken identifier xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx. ".repeat(1 + index % 3),
        _ => format!("Short help {index}"),
    }
}
fn records(count: usize) -> KeyedModel<u64, ui::ContentItem> {
    KeyedModel::new((1..=count).map(|index| {
        (
            index as u64,
            ui::ContentItem {
                title: format!("Record {index}").into(),
                excerpt: help(index).into(),
                ..Default::default()
            },
        )
    }))
    .unwrap()
}
thread_local!(static READY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
fn init() {
    if !READY.replace(true) {
        use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
        slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
            mock_time: true,
            renderer_name: Some("software".into()),
            ..Default::default()
        })))
        .unwrap();
    }
}
fn tick(milliseconds: u64) {
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(milliseconds));
    slint::platform::update_timers_and_animations();
}
fn settle() {
    for _ in 0..30 {
        tick(16);
    }
}
// Delegates are alive when their disabled-by-default Timer fires during a short pulse.
fn live_tokens(set_pulse: impl Fn(bool), pulsed: &Tokens) -> BTreeSet<i32> {
    settle();
    pulsed.borrow_mut().clear();
    set_pulse(true);
    for _ in 0..4 {
        tick(25);
    }
    set_pulse(false);
    slint::platform::update_timers_and_animations();
    let live = std::mem::take(&mut *pulsed.borrow_mut());
    for _ in 0..4 {
        tick(25);
    }
    assert!(
        pulsed.borrow().is_empty(),
        "every delegate timer stops after the pulse"
    );
    live
}
struct Rich {
    window: LongListFixture,
    model: KeyedModel<u64, ui::ContentItem>,
    pulsed: Tokens,
    mounts: Rc<RefCell<Vec<i32>>>,
}
fn rich(count: usize) -> Rich {
    init();
    let window = LongListFixture::new().unwrap();
    kit_slint::install(&window);
    let model = records(count);
    window.set_rows(model.model());
    let pulsed: Tokens = Default::default();
    let sink = pulsed.clone();
    window.on_pulsed(move |token| {
        sink.borrow_mut().insert(token);
    });
    let mounts: Rc<RefCell<Vec<i32>>> = Default::default();
    let log = mounts.clone();
    window.on_mounted(move |token| log.borrow_mut().push(token));
    window.show().unwrap();
    settle();
    Rich {
        window,
        model,
        pulsed,
        mounts,
    }
}
impl Rich {
    fn live(&self) -> BTreeSet<i32> {
        let weak = self.window.as_weak();
        live_tokens(
            move |value| weak.upgrade().unwrap().set_pulse(value),
            &self.pulsed,
        )
    }
    fn token(&self, key: u64) -> i32 {
        (0..self.model.model().row_count())
            .map(|index| self.model.model().row_data(index).unwrap())
            .find(|row| row.title == format!("Record {key}"))
            .unwrap()
            .token
    }
    fn index_of(&self, key: u64) -> i32 {
        (0..self.model.model().row_count())
            .position(|index| {
                self.model.model().row_data(index).unwrap().title == format!("Record {key}")
            })
            .unwrap() as i32
    }
    fn reveal_key(&self, key: u64) {
        self.window.invoke_reveal_row(self.index_of(key));
        settle();
    }
}
fn field_of(window: &impl ComponentHandle, label: &str) -> Option<ElementHandle> {
    ElementHandle::find_by_accessible_label(window, label).find(|element| {
        element.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::TextInput)
    })
}
fn editor_of(window: &impl ComponentHandle, label: &str) -> Option<ElementHandle> {
    field_of(window, label).map(|field| crate::native_editor(&field))
}
fn dump(window: &impl ComponentHandle) {
    for element in ElementQuery::from_root(window).find_all() {
        if let Some(name) = element.accessible_label() {
            eprintln!(
                "element {:?} {:?} {:?} at {:?} size {:?}",
                element.type_name(),
                element.accessible_role(),
                name,
                element.absolute_position(),
                element.size()
            );
        }
    }
}
fn draft_of(window: &impl ComponentHandle, label: &str) -> String {
    text_of(&field_of(window, label).unwrap_or_else(|| panic!("mounted field {label}")))
}
fn click(window: &impl ComponentHandle, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}
fn centre(element: &ElementHandle) -> LogicalPosition {
    let at = element.absolute_position();
    let size = element.size();
    LogicalPosition::new(at.x + size.width / 2., at.y + size.height / 2.)
}
fn type_text(window: &impl ComponentHandle, text: &str) {
    for character in text.chars() {
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: character.to_string().into(),
        });
        window.window().dispatch_event(WindowEvent::KeyReleased {
            text: character.to_string().into(),
        });
    }
}
fn text_of(element: &ElementHandle) -> String {
    element
        .accessible_value()
        .map(|value| value.to_string())
        .unwrap_or_default()
}

#[test]
fn virtualized_rich_rows_keep_a_bounded_live_window_at_0_100_and_1000_rows() {
    for count in [0usize, 100, 1000] {
        let list = rich(count);
        let first = list.live();
        if count == 0 {
            assert!(first.is_empty() && list.mounts.borrow().is_empty());
            continue;
        }
        let mut seen = vec![("top", first)];
        for (name, key) in [("middle", count as u64 / 2), ("end", count as u64)] {
            list.reveal_key(key);
            seen.push((name, list.live()));
        }
        for (name, set) in &seen {
            assert!(
                (1..=5).contains(&set.len()),
                "{count} rows, {name}: {} live delegates for a 346 px viewport",
                set.len()
            );
        }
        let last = list.token(count as u64);
        assert!(
            seen[2].1.contains(&last),
            "the last record is reachable and mounted"
        );
        assert!(
            (list.mounts.borrow().len()) < 40,
            "{count} rows mounted {} delegates over three seeks",
            list.mounts.borrow().len()
        );
        list.window.invoke_reveal_row(0);
        settle();
        assert!(
            list.live().contains(&list.token(1)),
            "returning to the top restores row one"
        );
        assert_eq!(list.window.get_content_y(), 0.);
    }
}

#[test]
fn the_unvirtualized_witness_proves_the_pulse_sees_every_live_delegate() {
    init();
    let window = UnvirtualizedWitness::new().unwrap();
    kit_slint::install(&window);
    let model = records(100);
    window.set_rows(model.model());
    let pulsed: Tokens = Default::default();
    let sink = pulsed.clone();
    window.on_pulsed(move |token| {
        sink.borrow_mut().insert(token);
    });
    window.show().unwrap();
    let weak = window.as_weak();
    let live = live_tokens(
        move |value| weak.upgrade().unwrap().set_pulse(value),
        &pulsed,
    );
    assert_eq!(
        live.len(),
        100,
        "a plain ScrollFrame instantiates every row and the pulse counts all of them"
    );
}

#[test]
fn a_coordinate_seek_is_not_reliable_for_variable_rows_but_reveal_row_is() {
    let list = rich(1000);
    let travel = list.window.get_content_height() - list.window.get_viewport_height();
    list.window.invoke_scroll_to_y(travel / 2.);
    settle();
    let _ = list.live();
    list.window.invoke_scroll_to_y(travel);
    settle();
    let by_coordinate = list.live();
    eprintln!(
        "coordinate seek after a middle seek: {:?}..{:?}, last token {}",
        by_coordinate.first(),
        by_coordinate.last(),
        list.token(1000)
    );
    let last = list.token(1000);
    list.reveal_key(1000);
    let by_row = list.live();
    assert!(by_row.contains(&last));
    assert!(
        !by_coordinate.contains(&last),
        "variable heights make the estimated end a different row; seek by identity with reveal-row"
    );
    let bottom = ElementHandle::find_by_accessible_label(&list.window, "Open Record 1000")
        .next()
        .unwrap();
    assert!(
        bottom.absolute_position().y + bottom.size().height <= list.window.get_list_height() + 1.
    );
}

#[test]
fn a_draft_is_kept_by_the_host_and_restored_to_the_same_identity() {
    let list = rich(100);
    let model = list.model.clone();
    let drafts: Rc<RefCell<BTreeMap<i32, String>>> = Default::default();
    let store = drafts.clone();
    list.window.on_draft_edited(move |token, text| {
        store.borrow_mut().insert(token, text.to_string());
        if let Some(key) = model.key(token) {
            model.update_by_key(&key, |row| row.metadata = text);
        }
    });
    let target = list.token(5);
    list.reveal_key(5);
    let editor = editor_of(&list.window, "Note for Record 5").expect("mounted editor");
    click(&list.window, centre(&editor));
    type_text(&list.window, "ab");
    type_text(&list.window, "c");
    eprintln!(
        "draft store {:?} model metadata {:?} editor text {:?}",
        drafts.borrow().get(&target),
        list.model
            .model()
            .row_data(list.index_of(5) as usize)
            .unwrap()
            .metadata,
        draft_of(&list.window, "Note for Record 5")
    );
    assert_eq!(
        drafts.borrow()[&target],
        "abc",
        "focus survives the row update caused by the first keystroke"
    );
    list.reveal_key(90);
    assert!(
        !list.live().contains(&target),
        "the edited row left the window"
    );
    assert!(editor_of(&list.window, "Note for Record 5").is_none());
    list.reveal_key(5);
    assert_eq!(
        draft_of(&list.window, "Note for Record 5"),
        "abc",
        "the draft is restored from the host on remount"
    );
    assert_eq!(draft_of(&list.window, "Note for Record 6"), "");
    list.model.sort_by(|a, b| b.cmp(a));
    settle();
    list.reveal_key(5);
    assert_eq!(
        draft_of(&list.window, "Note for Record 5"),
        "abc",
        "sorting keeps the draft on its identity"
    );
    assert_eq!(draft_of(&list.window, "Note for Record 6"), "");
    list.model.remove(target);
    settle();
    list.window.invoke_reveal_row(0);
    settle();
    assert!(!list.live().contains(&target));
    for key in [4u64, 6] {
        list.reveal_key(key);
        assert_eq!(
            draft_of(&list.window, &format!("Note for Record {key}")),
            "",
            "no neighbour inherits the removed draft"
        );
    }
    list.model
        .insert(
            5,
            ui::ContentItem {
                title: "Record 5".into(),
                ..Default::default()
            },
        )
        .unwrap();
    settle();
    list.reveal_key(5);
    assert_eq!(
        draft_of(&list.window, "Note for Record 5"),
        "",
        "a new lifecycle of the same key starts empty"
    );
    assert_ne!(list.token(5), target);
}

fn log_lines(count: usize) -> KeyedModel<u64, ui::TextLine> {
    KeyedModel::new((1..=count).map(|index| {
        let text = match index {
            10 => String::new(),
            20 => format!("{index} {}", "very-long-unbroken-token-".repeat(120)),
            _ => format!("2026-11-18 12:00:{:02} INFO souffle::engine line {index}: transcription événement à traiter, chemin /Users/éloïse/données/session et un long message qui se replie plusieurs fois", index % 60),
        };
        (index as u64, ui::TextLine { number: index as i32, text: text.into(), ..Default::default() })
    }))
    .unwrap()
}
struct Log {
    window: LogFixture,
    model: KeyedModel<u64, ui::TextLine>,
    pulsed: Tokens,
    selections: Rc<RefCell<Vec<(i32, i32, i32)>>>,
}
fn log(count: usize) -> Log {
    init();
    let window = LogFixture::new().unwrap();
    kit_slint::install(&window);
    let model = log_lines(count);
    window.set_lines(model.model());
    let pulsed: Tokens = Default::default();
    let sink = pulsed.clone();
    window.on_pulsed(move |token| {
        sink.borrow_mut().insert(token);
    });
    let selections: Rc<RefCell<Vec<(i32, i32, i32)>>> = Default::default();
    let record = selections.clone();
    window.on_selection_changed(move |token, anchor, cursor| {
        record.borrow_mut().push((token, anchor, cursor))
    });
    window.show().unwrap();
    settle();
    Log {
        window,
        model,
        pulsed,
        selections,
    }
}
impl Log {
    fn live(&self) -> BTreeSet<i32> {
        let weak = self.window.as_weak();
        live_tokens(
            move |value| weak.upgrade().unwrap().set_pulse(value),
            &self.pulsed,
        )
    }
    fn text_of(&self, key: u64) -> String {
        (0..self.model.model().row_count())
            .map(|index| self.model.model().row_data(index).unwrap())
            .find(|row| row.number as u64 == key)
            .unwrap()
            .text
            .to_string()
    }
    fn clipboard(&self) -> String {
        self.window.invoke_read_clipboard();
        self.window.get_clipboard_text().to_string()
    }
    fn drag_across_the_wrap(&self, label: &str) -> ElementHandle {
        // Presses at the same point inside the multi-click interval would select a word or the line.
        tick(1000);
        let editor = editor_of(&self.window, label).unwrap_or_else(|| {
            for element in ElementQuery::from_root(&self.window).find_all() {
                if let Some(name) = element.accessible_label() {
                    eprintln!(
                        "element {:?} {:?} {:?}",
                        element.type_name(),
                        element.accessible_role(),
                        name
                    );
                }
            }
            panic!("mounted line {label}")
        });
        let at = editor.absolute_position();
        let start = LogicalPosition::new(at.x + 4., at.y + 8.);
        let end = LogicalPosition::new(at.x + 140., at.y + editor.size().height * 0.4);
        self.window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position: start });
        self.window
            .window()
            .dispatch_event(WindowEvent::PointerPressed {
                position: start,
                button: PointerEventButton::Left,
            });
        self.window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position: end });
        self.window
            .window()
            .dispatch_event(WindowEvent::PointerReleased {
                position: end,
                button: PointerEventButton::Left,
            });
        editor
    }
    fn copy_shortcut(&self) {
        self.copy_with(Key::Control);
    }
    fn copy_with(&self, modifier: Key) {
        let window = self.window.window();
        window.dispatch_event(WindowEvent::KeyPressed {
            text: modifier.into(),
        });
        window.dispatch_event(WindowEvent::KeyPressed { text: "c".into() });
        window.dispatch_event(WindowEvent::KeyReleased { text: "c".into() });
        window.dispatch_event(WindowEvent::KeyReleased {
            text: modifier.into(),
        });
    }
}

#[test]
fn a_wrapped_log_line_keeps_native_substring_selection_and_platform_copy() {
    let log = log(80);
    let live = log.live();
    assert!(
        live.len() < 40,
        "80 long lines mounted {} editors",
        live.len()
    );
    let line = log.text_of(1);
    let editor = log.drag_across_the_wrap("Line 1");
    assert!(
        editor.size().height > 40.,
        "a 150 character line wraps into several visual lines at 360 px"
    );
    log.copy_shortcut();
    let copied = log.clipboard();
    assert!(
        !copied.is_empty() && line.contains(&copied),
        "copied {copied:?} is a substring of the line"
    );
    assert!(
        copied.chars().count() > 40 && copied.len() < line.len(),
        "the selection crosses the visual wrap and is not the whole line: {copied:?}"
    );
    let last = log.selections.borrow().last().copied().unwrap();
    assert!(
        last.1 != last.2,
        "anchor and cursor describe a non-empty native selection"
    );
}

#[test]
fn the_full_log_copy_is_independent_of_the_mounted_window() {
    let log = log(80);
    let live = log.live();
    let full = (1..=80u64)
        .map(|key| log.text_of(key))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(live.len() < 80);
    log.window.invoke_seed_clipboard(full.clone().into());
    assert_eq!(log.clipboard(), full);
    assert_eq!(full.lines().count(), 80);
    assert!(
        full.contains("é") && full.contains("\n\n"),
        "the empty line and accents are preserved"
    );
}

#[test]
fn selection_and_position_survive_simulated_two_second_appends_and_a_resize() {
    let log = log(80);
    log.window.invoke_reveal_line(0);
    settle();
    log.drag_across_the_wrap("Line 1");
    let selection = log.selections.borrow().last().copied().unwrap();
    let before_y = log.window.get_content_y();
    let first = *log.live().first().unwrap();
    for tick_index in 0..5u64 {
        tick(2000);
        log.model
            .insert(
                81 + tick_index,
                ui::TextLine {
                    number: 81 + tick_index as i32,
                    text: format!("appended {tick_index} événement").into(),
                    ..Default::default()
                },
            )
            .unwrap();
        settle();
    }
    assert_eq!(
        log.window.get_content_y(),
        before_y,
        "reading above the tail is not moved by appends"
    );
    assert_eq!(*log.live().first().unwrap(), first);
    assert_eq!(
        log.selections.borrow().last().copied().unwrap(),
        selection,
        "native selection is untouched by appends"
    );
    log.copy_shortcut();
    assert!(!log.clipboard().is_empty());
    log.window.window().set_size(LogicalSize::new(280., 400.));
    settle();
    assert_eq!(
        *log.live().first().unwrap(),
        first,
        "resize keeps the first read line"
    );
    assert_eq!(
        log.selections.borrow().last().copied().unwrap().0,
        selection.0,
        "the selected identity is unchanged by resize"
    );
}

#[test]
fn rotation_is_re_anchored_by_identity_and_the_tail_follows_only_at_the_end() {
    let log = log(80);
    log.window.invoke_reveal_line(40);
    settle();
    let anchor_token = *log.live().first().unwrap();
    let anchor_key = log.model.key(anchor_token).unwrap();
    for key in 1..=10u64 {
        let token = (0..log.model.model().row_count())
            .map(|index| log.model.model().row_data(index).unwrap())
            .find(|row| row.number as u64 == key)
            .unwrap()
            .token;
        log.model.remove(token);
    }
    settle();
    let index_of = |key: u64| {
        (0..log.model.model().row_count())
            .position(|index| log.model.model().row_data(index).unwrap().number as u64 == key)
            .unwrap() as i32
    };
    log.window.invoke_reveal_line(index_of(anchor_key));
    settle();
    assert!(
        log.live().contains(&anchor_token),
        "the host re-anchors the same line after rotation"
    );
    let at_end = |log: &Log| {
        log.window
            .invoke_reveal_line(log.model.model().row_count() as i32 - 1);
        settle();
    };
    at_end(&log);
    let last_token = log
        .model
        .model()
        .row_data(log.model.model().row_count() - 1)
        .unwrap()
        .token;
    assert!(
        log.live().contains(&last_token),
        "the declared follow policy reaches the final line"
    );
}

#[test]
fn a_nested_list_keeps_the_wheel_at_its_edge_and_the_parent_scrolls_outside_it() {
    init();
    let window = NestedListsFixture::new().unwrap();
    kit_slint::install(&window);
    let model = records(30);
    window.set_rows(model.model());
    window.show().unwrap();
    settle();
    let over_inner = LogicalPosition::new(200., 120. + 60.);
    let wheel = |delta: f32| {
        window.window().dispatch_event(WindowEvent::PointerMoved {
            position: over_inner,
        });
        window
            .window()
            .dispatch_event(WindowEvent::PointerScrolled {
                position: over_inner,
                delta_x: 0.,
                delta_y: delta,
            });
        settle();
    };
    wheel(-100.);
    assert!(
        window.get_inner_y() < 0. && window.get_outer_y() == 0.,
        "the child scrolls first and the parent stays"
    );
    for _ in 0..400 {
        wheel(-400.);
        if window.get_inner_y() <= -window.get_inner_travel() + 0.5 {
            break;
        }
    }
    assert!(
        (window.get_inner_y() + window.get_inner_travel()).abs() < 1.,
        "inner list reaches its end"
    );
    tick(900);
    let outer_at_edge = window.get_outer_y();
    let inner_at_edge = window.get_inner_y();
    for _ in 0..3 {
        wheel(-100.);
    }
    assert_eq!(
        window.get_outer_y(),
        outer_at_edge,
        "Slint does not chain the wheel at a child's edge"
    );
    assert_eq!(window.get_inner_y(), inner_at_edge);
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(200., 60.),
            delta_x: 0.,
            delta_y: -100.,
        });
    settle();
    assert!(
        window.get_outer_y() < outer_at_edge,
        "the parent scrolls when the pointer is outside the child"
    );
    wheel(100.);
    assert!(
        window.get_inner_y() > inner_at_edge,
        "reverse scrolling at the child's end moves the child"
    );
}

fn save(window: &impl ComponentHandle, name: &str) {
    crate::choice_presentation::save_capture(&window.window().take_snapshot().unwrap(), name);
}

#[test]
fn the_rich_list_and_the_log_render_in_both_palettes_widths_and_text_scales() {
    let list = rich(100);
    let log = log(80);
    for (palette, base) in [
        ("light", list.window.get_theme()),
        ("dark", list.window.get_dark_theme()),
    ] {
        for (width, name) in [(320., "narrow"), (680., "wide")] {
            for scale in [1., 2.] {
                let mut theme = base.clone();
                theme.text_scale = scale;
                list.window.set_theme(theme.clone());
                list.window.window().set_size(LogicalSize::new(width, 640.));
                settle();
                list.reveal_key(1);
                let Some(editor) = editor_of(&list.window, "Note for Record 1") else {
                    dump(&list.window);
                    panic!("no editor at {palette} {name} {scale}");
                };
                click(&list.window, centre(&editor));
                settle();
                let tag = format!("l04-list-{palette}-{name}-{scale}");
                save(&list.window, &tag);
                let live = list.live();
                assert!(
                    !live.is_empty() && live.len() < 12,
                    "{tag}: {} live",
                    live.len()
                );
                log.window.set_theme(theme);
                log.window.window().set_size(LogicalSize::new(width, 400.));
                settle();
                log.drag_across_the_wrap("Line 1");
                save(&log.window, &format!("l04-log-{palette}-{name}-{scale}"));
            }
        }
    }
}

fn wheel_at(window: &impl ComponentHandle, delta: f32) {
    let position = LogicalPosition::new(200., 180.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position,
            delta_x: 0.,
            delta_y: delta,
        });
    tick(900);
}
#[test]
fn nested_wheel_witnesses_separate_slint_from_the_kit_list() {
    init();
    let plain = PlainNestedFixture::new().unwrap();
    plain.show().unwrap();
    for _ in 0..12 {
        wheel_at(&plain, -400.);
    }
    eprintln!(
        "plain Flickable nest: inner {} outer {}",
        plain.get_inner_y(),
        plain.get_outer_y()
    );
    assert!(
        plain.get_inner_y() < -1000. && plain.get_outer_y() == 0.,
        "upstream Flickable keeps the wheel after the child's edge"
    );
    let frame = FrameNestedFixture::new().unwrap();
    kit_slint::install(&frame);
    frame.show().unwrap();
    for _ in 0..12 {
        wheel_at(&frame, -400.);
    }
    assert!(
        frame.get_inner_y() < -1000. && frame.get_outer_y() == 0.,
        "kit ScrollFrame inherits the same upstream behavior"
    );
}

#[test]
fn the_headless_copy_shortcut_for_the_kit_log_matches_a_plain_text_input() {
    let log = log(80);
    let outcomes: Vec<(Key, String, String)> = [Key::Control, Key::Meta]
        .into_iter()
        .map(|modifier| {
            log.window.invoke_seed_clipboard("sentinel".into());
            log.window.invoke_select_probe("plain".into());
            log.copy_with(modifier);
            let plain = log.clipboard();
            log.window.invoke_seed_clipboard("sentinel".into());
            log.drag_across_the_wrap("Line 1");
            log.copy_with(modifier);
            (modifier, plain, log.clipboard())
        })
        .collect();
    eprintln!("copy shortcut outcomes (plain TextInput, kit log): {outcomes:?}");
    for (modifier, plain, kit) in &outcomes {
        assert_eq!(
            plain == "plain",
            kit != "sentinel" && !kit.is_empty(),
            "{modifier:?}: the kit editor copies exactly when a plain Slint TextInput does"
        );
    }
    assert!(
        outcomes[0].1 == "plain" && outcomes[0].2 != "sentinel",
        "Control+C copies under the headless backend"
    );
}

#[test]
fn the_guide_snippet_restores_a_host_draft_when_its_row_is_recreated() {
    init();
    let window = GuideRows::new().unwrap();
    kit_slint::install(&window);
    let model = records(3);
    model.update_by_key(&1, |row| row.metadata = "kept".into());
    window.set_rows(model.model());
    window.window().set_size(LogicalSize::new(480., 400.));
    window.show().unwrap();
    settle();
    assert_eq!(draft_of(&window, "Note for Record 1"), "kept");
}
