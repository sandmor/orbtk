use crate::{Point, Color};

/// Describes a position on a colorful gradient.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GradientStop {
    pub position: f64,
    pub color: Color,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientCoords {
    Ends {
        start: Point,
        end: Point
    },
    Angle {
        radians: f64
    }
}

/// Describes a colorful linear gradient.
#[derive(Clone, PartialEq, Debug)]
pub struct Gradient {
    pub coords: GradientCoords,
    pub stops: Vec<GradientStop>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientKind {
    Linear
}