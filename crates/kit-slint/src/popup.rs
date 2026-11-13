// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ui;
use kit_core::{Insets, PopupPreference, Rect, Size, place_popup_with};

fn placement(request: ui::PopupRequest, viewport: ui::PopupViewport) -> ui::PopupGeometry {
    let bounds = place_popup_with(
        Rect {
            x: request.x,
            y: request.y,
            width: request.anchor_width,
            height: request.anchor_height,
        },
        Size {
            width: request.width,
            height: request.height,
        },
        Rect {
            width: viewport.width,
            height: viewport.height,
            ..Default::default()
        },
        Insets {
            top: request.gap,
            right: request.gap,
            bottom: request.gap,
            left: request.gap,
        },
        request.gap,
        match request.placement {
            ui::DropdownPlacement::Auto => PopupPreference::Auto,
            ui::DropdownPlacement::Below => PopupPreference::Below,
            ui::DropdownPlacement::Above => PopupPreference::Above,
        },
    )
    .bounds;
    ui::PopupGeometry {
        x: bounds.x - request.x,
        y: bounds.y - request.y,
        width: bounds.width,
        height: bounds.height,
    }
}

pub(crate) fn install<W>(window: &W)
where
    W: slint::ComponentHandle + 'static,
    for<'a> ui::KitFieldService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitFieldService>();
    service.on_place_popup(placement);
    let weak = window.as_weak();
    service.on_popup_viewport(move || {
        weak.upgrade()
            .map_or_else(ui::PopupViewport::default, |window| {
                let size = window
                    .window()
                    .size()
                    .to_logical(window.window().scale_factor());
                ui::PopupViewport {
                    width: size.width,
                    height: size.height,
                }
            })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_popup_bounds_flip_shift_and_clamp_in_the_window_space() {
        let request = ui::PopupRequest {
            x: 280.,
            y: 340.,
            anchor_width: 36.,
            anchor_height: 32.,
            width: 680.,
            height: 720.,
            gap: 4.,
            placement: ui::DropdownPlacement::Auto,
        };
        let result = placement(
            request,
            ui::PopupViewport {
                width: 320.,
                height: 400.,
            },
        );
        assert_eq!(
            (result.x, result.y, result.width, result.height),
            (-276., -336., 312., 332.)
        );
    }
}
