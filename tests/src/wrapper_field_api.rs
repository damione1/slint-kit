// SPDX-License-Identifier: GPL-3.0-or-later
use super::WrapperFieldApiFixture;
use slint::ComponentHandle;

fn fixture() -> WrapperFieldApiFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = WrapperFieldApiFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window
}

#[test]
fn number_stepper_forwards_localised_validation_messages() {
    let window = fixture();
    assert!(!window.invoke_report_stepper());
    assert_eq!(window.get_stepper_message(), "Obligatoire");
    window.set_stepper_value("1".into());
    assert!(!window.invoke_report_stepper());
    assert_eq!(window.get_stepper_message(), "Trop petit");
    window.set_stepper_value("9".into());
    assert!(!window.invoke_report_stepper());
    assert_eq!(window.get_stepper_message(), "Trop grand");
    window.set_stepper_value("5".into());
    assert!(window.invoke_report_stepper());
}

#[test]
fn label_less_number_stepper_keeps_an_accessible_name() {
    use i_slint_backend_testing::{AccessibleRole, ElementQuery};
    let window = fixture();
    let named = ElementQuery::from_root(&window)
        .match_accessible_role(AccessibleRole::Spinbox)
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Quantité"))
        .find_first();
    assert!(
        named.is_some(),
        "stepper uses accessible-name without a visible label"
    );
}

#[test]
fn color_field_and_picker_forward_the_required_message() {
    let window = fixture();
    assert!(!window.invoke_report_color());
    assert_eq!(window.get_color_message(), "Couleur obligatoire");
    assert!(!window.invoke_report_picker());
    assert_eq!(window.get_picker_message(), "Choisir une couleur");
}
