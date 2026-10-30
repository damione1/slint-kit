// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{Ownership, SelectOption, SelectionIntent};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Destination { Alpine, Desert, Coast }
impl Destination {
    fn label(self) -> &'static str {
        match self { Self::Alpine => "Alpine morning", Self::Desert => "Desert light", Self::Coast => "Coastal calm" }
    }
}

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    app.set_alpine(landscape(Landscape::Alpine));
    app.set_desert(landscape(Landscape::Desert));
    app.set_coast(landscape(Landscape::Coast));
    let selection = slint_kit::bind_select!(ui, app, vec![
        SelectOption { key: Destination::Alpine, label: "Alpine morning".into(), description: "A quiet trail, an open sky, and a little room to think.".into(), disabled: false },
        SelectOption { key: Destination::Desert, label: "Desert light".into(), description: "Find fresh perspective in the warm light of the desert.".into(), disabled: false },
        SelectOption { key: Destination::Coast, label: "Coastal calm".into(), description: "Slow down by the water and make space for your next idea.".into(), disabled: false },
    ], Some(Destination::Alpine), Ownership::Local);
    let weak = app.as_weak();
    selection.on_intent(move |intent| {
        if let Some(app) = weak.upgrade() {
            app.set_selected_label(match intent {
                SelectionIntent::Commit(key) => key.label(),
                SelectionIntent::Clear => "No selection",
            }.into());
        }
    });
    selection.open();
    let persistent = selection.clone();
    app.on_select_activate(move |token, generation| {
        persistent.activate(token, generation);
        persistent.open();
    });
    app.run()
}

// Original landscape illustrations, rendered locally; no external asset or network dependency.
#[derive(Clone, Copy)]
enum Landscape { Alpine, Desert, Coast }
fn landscape(scene: Landscape) -> slint::Image {
    let (sky, ridge, near, sun): ([u8; 3], [u8; 3], [u8; 3], [u8; 3]) = match scene {
        Landscape::Alpine => ([204, 230, 239], [90, 139, 148], [30, 81, 95], [255, 246, 212]),
        Landscape::Desert => ([249, 222, 192], [205, 134, 111], [136, 74, 78], [255, 250, 222]),
        Landscape::Coast => ([210, 229, 235], [99, 158, 165], [43, 107, 123], [255, 246, 215]),
    };
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(640, 260);
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let x = i32::try_from(index % 640).expect("image width fits i32");
        let y = i32::try_from(index / 640).expect("image height fits i32");
        let distant = 145 - (130 - (x - 210).abs()).max(0) / 2;
        let foreground = 222 - (180 - (x - 495).abs()).max(0) / 3;
        let color = if (x - 493).pow(2) + (y - 62).pow(2) < 28 * 28 {
            sun
        } else if y > foreground { near } else if y > distant { ridge } else { sky };
        *pixel = slint::Rgb8Pixel::new(color[0], color[1], color[2]);
    }
    slint::Image::from_rgb8(buffer)
}
