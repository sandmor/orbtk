use crate::{Color, Direction, OnLinePos, OnPlanePos, Point};

/// Describes a position on a colorful gradient.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GradientStop {
    pub pos: Option<OnLinePos>,
    pub color: Color,
}

/// Describes the coordinates of a colorful linear gradient.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LinearGradientCoords {
    /// Defines the linear gradient by point A to point B.
    Ends { start: Point, end: Point },
    /// Defines the linear gradient using an angle from the center of the target figure.
    Angle {
        radians: f64,
        // Defines a displacement from the center of the target shape.
        displacement: OnPlanePos,
    },
    // Defines the gradient as one that crosses the figure in a given direction.
    Direction {
        direction: Direction,
        displacement: OnPlanePos,
    },
}

impl Default for LinearGradientCoords {
    fn default() -> LinearGradientCoords {
        LinearGradientCoords::Direction {
            direction: Direction::ToTop,
            displacement: OnPlanePos::default(),
        }
    }
}

/// Describes the size of a colorful radial gradient, Boolean allows you to choose between
/// forcing a circle or allowing an ellipse, if a boolean is true, a circle will be forced
/// regardless of the shape of the figure, otherwise an ellipse will be allowed.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RadialGradientSize {
    ToClosestSide(bool),
    ToClosestCorner(bool),
    ToFarthestSide(bool),
    ToFarthestCorner(bool),
    Custom(OnPlanePos),
    Radius(OnLinePos),
}

impl Default for RadialGradientSize {
    fn default() -> Self {
        Self::ToClosestSide(false)
    }
}

/// Describes a colorful radial gradient shape and position.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RadialGradient {
    pub size: RadialGradientSize,
    pub pos: Option<OnPlanePos>,
}

impl Default for RadialGradient {
    fn default() -> Self {
        Self {
            size: RadialGradientSize::default(),
            pos: None,
        }
    }
}

/// Describes a colorful gradient.
#[derive(Clone, PartialEq, Debug)]
pub struct Gradient {
    pub kind: GradientKind,
    pub stops: Vec<GradientStop>,
    pub repeat: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientKind {
    Linear(LinearGradientCoords),
    Radial(RadialGradient),
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            kind: GradientKind::Linear(LinearGradientCoords::default()),
            stops: vec![
                GradientStop {
                    pos: None,
                    color: Color::rgb(0, 0, 0),
                },
                GradientStop {
                    pos: None,
                    color: Color::rgb(255, 255, 255),
                },
            ],
            repeat: false,
        }
    }
}
