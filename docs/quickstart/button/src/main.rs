use slint::ComponentHandle;
use std::cell::Cell;
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let count = Cell::new(0_i32);
    let weak = app.as_weak();
    app.on_increment(move || {
        if let Some(app) = weak.upgrade() {
            let next = count.get().checked_add(1).expect("counter capacity");
            count.set(next);
            app.set_count(next);
        }
    });
    app.run()
}
