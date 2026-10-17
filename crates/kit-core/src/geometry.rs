#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupSide {
    Above,
    Below,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupPlacement {
    pub bounds: Rect,
    pub side: PopupSide,
}

fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}
fn length(value: f32) -> f32 {
    finite(value).max(0.0)
}
impl Rect {
    pub fn right(self) -> f32 {
        self.x + self.width
    }
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
    pub fn contains(self, other: Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }
}
/// All geometry uses one host-provided logical coordinate space. Supply a
/// viewport already reduced by a docked keyboard, then safe-area insets here.
/// Prefers below, flips when above has more room, reduces height and shifts
/// horizontally. Offscreen anchors cannot cause an out-of-viewport popup.
pub fn place_popup(
    anchor: Rect,
    desired: Size,
    viewport: Rect,
    insets: Insets,
    gap: f32,
) -> PopupPlacement {
    let vx = finite(viewport.x);
    let vy = finite(viewport.y);
    let vw = length(viewport.width);
    let vh = length(viewport.height);
    let left = length(insets.left).min(vw);
    let right = length(insets.right).min(vw - left);
    let top = length(insets.top).min(vh);
    let bottom = length(insets.bottom).min(vh - top);
    let area = Rect {
        x: vx + left,
        y: vy + top,
        width: vw - left - right,
        height: vh - top - bottom,
    };
    let gap = length(gap);
    let anchor_top = finite(anchor.y);
    let anchor_bottom = anchor_top + length(anchor.height);
    let below_y = (anchor_bottom + gap).clamp(area.y, area.bottom());
    let above_end = (anchor_top - gap).clamp(area.y, area.bottom());
    let below = (area.bottom() - below_y).max(0.0);
    let above = (above_end - area.y).max(0.0);
    let desired_height = length(desired.height);
    let side = if below >= desired_height || below >= above {
        PopupSide::Below
    } else {
        PopupSide::Above
    };
    let width = length(desired.width).min(area.width);
    let (height, y) = match side {
        PopupSide::Below => {
            let h = desired_height.min(below);
            (h, below_y)
        }
        PopupSide::Above => {
            let h = desired_height.min(above);
            (h, above_end - h)
        }
    };
    let x = finite(anchor.x).clamp(area.x, (area.right() - width).max(area.x));
    PopupPlacement {
        bounds: Rect {
            x,
            y,
            width,
            height,
        },
        side,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edge_flips_and_keyboard_reduction_clamp_geometry() {
        let view = Rect {
            x: 0.,
            y: 0.,
            width: 800.,
            height: 400.,
        };
        let anchor = Rect {
            x: 760.,
            y: 360.,
            width: 80.,
            height: 30.,
        };
        let p = place_popup(
            anchor,
            Size {
                width: 300.,
                height: 250.,
            },
            view,
            Insets {
                top: 20.,
                right: 10.,
                bottom: 10.,
                left: 10.,
            },
            4.,
        );
        assert_eq!(p.side, PopupSide::Above);
        assert_eq!(p.bounds.x, 490.);
        assert_eq!(p.bounds.height, 250.);
        assert!(view.contains(p.bounds));
    }
    #[test]
    fn geometric_invariant_over_extreme_anchors_sizes_and_insets() {
        let view = Rect {
            x: 20.,
            y: 30.,
            width: 640.,
            height: 480.,
        };
        for x in [-1000., 0., 40., 620., 2000.] {
            for y in [-1000., 0., 40., 470., 2000.] {
                for size in [0., 50., 1000.] {
                    for inset in [0., 40., 1000.] {
                        let p = place_popup(
                            Rect {
                                x,
                                y,
                                width: 40.,
                                height: 40.,
                            },
                            Size {
                                width: size,
                                height: size,
                            },
                            view,
                            Insets {
                                top: inset,
                                right: inset,
                                bottom: inset,
                                left: inset,
                            },
                            6.,
                        );
                        assert!(view.contains(p.bounds), "{p:?}");
                        assert!(p.bounds.width >= 0. && p.bounds.height >= 0.);
                    }
                }
            }
        }
    }
    #[test]
    fn degenerate_input_remains_finite_and_nonnegative() {
        let p = place_popup(
            Rect {
                x: f32::NAN,
                y: f32::INFINITY,
                width: -1.,
                height: 30.,
            },
            Size {
                width: f32::INFINITY,
                height: -10.,
            },
            Rect::default(),
            Insets::default(),
            f32::NAN,
        );
        assert_eq!(p.bounds, Rect::default());
    }
}
