use crate::{Point, Color};

/// Describes a position on a colorful gradient.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GradientStop {
    pub position: f64,
    pub color: Color,
}

/// Describes a colorful linear gradient.
#[derive(Clone, PartialEq, Debug)]
pub struct LinearGradient {
    pub start: Point,
    pub end: Point,
    pub stops: Vec<GradientStop>,
}
