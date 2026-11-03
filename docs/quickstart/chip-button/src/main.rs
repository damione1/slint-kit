// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
mod ui { slint::include_modules!(); }
use ui::OrderFilter;

/// Every filter is listed: a new one breaks the build here until it has a label.
fn status(filter: OrderFilter) -> &'static str {
    match filter {
        OrderFilter::All => "Showing all orders",
        OrderFilter::AwaitingShipment => "Showing orders awaiting shipment",
        OrderFilter::InTransit => "Showing orders in transit",
        OrderFilter::Delayed => "Showing delayed orders",
        OrderFilter::Returned => "Showing returned orders",
    }
}
fn show(app: &ui::App, filter: OrderFilter) {
    app.set_active(filter);
    app.set_status(status(filter).into());
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    // The host owns the active filter; a chip only requests it.
    show(&app, OrderFilter::AwaitingShipment);
    let weak = app.as_weak();
    app.on_choose(move |filter| { if let Some(app) = weak.upgrade() { show(&app, filter); } });
    app.run()
}
