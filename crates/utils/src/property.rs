use crate::{Color, Number};
use std::iter::Peekable;
use std::str::Chars;

// Describes a RON declared function.
#[derive(Clone, PartialEq, Debug)]
pub enum Property {
    Color(Color),
    Method(String, Vec<Property>),
    Number(Number, String),
    Other(String),
}

impl Property {
    /// Try to convert `self` into a `Number`
    pub fn as_number(&self) -> Option<Number> {
        match self {
            Property::Number(number, d) if d.is_empty() => Some(*number),
            _ => None,
        }
    }
}

impl From<String> for Property {
    fn from(s: String) -> Self {
        Self::from(&s[..])
    }
}

impl From<&str> for Property {
    fn from(s: &str) -> Property {
        let mut s = s.chars().peekable();
        parse_property(&mut s)
    }
}

impl Into<Number> for Property {
    fn into(self) -> Number {
        match self {
            Property::Number(num, _) => num,
            _ => Number::default(),
        }
    }
}

fn parse_property(chrs: &mut Peekable<Chars>) -> Property {
    let mut text = String::new();
    let method;
    loop {
        match chrs.next() {
            Some('(') => {
                method = true;
                break;
            }
            Some(c) => {
                text.push(c);
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
                    args.push(parse_property(chrs));
                }
            }
        }
        Property::Method(text, args)
    } else {
        if text.starts_with('#') {
            return Property::Color(Color::from(text));
        } else if text.starts_with(|x: char| x.is_ascii_digit() || x == '.') {
            if let Some(mut ofs) = text.rfind(|x: char| x.is_ascii_digit() || x == '.') {
                ofs += 1; // Moves from before last digit to after last digit position
                if text[..ofs]
                    .find(|x| x == '.' || x == 'e' || x == 'E')
                    .is_some()
                {
                    if let Ok(v) = lexical_core::parse(text[..ofs].as_bytes()) {
                        return Property::Number(Number::Float(v), text[ofs..].to_owned());
                    }
                } else {
                    if let Ok(v) = lexical_core::parse(text[..ofs].as_bytes()) {
                        return Property::Number(Number::Real(v), text[ofs..].to_owned());
                    }
                }
            }
        }
        Property::Other(text)
    }
}
