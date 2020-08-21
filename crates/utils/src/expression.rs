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
                let mut values = [0.0f64; 4];
                for (i, arg) in args.iter().enumerate() {
                    if i > 3 {
                        return None;
                    }
                    let (mut v, p): (f64, bool) = match arg {
                        Expression::Number(v, u) if u.is_empty() => ((*v).into(), false),
                        Expression::Number(v, u) if u == "%" => ((*v).into(), true),
                        _ => {
                            return None;
                        }
                    };
                    if name == "rgb" || name == "rgba" {
                        if p {
                            v = v * 100.0 / 255.0;
                        } else if v.floor() == 0.0 {
                            v = 255.0 * v.fract();
                        }
                    } else if i != 0 && v > 1.0 {
                        v /= 100.0;
                    }
                    values[i] = v;
                }
                if args.len() == 3 {
                    Some(match &name[..] {
                        "rgb" => Color::rgb(values[0] as u8, values[1] as u8, values[2] as u8),
                        "hsv" | "hsb" => Color::hsv(values[0], values[1], values[2]),
                        "hsl" => Color::hsl(values[0], values[1], values[2]),
                        _ => return None,
                    })
                } else {
                    Some(match &name[..] {
                        "rgba" => Color::rgba(
                            values[0] as u8,
                            values[1] as u8,
                            values[2] as u8,
                            values[3] as u8,
                        ),
                        "hsva" | "hsba" => Color::hsva(values[0], values[1], values[2], values[3]),
                        "hsla" => Color::hsla(values[0], values[1], values[2], values[3]),
                        _ => return None,
                    })
                }
            }
            Expression::Other(s) => color_from_name(s),
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
                if angle.is_sign_negative() {
                    angle = (f64::consts::PI * 2.0) - -angle;
                } else {
                    angle = angle % (f64::consts::PI * 2.0);
                }
                Some(angle)
            }
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
        dbg!(&self);
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
                    "px" => {
                        let o: f64 = (*number).into();
                        GradientStopKind::Pixels(o)
                    }
                    _ => return None,
                };
                Some(GradientStop { kind, color })
            }
            _ => None,
        }
    }

    pub fn css_gradient(&self) -> Option<Gradient> {
        let (name, args) = match self {
            Expression::Method(name, args) => (name, args),
            _ => return None,
        };
        if args.is_empty() {
            return None;
        }
        let (kind, repeat) = match &name[..] {
            "repeating-linear-gradient" => (GradientKind::Linear, true),
            "linear-gradient" => (GradientKind::Linear, false),
            _ => {
                return None;
            }
        };
        let mut i = 0;
        let mut coords = GradientCoords::Angle { radians: 0.0 };
        if let Some(direction) = args[0].direction() {
            coords = GradientCoords::Direction(direction);
            i += 1;
        } else if let Some(radians) = args[0].angle() {
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
        }
        Some(Gradient {
            kind,
            coords,
            stops,
            repeat,
        })
    }

    pub fn brush(&self) -> Option<Brush> {
        if let Some(color) = self.color() {
            return Some(Brush::from(color));
        }
        if let Some(g) = self.css_gradient() {
            return Some(Brush::from(g));
        }
        None
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
        let mut expr = parse_expression(chrs)?;
        let mut reinterpret_op = None;
        if let Some(ExprOrOp::Operator(op)) = v.last().cloned() {
            if op == Operator::Add || op == Operator::Sub {
                reinterpret_op = Some(op == Operator::Add);
                if v.len() >= 2 {
                    match v[v.len() - 2] {
                        ExprOrOp::Expression(Expression::Number(_, _)) => {
                            // Mathematic expression
                            reinterpret_op = None;
                        }
                        _ => {}
                    };
                }
            }
        }
        if let Some(plus) = reinterpret_op {
            match expr {
                Expression::Number(ref mut n, _) => {
                    v.pop();
                    if !plus {
                        *n = -(*n);
                    }
                }
                _ => {}
            }
        }
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
            Some(c) if *c == ',' || *c == ')' || (c.is_whitespace() && text != "to") => {
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
