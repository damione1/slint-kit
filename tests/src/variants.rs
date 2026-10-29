// SPDX-License-Identifier: GPL-3.0-or-later
//! Every variant of the presentation enums that theme recipes map. Slint has
//! no `match`, so a recipe maps an enum with a ternary chain whose last branch
//! takes any variant not named before it. Each list below is checked by an
//! exhaustive match without a wildcard: a variant added to the Slint enum
//! fails to compile here, which points at the recipe tests to extend.
use super::*;

macro_rules! every_variant {
    ($name:ident: $ty:ident { $($variant:ident),+ $(,)? }) => {
        pub(crate) fn $name() -> Vec<$ty> {
            let listed = vec![$($ty::$variant),+];
            for value in &listed {
                match value {
                    $($ty::$variant => {})+
                }
            }
            listed
        }
    };
}

every_variant!(alert_severities: AlertSeverity { Information, Success, Warning, Error });
every_variant!(button_intents: ButtonIntent { Primary, Secondary, Outline, Quiet, Destructive });
every_variant!(control_sizes: ControlSize { Compact, Medium, Comfortable });
every_variant!(surface_levels: SurfaceLevel { Flat, Raised, Overlay, Popover });
every_variant!(typography_roles: TypographyRole { Caption, Control, Body, Subtitle, Title });
every_variant!(elevation_levels: ElevationLevel { None, Xs, Sm, Md, Lg, Xl });
every_variant!(field_accesses: FieldAccess { Editable, ReadOnly, Disabled });
