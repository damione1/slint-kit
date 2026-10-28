// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{ComponentHandle, platform::PointerEventButton};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[test]
fn localized_ambiguity_labels_keep_choices_explicit_and_typed() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DateTimeLocalizationFixture::new().unwrap();
    let choices = Rc::new(RefCell::new(Vec::new()));
    let output = choices.clone();
    let starts = Rc::new(Cell::new(0));
    let observed = starts.clone();
    window.on_started(move || observed.set(observed.get() + 1));
    let observed = starts.clone();
    window.on_chosen(move |value| {
        assert!(observed.get() > output.borrow().len());
        output.borrow_mut().push(value);
    });
    window.show().unwrap();
    assert!(choices.borrow().is_empty());
    for (label, choice) in [
        ("Première occurrence", AmbiguousTimeChoice::Earlier),
        ("Seconde occurrence", AmbiguousTimeChoice::Later),
    ] {
        ElementQuery::from_root(&window)
            .match_inherits("KitButton")
            .match_predicate(move |element| element.accessible_label().as_deref() == Some(label))
            .find_first()
            .expect("localized ambiguity action")
            .mock_single_click(PointerEventButton::Left);
        assert_eq!(choices.borrow().last(), Some(&choice));
    }
    for label in ["Date civile", "Heure locale", "Fuseau horaire"] {
        assert!(ElementQuery::from_root(&window).match_predicate(move|element|element.accessible_label().as_deref()==Some(label)).find_first().is_some(),"{label}");
    }
    window.set_access(FieldAccess::ReadOnly);
    let before = choices.borrow().len();
    ElementQuery::from_root(&window)
        .match_inherits("KitButton")
        .match_predicate(|element| {
            element.accessible_label().as_deref() == Some("Seconde occurrence")
        })
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(choices.borrow().len(), before);
}
