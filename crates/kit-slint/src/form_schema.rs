// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{
    FieldSpec, FieldSpecError, FormBinding, FormErrors, FormRegistrationError, FormSubmission,
    FormSubmitError, ui,
};
use kit_core::{FieldError, FieldFailure, SubmissionTicket};
use slint::{ComponentHandle, SharedString};
use std::rc::Rc;

/// Public control ports, declared once beside the field's stable key and codec.
pub struct FormFieldPorts<W> {
    pub snapshot: fn(&W) -> ui::FormFieldView,
    pub report: fn(&W) -> bool,
    pub focus: fn(&W) -> bool,
    pub reset: fn(&W),
    pub set_error: fn(&W, SharedString),
}
type ReadValue<W, K, T> = Rc<dyn Fn(&W) -> Result<T, FormErrors<K>>>;
/// A schema value; no native draft or window is retained by this handle.
pub struct FormValue<W, K, T> {
    read: ReadValue<W, K, T>,
}
impl<W: 'static, K: 'static, T: 'static> FormValue<W, K, T> {
    /// Parse both values once and retain errors from both fields in declaration order.
    pub fn zip<U: 'static>(self, other: FormValue<W, K, U>) -> FormValue<W, K, (T, U)> {
        FormValue {
            read: Rc::new(
                move |window| match ((self.read)(window), (other.read)(window)) {
                    (Ok(left), Ok(right)) => Ok((left, right)),
                    (Err(error), Ok(_)) | (Ok(_), Err(error)) => Err(error),
                    (Err(mut left), Err(right)) => {
                        left.remaining.push(right.first);
                        left.remaining.extend(right.remaining);
                        Err(left)
                    }
                },
            ),
        }
    }
    /// Construct the application's typed record only after every field has parsed.
    pub fn map<P: 'static>(self, map: impl Fn(T) -> P + 'static) -> FormValue<W, K, P> {
        FormValue {
            read: Rc::new(move |window| (self.read)(window).map(&map)),
        }
    }
}
impl<W: ComponentHandle + 'static, K: Clone + PartialEq + 'static> FormBinding<W, K> {
    fn register_ports(
        &self,
        key: K,
        ports: FormFieldPorts<W>,
    ) -> Result<(), FormRegistrationError> {
        self.register_view(
            key,
            ports.snapshot,
            ports.report,
            ports.focus,
            ports.reset,
            ports.set_error,
        )
    }
    /// Declare one required native draft, its codec and control ports together.
    /// The shared HTML engine and FieldSpec parse at the Rust submission boundary.
    pub fn bind_required<T: 'static, E: 'static>(
        &self,
        key: K,
        spec: FieldSpec<T, E>,
        commit: fn(&W) -> ui::FieldCommit,
        ports: FormFieldPorts<W>,
        error_text: impl Fn(FieldSpecError<E>) -> String + 'static,
    ) -> Result<FormValue<W, K, T>, FormRegistrationError> {
        self.register_ports(key.clone(), ports)?;
        Ok(FormValue {
            read: Rc::new(move |window| {
                spec.accept(commit(window))
                    .and_then(|value| {
                        value.ok_or(FieldSpecError::Field(FieldFailure::MissingValue))
                    })
                    .map_err(|error| FormErrors::one(key.clone(), error_text(error)))
            }),
        })
    }
    /// Bind a value already represented by a strict UI type, such as a choice enum.
    /// Its registered constraint snapshot still participates in guarded submission.
    pub fn bind_value<T: 'static>(
        &self,
        key: K,
        ports: FormFieldPorts<W>,
        value: impl Fn(&W) -> T + 'static,
    ) -> Result<FormValue<W, K, T>, FormRegistrationError> {
        self.register_ports(key, ports)?;
        Ok(FormValue {
            read: Rc::new(move |window| Ok(value(window))),
        })
    }
    /// Bind a checkbox's strict state to its checked boolean at the kit boundary.
    pub fn bind_checked(
        &self,
        key: K,
        ports: FormFieldPorts<W>,
        state: fn(&W) -> ui::CheckState,
    ) -> Result<FormValue<W, K, bool>, FormRegistrationError> {
        self.bind_value(key, ports, move |window| match state(window) {
            ui::CheckState::Checked => true,
            ui::CheckState::Unchecked | ui::CheckState::Mixed => false,
        })
    }
    pub fn with_schema<P: 'static>(&self, schema: FormValue<W, K, P>) -> TypedFormBinding<W, K, P> {
        TypedFormBinding {
            binding: self.clone(),
            schema,
        }
    }
}
/// A form whose submission returns the record described by its one typed schema.
pub struct TypedFormBinding<W: ComponentHandle, K, P> {
    binding: FormBinding<W, K>,
    schema: FormValue<W, K, P>,
}
impl<W: ComponentHandle + 'static, K: Clone + PartialEq + 'static, P: 'static>
    TypedFormBinding<W, K, P>
{
    pub fn submit(&self) -> Result<FormSubmission<P>, FormSubmitError<K>> {
        self.binding
            .submit_typed(|window| (self.schema.read)(window))
    }
    pub fn finish(&self, ticket: SubmissionTicket, errors: Vec<FieldError<K, String>>) -> bool {
        self.binding.finish(ticket, errors)
    }
    pub fn binding(&self) -> &FormBinding<W, K> {
        &self.binding
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Key {
        First,
        Second,
    }
    #[test]
    fn each_value_is_read_once_before_the_record_is_constructed() {
        let reads = Rc::new(Cell::new(0));
        let left_reads = reads.clone();
        let left: FormValue<(), Key, i32> = FormValue {
            read: Rc::new(move |_| {
                left_reads.set(left_reads.get() + 1);
                Ok(3)
            }),
        };
        let right_reads = reads.clone();
        let right = FormValue {
            read: Rc::new(move |_| {
                right_reads.set(right_reads.get() + 1);
                Ok(4)
            }),
        };
        let records = Rc::new(Cell::new(0));
        let constructed = records.clone();
        let schema = left.zip(right).map(move |(a, b)| {
            constructed.set(constructed.get() + 1);
            a + b
        });
        assert_eq!((schema.read)(&()).unwrap(), 7);
        assert_eq!(reads.get(), 2);
        assert_eq!(records.get(), 1);
    }
    #[test]
    fn all_schema_errors_are_retained_before_a_record_can_be_constructed() {
        let reads = Rc::new(Cell::new(0));
        let first_reads = reads.clone();
        let first: FormValue<(), Key, i32> = FormValue {
            read: Rc::new(move |_| {
                first_reads.set(first_reads.get() + 1);
                Err(FormErrors::one(Key::First, "First rejected"))
            }),
        };
        let second_reads = reads.clone();
        let second = FormValue {
            read: Rc::new(move |_| {
                second_reads.set(second_reads.get() + 1);
                Err(FormErrors::one(Key::Second, "Second rejected"))
            }),
        };
        let schema = first
            .zip(second)
            .map(|_: (i32, i32)| -> i32 { panic!("invalid values cannot construct the record") });
        let error = (schema.read)(&()).unwrap_err();
        assert_eq!(reads.get(), 2);
        assert_eq!(error.first.field, Key::First);
        assert_eq!(error.remaining[0].field, Key::Second);
    }
}
