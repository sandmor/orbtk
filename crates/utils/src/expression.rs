use crate::{Color, Number, Brush, Operator, LinearGradient};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, PartialEq, Debug)]
pub enum ExprOrOp {
    Expression(Expression),
    Operator(Operator),
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
    pub fn as_number(&self) -> Option<Number> {
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
                        args[0].as_number().unwrap().into(),
                        args[1].as_number().unwrap().into(),
                        args[2].as_number().unwrap().into(),
                    )),
                    "rgba" if args.len() == 4 => Some(Color::rgba(
                        args[0].as_number().unwrap().into(),
                        args[1].as_number().unwrap().into(),
                        args[2].as_number().unwrap().into(),
                        args[3].as_number().unwrap().into(),
                    )),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn linear_gradient(&self) -> Option<LinearGradient> {
        None
    }

    pub fn brush(&self) -> Option<Brush> {
        if let Some(color) = self.color() {
            return Some(Brush::from(color));
        }
        // TODO
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
        let mut v = Vec::new();
        loop {
            if let Some(c) = s.peek() {
                let c = *c;
                if c.is_whitespace() { // Ignore whitespaces
                    s.next().unwrap();
                    continue;
                }
                else if c == '+' {
                    v.push(ExprOrOp::Operator(Operator::Add));
                    s.next().unwrap();
                    continue;
                }
                else if c == '-' {
                    v.push(ExprOrOp::Operator(Operator::Sub));
                    s.next().unwrap();
                    continue;
                }
                else if c == '*' {
                    v.push(ExprOrOp::Operator(Operator::Mul));
                    s.next().unwrap();
                    continue;
                }
                else if c == '/' {
                    v.push(ExprOrOp::Operator(Operator::Div));
                    s.next().unwrap();
                    continue;
                }
            }
            else {
                break;
            }
            v.push(ExprOrOp::Expression(parse_expression(&mut s)));
        }
        if v.is_empty() {
            Self::default()
        }
        else if v.len() == 1 {
            match v[0] {
                ExprOrOp::Expression(ref e) => e.to_owned(),
                ExprOrOp::Operator(_) => Expression::Complex(v)
            }
        }
        else {
            Expression::Complex(v)
        }
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

fn parse_expression(chrs: &mut Peekable<Chars>) -> Expression {
    let mut text = String::new();
    let method;
    loop {
        match chrs.peek() {
            Some('(') => {
                chrs.next().unwrap();
                method = true;
                break;
            }
            Some(c) if *c == ',' || c.is_whitespace() => {
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
    if method {
        let mut args = Vec::new();
        loop {
            match chrs.peek() {
                Some(',') => {
                    chrs.next().unwrap();
                }
                None | Some(')') => {
                    let _ = chrs.next();
                    break;
                }
                _ => {
                    args.push(parse_expression(chrs));
                }
            }
        }
        Expression::Method(text, args)
    } else {
        if text.starts_with('#') {
            return Expression::Color(Color::from(text));
        } else if text.starts_with(|x: char| x.is_ascii_digit() || x == '.') {
            if let Some(mut ofs) = text.rfind(|x: char| x.is_ascii_digit() || x == '.') {
                ofs += 1; // Moves from before last digit to after last digit position
                if text[..ofs]
                    .find(|x| x == '.' || x == 'e' || x == 'E')
                    .is_some()
                {
                    if let Ok(v) = lexical_core::parse(text[..ofs].as_bytes()) {
                        return Expression::Number(Number::Float(v), text[ofs..].to_owned());
                    }
                } else {
                    if let Ok(v) = lexical_core::parse(text[..ofs].as_bytes()) {
                        return Expression::Number(Number::Real(v), text[ofs..].to_owned());
                    }
                }
            }
        }
        Expression::Other(text)
    }
}
