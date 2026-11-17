// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{BaseDataFixture, ScrollReversalFixture, ScrollTransactionsFixture};
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, ModelRc, VecModel,
    platform::{Key, WindowEvent},
};
use std::time::Duration;

fn init() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}

fn key(window: &impl ComponentHandle, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}

fn frame(window: &impl ComponentHandle, ms: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(ms));
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
}

fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.1, "{actual} != {expected}");
}

#[test]
fn collection_restoration_survives_initial_and_empty_model_layout() {
    init();
    for already_shown in [false, true] {
        let window = BaseDataFixture::new().unwrap();
        slint_kit::install(&window);
        if already_shown {
            window.show().unwrap();
            frame(&window, 0);
        }
        window.set_columns(ModelRc::new(VecModel::from(crate::base_data::column_set())));
        window.set_rows(ModelRc::new(VecModel::from(crate::base_data::rows(
            1, 1000,
        ))));
        window.set_items(ModelRc::new(VecModel::from(crate::base_data::items(1000))));
        window.set_lines(ModelRc::new(VecModel::from(crate::base_data::lines(1000))));
        window.invoke_restore_table_scroll(0., -3200.5);
        window.invoke_restore_content_scroll(0., -3200.5);
        window.invoke_restore_line_scroll(0., -3200.5);
        window.show().unwrap();
        frame(&window, 0);
        near(window.get_table_scroll(), -3200.5);
        near(window.get_content_scroll(), -3200.5);
        near(window.get_line_scroll(), -3200.5);
    }
}

#[test]
fn rapid_scroll_keys_accumulate_and_home_supersedes_end() {
    init();
    let window = ScrollTransactionsFixture::new().unwrap();
    window.show().unwrap();
    frame(&window, 0);
    window.invoke_focus_area();
    for _ in 0..5 {
        key(&window, Key::DownArrow);
        frame(&window, 30);
    }
    frame(&window, 1000);
    near(window.get_offset_y(), -200.);
    window.invoke_restore(0., 0.);
    frame(&window, 0);
    key(&window, Key::End);
    key(&window, Key::Home);
    frame(&window, 1000);
    near(window.get_offset_y(), 0.);
}

#[test]
fn rapid_strip_reversal_keeps_the_active_item_visible() {
    init();
    let window = ScrollReversalFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_options(ModelRc::new(VecModel::from(
        (0..15)
            .map(|index| slint_kit::ui::OptionRow {
                token: index + 1,
                generation: 1,
                label: format!("Item {index:02}").into(),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    )));
    window.show().unwrap();
    frame(&window, 0);
    for component in ["KitTabs", "KitMenuBar"] {
        if component == "KitTabs" {
            window.invoke_focus_tabs();
        } else {
            window.invoke_focus_menu();
        }
        key(&window, Key::End);
        frame(&window, 0);
        key(&window, Key::Home);
        crate::settle_scroll(&window);
        assert_eq!((window.get_tab_active(), window.get_menu_active()), (0, 0));
        let strip = ElementQuery::from_root(&window)
            .match_inherits(component)
            .find_first()
            .unwrap();
        let first = strip
            .query_descendants()
            .match_inherits("KitNavigationAction")
            .match_predicate(|item| item.accessible_label().as_deref() == Some("Item 00"))
            .find_first()
            .expect("the active first item must remain visible after reversal");
        assert!(first.absolute_position().x >= strip.absolute_position().x - 0.1);
    }
}
