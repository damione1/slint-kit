// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{RangeController, RangeThumb, DecimalValue, DecimalStep, NumericBounds, OrderedRange, StepDirection};
use std::{cell::RefCell, rc::Rc};
mod ui { slint::include_modules!(); }
fn value(n: i64) -> DecimalValue { DecimalValue::from_integer(n) }
fn thumb(t: ui::RangeThumb) -> RangeThumb { match t { ui::RangeThumb::Lower => RangeThumb::Start, ui::RangeThumb::Upper => RangeThumb::End } }
fn publish(app: &ui::App, range: &RangeController) {
    app.set_lower(range.value().start().to_string().parse().expect("integral percent"));
    app.set_upper(range.value().end().to_string().parse().expect("integral percent"));
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let range = Rc::new(RefCell::new(RangeController::new(NumericBounds::new(value(0), value(100)).unwrap(), OrderedRange::new(value(0), value(65)).unwrap(), DecimalStep::new(value(5)).unwrap()).unwrap()));
    publish(&app, &range.borrow());
    let state = range.clone(); let weak = app.as_weak();
    app.on_position(move |t, position| { if !position.is_finite() { return; }
        let quantized = value((position.clamp(0.0, 1.0) * 100.0).round() as i64);
        let _ = state.borrow_mut().set(thumb(t), quantized);
        if let Some(app) = weak.upgrade() { publish(&app, &state.borrow()); }
    });
    let state = range.clone(); let weak = app.as_weak();
    app.on_navigate(move |t, step| {
        let t = thumb(t);
        { let mut range = state.borrow_mut();
          let _ = match step {
            ui::NavigationStep::Next | ui::NavigationStep::PageNext => range.step(t, StepDirection::Increment),
            ui::NavigationStep::Previous | ui::NavigationStep::PagePrevious => range.step(t, StepDirection::Decrement),
            ui::NavigationStep::First => { let v = if t == RangeThumb::Start { value(0) } else { range.value().start() }; range.set(t, v) },
            ui::NavigationStep::Last => { let v = if t == RangeThumb::End { value(100) } else { range.value().end() }; range.set(t, v) },
          };
        }
        if let Some(app) = weak.upgrade() { publish(&app, &state.borrow()); }
    });
    let state = range.clone(); app.on_begin(move || state.borrow_mut().begin_drag());
    let state = range.clone(); app.on_commit(move || state.borrow_mut().commit_drag());
    let weak = app.as_weak(); app.on_cancel(move || { range.borrow_mut().cancel_drag(); if let Some(app) = weak.upgrade() { publish(&app, &range.borrow()); } });
    app.run()
}
