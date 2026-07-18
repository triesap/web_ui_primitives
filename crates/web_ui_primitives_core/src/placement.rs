#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PlacementRect {
    pub fn right(self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(self) -> f64 {
        self.y + self.height
    }

    pub fn center_x(self) -> f64 {
        self.x + (self.width / 2.0)
    }

    pub fn center_y(self) -> f64 {
        self.y + (self.height / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl PlacementSide {
    pub fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementOptions {
    pub side: PlacementSide,
    pub align: PlacementAlign,
    pub spacing: f64,
    pub viewport_padding: f64,
    pub flip: bool,
    pub shift: bool,
}

impl PlacementOptions {
    pub fn new(side: PlacementSide, align: PlacementAlign) -> Self {
        Self {
            side,
            align,
            spacing: 0.0,
            viewport_padding: 0.0,
            flip: true,
            shift: true,
        }
    }

    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn viewport_padding(mut self, viewport_padding: f64) -> Self {
        self.viewport_padding = viewport_padding;
        self
    }

    pub fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    pub fn shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub max_width: f64,
    pub max_height: f64,
    pub side: PlacementSide,
    pub align: PlacementAlign,
}

pub fn place_layer(
    anchor: PlacementRect,
    layer: PlacementSize,
    viewport: PlacementSize,
    options: PlacementOptions,
) -> Placement {
    let padding = finite_positive(options.viewport_padding);
    let spacing = finite_positive(options.spacing);
    let side = resolve_side(anchor, layer, viewport, options, padding, spacing);
    let viewport_max_width = (viewport.width - (padding * 2.0)).max(0.0);
    let viewport_max_height = (viewport.height - (padding * 2.0)).max(0.0);
    let available_main = available_main_size(anchor, viewport, side, padding, spacing).max(0.0);
    let (max_width, max_height) = if side.is_vertical() {
        (viewport_max_width, available_main.min(viewport_max_height))
    } else {
        (available_main.min(viewport_max_width), viewport_max_height)
    };
    let (width, height) = if side.is_vertical() {
        (
            finite_positive(layer.width).min(max_width),
            finite_positive(layer.height).min(max_height),
        )
    } else {
        (
            finite_positive(layer.width).min(max_height),
            finite_positive(layer.height).min(viewport_max_height),
        )
    };
    let x = resolve_x(
        anchor,
        width,
        viewport.width,
        side,
        options.align,
        padding,
        spacing,
        options.shift,
    );
    let y = resolve_y(
        anchor,
        height,
        viewport.height,
        side,
        options.align,
        padding,
        spacing,
        options.shift,
    );

    Placement {
        x,
        y,
        width,
        height,
        max_width,
        max_height,
        side,
        align: options.align,
    }
}

fn resolve_side(
    anchor: PlacementRect,
    layer: PlacementSize,
    viewport: PlacementSize,
    options: PlacementOptions,
    padding: f64,
    spacing: f64,
) -> PlacementSide {
    if !options.flip {
        return options.side;
    }

    let preferred = available_main_size(anchor, viewport, options.side, padding, spacing);
    let opposite = available_main_size(anchor, viewport, options.side.opposite(), padding, spacing);
    let required = if options.side.is_vertical() {
        finite_positive(layer.height)
    } else {
        finite_positive(layer.width)
    };

    if required > preferred && opposite > preferred {
        options.side.opposite()
    } else {
        options.side
    }
}

fn available_main_size(
    anchor: PlacementRect,
    viewport: PlacementSize,
    side: PlacementSide,
    padding: f64,
    spacing: f64,
) -> f64 {
    match side {
        PlacementSide::Top => anchor.y - padding - spacing,
        PlacementSide::Right => viewport.width - padding - anchor.right() - spacing,
        PlacementSide::Bottom => viewport.height - padding - anchor.bottom() - spacing,
        PlacementSide::Left => anchor.x - padding - spacing,
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_x(
    anchor: PlacementRect,
    width: f64,
    viewport_width: f64,
    side: PlacementSide,
    align: PlacementAlign,
    padding: f64,
    spacing: f64,
    shift: bool,
) -> f64 {
    let x = match side {
        PlacementSide::Left => anchor.x - spacing - width,
        PlacementSide::Right => anchor.right() + spacing,
        PlacementSide::Top | PlacementSide::Bottom => match align {
            PlacementAlign::Start => anchor.x,
            PlacementAlign::Center => anchor.center_x() - (width / 2.0),
            PlacementAlign::End => anchor.right() - width,
        },
    };

    if shift {
        clamp_axis(x, width, viewport_width, padding)
    } else {
        x
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_y(
    anchor: PlacementRect,
    height: f64,
    viewport_height: f64,
    side: PlacementSide,
    align: PlacementAlign,
    padding: f64,
    spacing: f64,
    shift: bool,
) -> f64 {
    let y = match side {
        PlacementSide::Top => anchor.y - spacing - height,
        PlacementSide::Bottom => anchor.bottom() + spacing,
        PlacementSide::Left | PlacementSide::Right => match align {
            PlacementAlign::Start => anchor.y,
            PlacementAlign::Center => anchor.center_y() - (height / 2.0),
            PlacementAlign::End => anchor.bottom() - height,
        },
    };

    if shift {
        clamp_axis(y, height, viewport_height, padding)
    } else {
        y
    }
}

fn clamp_axis(position: f64, size: f64, viewport_size: f64, padding: f64) -> f64 {
    let min = padding;
    let max = (viewport_size - padding - size).max(min);
    position.max(min).min(max)
}

fn finite_positive(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlacementAlign, PlacementOptions, PlacementRect, PlacementSide, PlacementSize, place_layer,
    };

    #[test]
    fn placement_flips_bottom_to_top_when_bottom_space_is_smaller() {
        let placement = place_layer(
            PlacementRect {
                x: 40.0,
                y: 170.0,
                width: 40.0,
                height: 20.0,
            },
            PlacementSize {
                width: 120.0,
                height: 80.0,
            },
            PlacementSize {
                width: 240.0,
                height: 220.0,
            },
            PlacementOptions::new(PlacementSide::Bottom, PlacementAlign::Start)
                .spacing(4.0)
                .viewport_padding(8.0),
        );

        assert_eq!(placement.side, PlacementSide::Top);
        assert_eq!(placement.y, 86.0);
        assert_eq!(placement.max_height, 158.0);
    }

    #[test]
    fn placement_shifts_end_aligned_layer_inside_viewport() {
        let placement = place_layer(
            PlacementRect {
                x: 220.0,
                y: 24.0,
                width: 40.0,
                height: 20.0,
            },
            PlacementSize {
                width: 160.0,
                height: 80.0,
            },
            PlacementSize {
                width: 260.0,
                height: 220.0,
            },
            PlacementOptions::new(PlacementSide::Bottom, PlacementAlign::End)
                .spacing(4.0)
                .viewport_padding(12.0),
        );

        assert_eq!(placement.side, PlacementSide::Bottom);
        assert_eq!(placement.x, 88.0);
        assert_eq!(placement.max_width, 236.0);
    }

    #[test]
    fn placement_constrains_oversized_layer_to_padded_viewport() {
        let placement = place_layer(
            PlacementRect {
                x: 4.0,
                y: 4.0,
                width: 20.0,
                height: 20.0,
            },
            PlacementSize {
                width: 500.0,
                height: 500.0,
            },
            PlacementSize {
                width: 320.0,
                height: 240.0,
            },
            PlacementOptions::new(PlacementSide::Bottom, PlacementAlign::Start)
                .spacing(4.0)
                .viewport_padding(16.0)
                .flip(false),
        );

        assert_eq!(placement.x, 16.0);
        assert_eq!(placement.y, 28.0);
        assert_eq!(placement.width, 288.0);
        assert_eq!(placement.height, 196.0);
        assert_eq!(placement.max_height, 196.0);
    }
}
