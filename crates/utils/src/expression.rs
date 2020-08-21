use crate::prelude::*;
use std::f64;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, PartialEq, Debug)]
pub enum ExprOrOp {
    Expression(Expression),
    Operator(Operator),
}

impl ExprOrOp {
    pub fn expression(&self) -> Option<&Expression> {
        match self {
            Self::Expression(e) => Some(e),
            _ => None,
        }
    }
}

// Describes a RON declared function.
#[derive(Clone, PartialEq, Debug)]
pub enum Expression {
    Method(String, Vec<Expression>),
    Complex(Vec<ExprOrOp>),
    Number(Number, String),
    Color(Color),
    Other(String),
}

impl Expression {
    /// Try to convert `self` into a `Number`
    pub fn number(&self) -> Option<Number> {
        match self {
            Expression::Number(number, d) if d.is_empty() => Some(*number),
            _ => None,
        }
    }

    pub fn color(&self) -> Option<Color> {
        match self {
            Expression::Color(color) => Some(*color),
            Expression::Method(name, args) => {
                for arg in args.iter() {
                    match arg {
                        Expression::Number(_, t) if t.is_empty() => {}
                        _ => {
                            return None;
                        }
                    };
                }
                match &name[..] {
                    "rgb" if args.len() == 3 => Some(Color::rgb(
                        args[0].number().unwrap().into(),
                        args[1].number().unwrap().into(),
                        args[2].number().unwrap().into(),
                    )),
                    "hsv" if args.len() == 3 => Some(Color::hsv(
                        args[0].number().unwrap().into(),
                        args[1].number().unwrap().into(),
                        args[2].number().unwrap().into(),
                    )),
                    "hsl" if args.len() == 3 => Some(Color::hsl(
                        args[0].number().unwrap().into(),
                        args[1].number().unwrap().into(),
                        args[2].number().unwrap().into(),
                    )),
                    "rgba" if args.len() == 4 => Some(Color::rgba(
                        args[0].number().unwrap().into(),
                        args[1].number().unwrap().into(),
                        args[2].number().unwrap().into(),
                        args[3].number().unwrap().into(),
                    )),
                    "hsva" if args.len() == 4 => Some(Color::hsva(
                        args[0].number().unwrap().into(),
                        args[1].number().unwrap().into(),
                        args[2].number().unwrap().into(),
                        args[3].number().unwrap().into(),
                    )),
                    "hsla" if args.len() == 4 => Some(Color::hsla(
                        args[0].number().unwrap().into(),
                        args[1].number().unwrap().into(),
                        args[2].number().unwrap().into(),
                        args[3].number().unwrap().into(),
                    )),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn angle(&self) -> Option<f64> {
        match self {
            Expression::Number(num, unit) => {
                let num: f64 = (*num).into();
                let mut angle = match &unit[..] {
                    "rad" => num,
                    "turn" => f64::consts::PI * 2.0 * num,
                    _ => {
                        // Fallback to degrees
                        num * f64::consts::PI / 180.0
                    }
                };
                angle = (angle % (f64::consts::PI * 2.0)).abs();
                Some(angle)
            }
            /*            Expression::Other(label) => match &label[..] {
                "to top" => Some(f64::consts::PI * 2.0 * 0.0),
                "to top right" => Some(f64::consts::PI * 2.0 * 0.125),
                "to right" => Some(f64::consts::PI * 2.0 * 0.25),
                "to bottom right" => Some(f64::consts::PI * 2.0 * 0.375),
                "to bottom" => Some(f64::consts::PI * 2.0 * 0.5),
                "to bottom left" => Some(f64::consts::PI * 2.0 * 0.625),
                "to left" => Some(f64::consts::PI * 2.0 * 0.75),
                "to top left" => Some(f64::consts::PI * 2.0 * 0.875),
                _ => None,
            },*/
            _ => None,
        }
    }

    pub fn direction(&self) -> Option<Direction> {
        match self {
            Expression::Other(label) => match &label[..] {
                "to top" => Some(Direction::ToTop),
                "to top right" => Some(Direction::ToTopRight),
                "to right" => Some(Direction::ToRight),
                "to bottom right" => Some(Direction::ToBottomRight),
                "to bottom" => Some(Direction::ToBottom),
                "to bottom left" => Some(Direction::ToBottomLeft),
                "to left" => Some(Direction::ToLeft),
                "to top left" => Some(Direction::ToTopLeft),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn gradient_stop(&self) -> Option<GradientStop> {
        if let Some(color) = self.color() {
            return Some(GradientStop {
                kind: GradientStopKind::Interpolated,
                color,
            });
        }
        match self {
            Expression::Complex(v) if v.len() == 2 => {
                let color = match v[0].expression().and_then(|e| e.color()) {
                    Some(color) => color,
                    None => return None,
                };
                let (number, m) = match v[1].expression() {
                    Some(Expression::Number(n, m)) => (n, m),
                    _ => return None,
                };
                let kind = match &m[..] {
                    "%" => {
                        let mut o: f64 = (*number).into();
                        o /= 100.0;
                        GradientStopKind::Fixed(o)
                    }
                    _ => return None,
                };
                Some(GradientStop { kind, color })
            }
            _ => None,
        }
    }

    pub fn brush(&self) -> Option<Brush> {
        if let Some(color) = self.color() {
            return Some(Brush::from(color));
        }
        let (name, args) = match self {
            Expression::Method(name, args) => (name, args),
            _ => return None,
        };
        match &name[..] {
            "linear-gradient" if !args.is_empty() => {
                let mut i = 0;
                let mut coords = GradientCoords::Angle { radians: 0.0 };
                if let Some(direction) = args[0].direction() {
                    coords = GradientCoords::Direction(direction);
                    i += 1;
                }
                else if let Some(radians) = args[0].angle() {
                    coords = GradientCoords::Angle { radians };
                    i += 1;
                }
                let mut stops = Vec::new();
                for i in i..args.len() {
                    let stop = match args[i].gradient_stop() {
                        Some(stop) => stop,
                        None => continue,
                    };
                    stops.push(stop);
                }
                if stops.is_empty() {
                    return None;
                } else if stops.len() == 1 {
                    return Some(Brush::SolidColor(stops[0].color));
                }
                Some(Brush::Gradient(
                    GradientKind::Linear,
                    Gradient {
                        coords,
                        stops,
                    },
                ))
            }
            _ => None,
        }
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Complex(Vec::new())
    }
}

impl From<String> for Expression {
    fn from(s: String) -> Self {
        Self::from(&s[..])
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Expression {
        let mut s = s.chars().peekable();
        parse_expression_with_complex(&mut s).unwrap_or_default()
    }
}

impl Into<Number> for Expression {
    fn into(self) -> Number {
        match self {
            Expression::Number(num, _) => num,
            _ => Number::default(),
        }
    }
}

fn parse_expression_with_complex(chrs: &mut Peekable<Chars>) -> Option<Expression> {
    let mut v = Vec::new();
    loop {
        if let Some(c) = chrs.peek() {
            let c = *c;
            if c == ',' || c == ')' {
                break;
            } else if c.is_whitespace() {
                // Ignore whitespaces
                chrs.next().unwrap();
                continue;
            } else if c == '+' {
                v.push(ExprOrOp::Operator(Operator::Add));
                chrs.next().unwrap();
                continue;
            } else if c == '-' {
                v.push(ExprOrOp::Operator(Operator::Sub));
                chrs.next().unwrap();
                continue;
            } else if c == '*' {
                v.push(ExprOrOp::Operator(Operator::Mul));
                chrs.next().unwrap();
                continue;
            } else if c == '/' {
                v.push(ExprOrOp::Operator(Operator::Div));
                chrs.next().unwrap();
                continue;
            }
        } else {
            break;
        }
        let expr = parse_expression(chrs)?;
        v.push(ExprOrOp::Expression(expr));
    }
    if v.is_empty() {
        None
    } else if v.len() == 1 {
        Some(match v[0] {
            ExprOrOp::Expression(ref e) => e.to_owned(),
            ExprOrOp::Operator(_) => Expression::Complex(v),
        })
    } else {
        Some(Expression::Complex(v))
    }
}

fn parse_expression(chrs: &mut Peekable<Chars>) -> Option<Expression> {
    let mut text = String::new();
    let method;
    loop {
        match chrs.peek() {
            Some('(') => {
                chrs.next().unwrap();
                method = true;
                break;
            }
            Some(c)
                if *c == ','
                    || *c == ')'
                    || (c.is_whitespace()
                        && text.starts_with(|x: char| {
                            x == '#' || x.is_ascii_digit() || x == '.' || x == '-'
                        })) =>
            {
                method = false;
                break;
            }
            Some(c) => {
                text.push(*c);
                chrs.next().unwrap();
            }
            None => {
                method = false;
                break;
            }
        }
    }
    debug_assert!(!text.is_empty());
    if method {
        let mut args = Vec::new();
        loop {
            match chrs.peek() {
                Some(c) if c.is_whitespace() || *c == ',' => {
                    chrs.next().unwrap();
                }
                None | Some(')') => {
                    let _ = chrs.next();
                    break;
                }
                _ => {
                    args.push(parse_expression_with_complex(chrs)?);
                }
            }
        }
        Some(Expression::Method(text, args))
    } else {
        if text.starts_with('#') {
            return Some(Expression::Color(Color::from(text)));
        } else if text.starts_with(|x: char| x.is_ascii_digit() || x == '.' || x == '-') {
            if let Some(mut ofs) = text.rfind(|x: char| x.is_ascii_digit() || x == '.' || x == '-')
            {
                ofs += 1; // Moves from before last position digit to after last digit position
                if text[..ofs]
                    .find(|x| x == '.' || x == 'e' || x == 'E')
                    .is_some()
                {
                    if let Ok(v) = lexical_core::parse(text[..ofs].as_bytes()) {
                        return Some(Expression::Number(Number::Float(v), text[ofs..].to_owned()));
                    }
                } else {
                    if let Ok(v) = lexical_core::parse(text[..ofs].as_bytes()) {
                        return Some(Expression::Number(Number::Real(v), text[ofs..].to_owned()));
                    }
                }
            }
        }
        Some(Expression::Other(text))
    }
}
