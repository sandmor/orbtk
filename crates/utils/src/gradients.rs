use crate::{Color, Direction, Point};

/// Describes a position on a colorful gradient.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GradientStop {
    pub kind: GradientStopKind,
    pub color: Color,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientStopKind {
    Interpolated,
    Fixed(f64),
    Pixels(f64),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientCoords {
    Ends { start: Point, end: Point },
    Angle { radians: f64 },
    Direction(Direction),
}

/// Describes a colorful linear gradient.
#[derive(Clone, PartialEq, Debug)]
pub struct Gradient {
    pub kind: GradientKind,
    pub coords: GradientCoords,
    pub stops: Vec<GradientStop>,
    pub repeat: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientKind {
    Linear,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            kind: GradientKind::Linear,
            coords: GradientCoords::Angle { radians: 0.0 },
            stops: vec![
                GradientStop {
                    kind: GradientStopKind::Interpolated,
                    color: Color::rgb(0, 0, 0),
                },
                GradientStop {
                    kind: GradientStopKind::Interpolated,
                    color: Color::rgb(255, 255, 255),
                },
            ],
            repeat: false,
        }
    }
}
