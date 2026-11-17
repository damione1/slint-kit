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

struct CollectionProbe {
    component: &'static str,
    restore: fn(&BaseDataFixture, f32, f32),
    reveal: fn(&BaseDataFixture, i32),
    offset: fn(&BaseDataFixture) -> f32,
    first: fn(&BaseDataFixture) -> i32,
    end: fn(&BaseDataFixture) -> bool,
    set_active: fn(&BaseDataFixture, i32),
    active: fn(&BaseDataFixture) -> i32,
    reveal_active: fn(&BaseDataFixture),
}
const COLLECTIONS: [CollectionProbe; 3] = [
    CollectionProbe {
        component: "KitDataTable",
        restore: BaseDataFixture::invoke_restore_table_scroll,
        reveal: BaseDataFixture::invoke_scroll_table_to,
        offset: BaseDataFixture::get_table_scroll,
        first: BaseDataFixture::get_table_first_visible,
        end: BaseDataFixture::get_table_end_near,
        set_active: BaseDataFixture::set_table_active,
        active: BaseDataFixture::get_table_active,
        reveal_active: BaseDataFixture::invoke_reveal_table,
    },
    CollectionProbe {
        component: "KitContentList",
        restore: BaseDataFixture::invoke_restore_content_scroll,
        reveal: BaseDataFixture::invoke_scroll_content_to,
        offset: BaseDataFixture::get_content_scroll,
        first: BaseDataFixture::get_content_first_visible,
        end: BaseDataFixture::get_content_end_near,
        set_active: BaseDataFixture::set_content_active,
        active: BaseDataFixture::get_content_active,
        reveal_active: BaseDataFixture::invoke_reveal_content,
    },
    CollectionProbe {
        component: "KitVirtualTextView",
        restore: BaseDataFixture::invoke_restore_line_scroll,
        reveal: BaseDataFixture::invoke_scroll_line_to,
        offset: BaseDataFixture::get_line_scroll,
        first: BaseDataFixture::get_line_first_visible,
        end: BaseDataFixture::get_line_end_near,
        set_active: BaseDataFixture::set_line_active,
        active: BaseDataFixture::get_line_active,
        reveal_active: BaseDataFixture::invoke_reveal_line,
    },
];

fn populate(window: &BaseDataFixture, count: i32) {
    window.set_columns(ModelRc::new(VecModel::from(crate::base_data::column_set())));
    window.set_rows(ModelRc::new(VecModel::from(crate::base_data::rows(
        1, count,
    ))));
    window.set_items(ModelRc::new(VecModel::from(crate::base_data::items(count))));
    window.set_lines(ModelRc::new(VecModel::from(crate::base_data::lines(count))));
}

#[test]
fn collection_restoration_survives_initial_and_empty_model_layout() {
    init();
    for dark in [false, true] {
        for scale in [1., 2.] {
            for already_shown in [false, true] {
                let window = BaseDataFixture::new().unwrap();
                slint_kit::install(&window);
                let mut theme = if dark {
                    window.get_dark_theme()
                } else {
                    window.get_theme()
                };
                theme.text_scale = scale;
                window.set_theme(theme);
                if already_shown {
                    window.show().unwrap();
                    frame(&window, 0);
                }
                populate(&window, 20_000);
                for probe in &COLLECTIONS {
                    (probe.reveal)(&window, 19_900);
                    (probe.restore)(&window, 0., -1600.25);
                    (probe.restore)(&window, 0., -3200.5);
                    near((probe.offset)(&window), -3200.5);
                }
                window.show().unwrap();
                crate::settle_scroll(&window);
                for probe in &COLLECTIONS {
                    near((probe.offset)(&window), -3200.5);
                    let rows = crate::base_data::delegates(&window, probe.component);
                    assert!(
                        !rows.is_empty() && rows.len() < 40,
                        "{} stays virtual",
                        probe.component
                    );
                }
                crate::choice_presentation::save_capture(
                    &window.window().take_snapshot().unwrap(),
                    &format!("scroll-restore-dark-{dark}-scale-{scale}-shown-{already_shown}"),
                );
            }
        }
    }
}

#[test]
fn collection_requests_supersede_each_other_without_late_restoration() {
    init();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    populate(&window, 1000);
    window.show().unwrap();
    frame(&window, 0);
    for probe in &COLLECTIONS {
        (probe.restore)(&window, 0., -3200.5);
        (probe.reveal)(&window, 900);
        (probe.restore)(&window, 0., -800.25);
        crate::settle_scroll(&window);
        near((probe.offset)(&window), -800.25);
        for invalid in [-1, 1000] {
            (probe.set_active)(&window, invalid);
            (probe.reveal_active)(&window);
            crate::settle_scroll(&window);
            near((probe.offset)(&window), -800.25);
            assert_eq!((probe.active)(&window), invalid);
        }
        (probe.restore)(&window, 0., -3200.5);
        (probe.reveal)(&window, 900);
        crate::settle_scroll(&window);
        assert!((probe.first)(&window) > 880);
        for target in [100, 900, 500, 950] {
            (probe.reveal)(&window, target);
            crate::settle_scroll(&window);
            let first = (probe.first)(&window);
            assert!(
                first <= target && first > target - 20,
                "{}: target {target}, first {first}",
                probe.component
            );
        }
        (probe.restore)(&window, 0., 100.);
        near((probe.offset)(&window), 0.);
        (probe.restore)(&window, 0., -1_000_000.);
        frame(&window, 0);
        assert!((probe.end)(&window));
        let end = (probe.offset)(&window);
        frame(&window, 1000);
        near((probe.offset)(&window), end);
        (probe.restore)(&window, 0., -3200.5);
        frame(&window, 0);
        let viewport = ElementQuery::from_root(&window)
            .match_inherits(probe.component)
            .find_first()
            .unwrap()
            .query_descendants()
            .match_type_name("Flickable")
            .find_first()
            .unwrap();
        let origin = viewport.absolute_position();
        window
            .window()
            .dispatch_event(WindowEvent::PointerScrolled {
                position: slint::LogicalPosition::new(origin.x + 40., origin.y + 40.),
                delta_x: 0.,
                delta_y: -80.,
            });
        crate::settle_scroll(&window);
        assert!((probe.offset)(&window) < -3201.);
        let moved = (probe.offset)(&window);
        frame(&window, 1000);
        near((probe.offset)(&window), moved);
    }
    populate(&window, 1);
    frame(&window, 0);
    for probe in &COLLECTIONS {
        near((probe.offset)(&window), 0.);
    }
}

#[test]
fn rapid_scroll_keys_accumulate_and_home_supersedes_end() {
    init();
    for dark in [false, true] {
        for scale in [1., 2.] {
            for reduced in [false, true] {
                let window = ScrollTransactionsFixture::new().unwrap();
                let mut theme = if dark {
                    window.get_dark_theme()
                } else {
                    window.get_theme()
                };
                theme.text_scale = scale;
                theme.reduced_motion = reduced;
                window.set_theme(theme);
                window.show().unwrap();
                frame(&window, 0);
                window.invoke_focus_area();
                for _ in 0..5 {
                    key(&window, Key::DownArrow);
                    frame(&window, 30);
                }
                near(window.get_offset_y(), -200.);
                window.invoke_restore(0., 0.);
                frame(&window, 0);
                key(&window, Key::End);
                key(&window, Key::Home);
                near(window.get_offset_y(), 0.);
                // An explicit animated helper can still be pending at zero.
                window.invoke_to_end();
                key(&window, Key::Home);
                frame(&window, 1000);
                near(window.get_offset_y(), 0.);
            }
        }
    }
}

#[test]
fn direct_keys_follow_pointer_restoration_and_content_changes_on_both_axes() {
    init();
    let window = ScrollTransactionsFixture::new().unwrap();
    window.set_axes(crate::ScrollAxes::Both);
    window.show().unwrap();
    frame(&window, 0);
    window.invoke_focus_area();
    window.invoke_restore(-120., -200.);
    key(&window, Key::DownArrow);
    near(window.get_offset_x(), -120.);
    near(window.get_offset_y(), -240.);
    key(&window, Key::RightArrow);
    near(window.get_offset_x(), -160.);
    near(window.get_offset_y(), -240.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: slint::LogicalPosition::new(100., 100.),
            delta_x: 0.,
            delta_y: -80.,
        });
    crate::settle_scroll(&window);
    let moved = window.get_offset_y();
    assert!(moved < -240.);
    key(&window, Key::DownArrow);
    near(window.get_offset_y(), moved - 40.);
    window.invoke_restore(-100., -100.);
    key(&window, Key::DownArrow);
    near(window.get_offset_y(), -140.);
    key(&window, Key::RightArrow);
    near(window.get_offset_x(), -140.);
    window.set_extent(200.);
    frame(&window, 0);
    near(window.get_offset_y(), 0.);
    near(window.get_offset_x(), 0.);
    window.set_extent(2000.);
    frame(&window, 0);
    key(&window, Key::DownArrow);
    frame(&window, 1000);
    near(window.get_offset_y(), -40.);
    window.set_axes(crate::ScrollAxes::Horizontal);
    frame(&window, 0);
    for _ in 0..5 {
        key(&window, Key::RightArrow);
        frame(&window, 30);
    }
    near(window.get_offset_x(), -200.);
}

#[test]
fn rapid_strip_reversal_keeps_the_active_item_visible() {
    init();
    for dark in [false, true] {
        for scale in [1., 2.] {
            for reduced in [false, true] {
                for flow in [
                    crate::ChoiceFlow::LeftToRight,
                    crate::ChoiceFlow::RightToLeft,
                ] {
                    for orientation in [
                        i_slint_backend_testing::Orientation::Horizontal,
                        i_slint_backend_testing::Orientation::Vertical,
                    ] {
                        let window = ScrollReversalFixture::new().unwrap();
                        slint_kit::install(&window);
                        let mut theme = if dark {
                            window.get_dark_theme()
                        } else {
                            window.get_theme()
                        };
                        theme.text_scale = scale;
                        theme.reduced_motion = reduced;
                        theme.direction = flow;
                        window.set_theme(theme);
                        window.set_orientation(orientation);
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
                        type StripProbe = (
                            &'static str,
                            fn(&ScrollReversalFixture),
                            fn(&ScrollReversalFixture) -> i32,
                        );
                        let strips: [StripProbe; 2] = [
                            (
                                "KitTabs",
                                ScrollReversalFixture::invoke_focus_tabs,
                                ScrollReversalFixture::get_tab_active,
                            ),
                            (
                                "KitMenuBar",
                                ScrollReversalFixture::invoke_focus_menu,
                                ScrollReversalFixture::get_menu_active,
                            ),
                        ];
                        for (component, focus, active) in strips {
                            focus(&window);
                            key(&window, Key::End);
                            key(&window, Key::Home);
                            crate::settle_scroll(&window);
                            key(&window, Key::End);
                            frame(&window, 0);
                            key(&window, Key::Home);
                            key(&window, Key::Home);
                            crate::settle_scroll(&window);
                            assert_eq!(active(&window), 0);
                            let strip = ElementQuery::from_root(&window)
                                .match_inherits(component)
                                .find_first()
                                .unwrap();
                            let first = strip
                                .query_descendants()
                                .match_inherits("KitNavigationAction")
                                .match_predicate(|item| {
                                    item.accessible_label().as_deref() == Some("Item 00")
                                })
                                .find_first()
                                .expect("the active first item must remain visible after reversal");
                            let viewport = strip
                                .query_descendants()
                                .match_type_name("Flickable")
                                .find_first()
                                .unwrap();
                            let point = first.absolute_position();
                            let origin = viewport.absolute_position();
                            assert!(point.x >= origin.x - 0.1 && point.y >= origin.y - 0.1);
                            assert!(
                                point.x + first.size().width
                                    <= origin.x + viewport.size().width + 0.1
                            );
                            assert!(
                                point.y + first.size().height
                                    <= origin.y + viewport.size().height + 0.1
                            );
                        }
                        crate::choice_presentation::save_capture(
                            &window.window().take_snapshot().unwrap(),
                            &format!(
                                "scroll-reversal-dark-{dark}-scale-{scale}-reduced-{reduced}-{flow:?}-{orientation:?}"
                            ),
                        );
                    }
                }
            }
        }
    }
}
