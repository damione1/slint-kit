// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ui;
use slint::{ModelRc, VecModel};

const COMMANDS: [ui::TextEditCommand; 6] = [
    ui::TextEditCommand::Undo,
    ui::TextEditCommand::Redo,
    ui::TextEditCommand::Cut,
    ui::TextEditCommand::Copy,
    ui::TextEditCommand::Paste,
    ui::TextEditCommand::SelectAll,
];
fn token(command: ui::TextEditCommand) -> i32 {
    match command {
        ui::TextEditCommand::Undo => 0,
        ui::TextEditCommand::Redo => 1,
        ui::TextEditCommand::Cut => 2,
        ui::TextEditCommand::Copy => 3,
        ui::TextEditCommand::Paste => 4,
        ui::TextEditCommand::SelectAll => 5,
    }
}
fn options(labels: ui::TextEditLabels, access: ui::TextEditAvailability) -> ModelRc<ui::OptionRow> {
    ModelRc::new(VecModel::from(
        COMMANDS
            .map(|command| {
                let (label, enabled) = match command {
                    ui::TextEditCommand::Undo => (labels.undo.clone(), access.editable),
                    ui::TextEditCommand::Redo => (labels.redo.clone(), access.editable),
                    ui::TextEditCommand::Cut => (
                        labels.cut.clone(),
                        access.editable && access.selection && !access.hidden_password,
                    ),
                    ui::TextEditCommand::Copy => (
                        labels.copy.clone(),
                        access.selection && !access.hidden_password,
                    ),
                    ui::TextEditCommand::Paste => (labels.paste.clone(), access.editable),
                    ui::TextEditCommand::SelectAll => (labels.select_all.clone(), access.has_text),
                };
                ui::OptionRow {
                    token: token(command),
                    generation: 0,
                    label,
                    disabled: !enabled,
                    ..Default::default()
                }
            })
            .to_vec(),
    ))
}
fn command(row: ui::OptionRow) -> ui::TextEditCommandResult {
    let command = COMMANDS
        .into_iter()
        .find(|command| token(*command) == row.token);
    match command {
        Some(command) if row.generation == 0 && !row.disabled => ui::TextEditCommandResult {
            valid: true,
            command,
        },
        Some(_) | None => ui::TextEditCommandResult::default(),
    }
}
fn placement(request: ui::TextMenuPlacement, window: slint::LogicalSize) -> ui::TextMenuGeometry {
    let values = [
        request.origin_x,
        request.origin_y,
        request.anchor_x,
        request.anchor_y,
        request.width,
        request.height,
        window.width,
        window.height,
    ];
    if values.iter().any(|value| !value.is_finite())
        || request.width <= 0.0
        || request.height <= 0.0
        || window.width <= 0.0
        || window.height <= 0.0
    {
        return ui::TextMenuGeometry::default();
    }
    let width = request.width.min(window.width);
    let height = request.height.min(window.height);
    ui::TextMenuGeometry {
        valid: true,
        x: (request.origin_x + request.anchor_x).clamp(0.0, window.width - width)
            - request.origin_x,
        y: (request.origin_y + request.anchor_y).clamp(0.0, window.height - height)
            - request.origin_y,
        width,
        height,
    }
}
pub(crate) fn install<W>(window: &W)
where
    W: slint::ComponentHandle + 'static,
    for<'a> ui::KitFieldService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitFieldService>();
    service.on_text_menu_options(options);
    service.on_text_menu_command(command);
    let weak = window.as_weak();
    service.on_text_menu_placement(move |request| {
        weak.upgrade()
            .map_or_else(ui::TextMenuGeometry::default, |window| {
                placement(
                    request,
                    window
                        .window()
                        .size()
                        .to_logical(window.window().scale_factor()),
                )
            })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;
    #[test]
    fn menu_rows_keep_typed_commands_and_access_guards() {
        let rows = options(
            ui::TextEditLabels::default(),
            ui::TextEditAvailability {
                editable: true,
                selection: true,
                hidden_password: true,
                has_text: true,
            },
        );
        for row in rows.iter() {
            let result = command(row.clone());
            assert_eq!(result.valid, !row.disabled);
            assert_eq!(
                row.disabled,
                row.token == token(ui::TextEditCommand::Cut)
                    || row.token == token(ui::TextEditCommand::Copy)
            );
        }
        assert!(
            !command(ui::OptionRow {
                token: 100,
                ..Default::default()
            })
            .valid
        );
        assert!(
            !command(ui::OptionRow {
                generation: 1,
                ..Default::default()
            })
            .valid
        );
    }
    #[test]
    fn menu_placement_clamps_window_edges_and_rejects_invalid_geometry() {
        let request = ui::TextMenuPlacement {
            origin_x: 100.0,
            origin_y: 80.0,
            anchor_x: 490.0,
            anchor_y: 300.0,
            width: 200.0,
            height: 240.0,
        };
        let result = placement(request.clone(), slint::LogicalSize::new(600.0, 400.0));
        assert!(result.valid);
        assert_eq!(
            (result.x, result.y, result.width, result.height),
            (300.0, 80.0, 200.0, 240.0)
        );
        let result = placement(request.clone(), slint::LogicalSize::new(80.0, 60.0));
        assert_eq!(
            (result.x, result.y, result.width, result.height),
            (-100.0, -80.0, 80.0, 60.0)
        );
        assert!(
            !placement(
                ui::TextMenuPlacement {
                    anchor_x: f32::NAN,
                    ..request
                },
                slint::LogicalSize::new(600.0, 400.0)
            )
            .valid
        );
    }
}
