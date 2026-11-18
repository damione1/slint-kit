// SPDX-License-Identifier: GPL-3.0-or-later
use crate::FragmentFixture;
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, TestingBackend, TestingBackendOptions,
};
use slint::{
    ComponentHandle, LogicalPosition, ModelRc, SharedString, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{CloseReason, OptionRow, PopoverAnchor, PopupGeometry, TextFragment};
use std::{cell::Cell, rc::Rc};

fn fixture() -> FragmentFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        renderer_name: Some("software".into()),
        mock_time: true,
        ..Default::default()
    })))
    .unwrap();
    let w = FragmentFixture::new().unwrap();
    slint_kit::install(&w);
    w.show().unwrap();
    settle(&w);
    w
}
fn settle(w: &FragmentFixture) {
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
        w.window().take_snapshot().unwrap();
    }
}
fn key(w: &FragmentFixture, key: impl Into<SharedString>) {
    let text = key.into();
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    w.window().dispatch_event(WindowEvent::KeyReleased { text });
    settle(w);
}
fn element(w: &FragmentFixture, role: AccessibleRole, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(w, label)
        .find(|e| e.accessible_role() == Some(role))
        .unwrap_or_else(|| panic!("missing {role:?} {label}"))
}
fn click(w: &FragmentFixture, role: AccessibleRole, label: &str) {
    element(w, role, label).mock_single_click(PointerEventButton::Left);
    settle(w);
}
fn at(w: &FragmentFixture, p: LogicalPosition) {
    w.window()
        .dispatch_event(WindowEvent::PointerMoved { position: p });
    w.window().dispatch_event(WindowEvent::PointerPressed {
        position: p,
        button: PointerEventButton::Left,
    });
    w.window().dispatch_event(WindowEvent::PointerReleased {
        position: p,
        button: PointerEventButton::Left,
    });
    settle(w);
}
fn anchor(token: i32, generation: i32) -> PopoverAnchor {
    PopoverAnchor {
        token,
        generation,
        bounds: PopupGeometry {
            x: 90.,
            y: 160.,
            width: 52.,
            height: 19.,
        },
    }
}
fn fragment(token: i32, generation: i32, label: &str, disabled: bool) -> TextFragment {
    TextFragment {
        item: OptionRow {
            token,
            generation,
            label: label.into(),
            disabled,
            ..Default::default()
        },
        suffix: ", ".into(),
    }
}
fn rows(items: Vec<TextFragment>) -> ModelRc<TextFragment> {
    ModelRc::new(VecModel::from(items))
}

#[test]
fn suffix_is_not_a_target_and_native_fields_keep_focus_drafts_and_identity() {
    let w = fixture();
    let word = element(&w, AccessibleRole::Button, "Heather");
    let p = word.absolute_position();
    at(
        &w,
        LogicalPosition::new(p.x + word.size().width + 1., p.y + word.size().height / 2.),
    );
    assert!(!w.get_editing());
    click(&w, AccessibleRole::Button, "Heather");
    assert!(w.get_editing());
    let ticket = w.get_ticket();
    click(&w, AccessibleRole::TextInput, "Annotation");
    key(&w, "é");
    assert_eq!(w.get_note(), "é");
    let make = |token, name| fragment(token, 1, name, false);
    w.set_fragments(rows(vec![make(20, "First"), make(10, "Heather")]));
    settle(&w);
    assert_eq!(w.get_ticket(), ticket);
    key(&w, "!");
    assert_eq!(w.get_note(), "é!");
    let count = Rc::new(Cell::new(0));
    let saved = count.clone();
    w.on_saved(move |ticket, name, note| {
        assert_eq!(
            (ticket.token, name.as_str(), note.as_str()),
            (10, "Heather", "é!")
        );
        saved.set(saved.get() + 1);
    });
    click(&w, AccessibleRole::Button, "Save");
    assert!(!w.get_editing());
    assert_eq!(count.get(), 1);
    assert!(!w.invoke_save(ticket));
    key(&w, Key::Return);
    assert!(w.get_editing(), "focus returned to the original fragment");
    key(&w, Key::Escape);
    assert!(!w.get_editing());
}

#[test]
fn stale_editor_preserves_drafts_and_refuses_replacement_removal_and_hide() {
    let w = fixture();
    let a = anchor(10, 1);
    assert!(w.invoke_edit(a.clone()));
    let first = w.get_ticket();
    w.set_heading("draft".into());
    w.set_fragments(rows(vec![fragment(10, 2, "Replacement", false)]));
    assert!(!w.invoke_save(first.clone()));
    assert_eq!(w.get_heading(), "draft");
    w.set_fragments(rows(vec![]));
    assert!(!w.invoke_save(first));
    assert!(w.get_editing());
    click(&w, AccessibleRole::Button, "Cancel");
    assert!(!w.get_editing());
    w.set_fragments(rows(vec![fragment(10, 1, "Restored", false)]));
    assert!(w.invoke_edit(a.clone()));
    let second = w.get_ticket();
    w.set_route_active(false);
    assert!(!w.invoke_save(second));
    settle(&w);
    assert!(!w.get_editing());
    w.set_route_active(true);
    assert!(w.invoke_edit(a));
    let before_hide = w.get_ticket();
    slint_kit::hide_window(&w).unwrap();
    w.show().unwrap();
    assert!(!w.invoke_save(before_hide));
    settle(&w);
    assert!(!w.get_editing());
}

#[test]
fn long_flow_has_one_tab_stop_and_directional_navigation_skips_literals() {
    let w = fixture();
    w.set_fragments(rows(
        (0..240)
            .map(|i| fragment(i, 3, &format!("word{i}"), i == 1))
            .collect(),
    ));
    settle(&w);
    for rtl in [false, true] {
        let mut theme = w.get_theme();
        theme.direction = if rtl {
            slint_kit::ui::ChoiceFlow::RightToLeft
        } else {
            slint_kit::ui::ChoiceFlow::LeftToRight
        };
        w.set_theme(theme);
        click(&w, AccessibleRole::Button, "word0");
        key(&w, Key::Escape);
        key(&w, if rtl { Key::LeftArrow } else { Key::RightArrow });
        key(&w, Key::Return);
        assert_eq!(w.get_ticket().token, 2);
        key(&w, Key::Escape);
        key(&w, Key::End);
        key(&w, Key::Return);
        assert_eq!(w.get_ticket().token, 239);
        key(&w, Key::Escape);
        key(&w, Key::Home);
        key(&w, Key::Tab);
        w.set_status("".into());
        key(&w, Key::Space);
        assert_eq!(
            w.get_status(),
            "Notebook read",
            "one Tab exits 240 fragments"
        );
        assert!(!w.get_editing());
    }
}

#[test]
fn reentrant_open_dismiss_and_restore_cannot_retire_a_new_ticket() {
    let w = fixture();
    let first = anchor(10, 1);
    let next = anchor(11, 1);
    let weak = w.as_weak();
    let once = Rc::new(Cell::new(true));
    let arm = once.clone();
    w.on_opened(move || {
        if arm.replace(false) {
            let w = weak.unwrap();
            w.invoke_cancel();
            assert!(w.invoke_edit(anchor(11, 1)));
        }
    });
    assert!(!w.invoke_edit(first.clone()));
    assert_eq!(w.get_heading(), "ferns");
    let old = w.get_ticket();
    let weak = w.as_weak();
    w.on_dismissed(move |reason| {
        assert_eq!(reason, CloseReason::Programmatic);
        assert!(weak.unwrap().invoke_edit(anchor(10, 1)));
    });
    w.invoke_cancel();
    assert!(w.get_editing());
    assert_eq!(w.get_ticket().token, 10);
    assert!(!w.invoke_save(old));
    w.on_dismissed(|_| {});
    let weak = w.as_weak();
    w.on_focus_returned(move |_| {
        assert!(weak.unwrap().invoke_edit(next.clone()));
    });
    w.invoke_cancel();
    settle(&w);
    assert_eq!(w.get_heading(), "ferns");
    w.on_focus_returned(|_| {});
    assert!(w.invoke_save(w.get_ticket()));
    assert!(!w.get_editing());
}

#[test]
fn open_editor_keeps_native_traversal_and_closes_on_a_viewport_change() {
    let w = fixture();
    click(&w, AccessibleRole::Button, "Heather");
    click(&w, AccessibleRole::TextInput, "Heading");
    key(&w, Key::Tab);
    key(&w, "note");
    assert_eq!(w.get_note(), "note");
    for _ in 0..3 {
        key(&w, Key::Tab);
    }
    key(&w, "x");
    assert!(w.get_editing());
    assert!(
        w.get_heading().contains('x'),
        "Tab wraps to the first editor"
    );
    key(&w, Key::Backtab);
    key(&w, Key::Space);
    assert!(
        !w.get_editing(),
        "backward traversal wraps from first editor to Save"
    );
    assert!(w.invoke_edit(anchor(10, 1)));
    let old = w.get_ticket();
    w.window().set_size(slint::LogicalSize::new(460., 410.));
    settle(&w);
    assert!(!w.get_editing());
    assert!(!w.invoke_save(old));
    assert!(w.invoke_edit(anchor(10, 1)));
    at(&w, LogicalPosition::new(2., 2.));
    assert!(!w.get_editing());
}

#[test]
fn scrolling_the_source_keeps_the_window_anchor_and_retained_editor() {
    use i_slint_backend_testing::ElementQuery;
    use slint_kit::ui::{DropdownPlacement, KitFieldService, PopupRequest, PopupViewport};
    let w = fixture();
    w.set_fragments(rows(
        (0..240)
            .map(|i| fragment(i, 3, &format!("fragment{i}"), false))
            .collect(),
    ));
    settle(&w);
    let opener = element(&w, AccessibleRole::Button, "fragment0");
    let original = opener.absolute_position();
    let size = opener.size();
    click(&w, AccessibleRole::Button, "fragment0");
    let panel = ElementQuery::from_root(&w)
        .match_inherits("PopoverPanel")
        .find_first()
        .unwrap();
    let dims = w.window().size().to_logical(w.window().scale_factor());
    let bounds = w.global::<KitFieldService>().invoke_place_popup(
        PopupRequest {
            x: original.x,
            y: original.y,
            anchor_width: size.width,
            anchor_height: size.height,
            width: panel.size().width,
            height: panel.size().height,
            gap: w.get_theme().spacing.unit,
            placement: DropdownPlacement::Auto,
        },
        PopupViewport {
            width: dims.width,
            height: dims.height,
        },
    );
    let input = element(&w, AccessibleRole::TextInput, "Annotation");
    let local = input.absolute_position();
    let hit = LogicalPosition::new(
        original.x + bounds.x + local.x + input.size().width / 2.,
        original.y + bounds.y + local.y + input.size().height / 2.,
    );
    at(&w, hit);
    key(&w, "a");
    w.set_document_offset(-240.);
    settle(&w);
    assert!(opener.absolute_position().y < original.y - 120.);
    assert_eq!(w.get_note(), "a");
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(1200));
    at(&w, hit);
    assert!(w.get_editing());
    assert_eq!(w.get_note(), "a", "pointer reentry preserves the draft");
    key(&w, Key::RightArrow);
    key(&w, "b");
    assert_eq!(
        w.get_note(),
        "ab",
        "same window coordinate still hits the retained editor"
    );
}

#[test]
fn a_recycled_delegate_cannot_finish_an_old_pointer_gesture() {
    let w = fixture();
    let button = element(&w, AccessibleRole::Button, "Heather");
    let p = button.absolute_position();
    let size = button.size();
    let position = LogicalPosition::new(p.x + size.width / 2., p.y + size.height / 2.);
    w.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    w.set_fragments(rows(vec![fragment(81, 2, "Heather", false)]));
    w.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    settle(&w);
    assert!(!w.get_editing());
    element(&w, AccessibleRole::Button, "Heather").invoke_accessible_default_action();
    assert!(w.get_editing());
    assert_eq!((w.get_ticket().token, w.get_ticket().generation), (81, 2));
}

#[test]
fn wrapped_fragment_keeps_its_leading_edge_and_places_punctuation_after_the_last_glyph() {
    use i_slint_backend_testing::ElementQuery;
    let w = fixture();
    w.window().set_size(slint::LogicalSize::new(360., 620.));
    let mut theme = w.get_theme();
    theme.text_scale = 2.;
    theme.reduced_motion = true;
    let label = "polychromatographieinterdisciplinaire";
    let mut long = fragment(7, 1, label, false);
    long.suffix = ". ".into();
    w.set_fragments(rows(vec![long]));
    for rtl in [false, true] {
        theme.direction = if rtl {
            slint_kit::ui::ChoiceFlow::RightToLeft
        } else {
            slint_kit::ui::ChoiceFlow::LeftToRight
        };
        w.set_theme(theme.clone());
        settle(&w);
        let word = element(&w, AccessibleRole::Button, label);
        let suffix = ElementQuery::from_root(&w)
            .match_id("KitTextFlow::suffix")
            .find_first()
            .unwrap();
        let p = word.absolute_position();
        let size = word.size();
        let s = suffix.absolute_position();
        assert!(s.y > p.y && p.x >= 0. && p.x + size.width <= 360.);
        let image = w.window().take_snapshot().unwrap();
        let mut ink = Vec::new();
        for y in 0..image.height() {
            for x in 0..image.width() {
                // This software fixture runs at one physical pixel per logical pixel.
                let lx = f64::from(x);
                let ly = f64::from(y);
                if ly < f64::from(s.y)
                    || ly >= f64::from(p.y + size.height)
                    || lx < f64::from(p.x)
                    || lx >= f64::from(p.x + size.width)
                {
                    continue;
                }
                let index = usize::try_from(y * image.width() + x).unwrap();
                let px = image.as_slice()[index];
                if (px.r, px.g, px.b)
                    == (
                        theme.accent.red(),
                        theme.accent.green(),
                        theme.accent.blue(),
                    )
                {
                    ink.push(lx);
                }
            }
        }
        let min = ink
            .iter()
            .copied()
            .reduce(f64::min)
            .expect("last line has visible glyphs");
        let max = ink.iter().copied().reduce(f64::max).unwrap();
        let (leading_gap, punctuation_gap) = if rtl {
            (
                f64::from(p.x + size.width) - max,
                min - f64::from(s.x + suffix.size().width),
            )
        } else {
            (min - f64::from(p.x), f64::from(s.x) - max)
        };
        assert!(
            (-1.0..8.0).contains(&leading_gap),
            "leading gap {leading_gap}, rtl={rtl}"
        );
        assert!(
            (-1.0..8.0).contains(&punctuation_gap),
            "suffix gap {punctuation_gap}, rtl={rtl}"
        );
        at(
            &w,
            LogicalPosition::new(
                s.x + suffix.size().width / 2.,
                s.y + suffix.size().height / 2.,
            ),
        );
        assert!(!w.get_editing());
        at(
            &w,
            LogicalPosition::new(p.x + size.width / 2., p.y + suffix.size().height / 2.),
        );
        assert!(w.get_editing());
        w.invoke_cancel();
    }
}
