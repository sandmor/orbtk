use std::iter::Peekable;
use std::str::Chars;
// Describes a RON declared function.
#[derive(Clone, PartialEq, Debug)]
pub enum Property {
    Color(Color),
    Method(String, Vec<Property>),
    Other(String)
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

fn parse_property(chrs: &mut Peekable<Chars>) -> Property {
    let mut text = String::new();
    let method;
    loop {
        match chrs.next() {
            Some('(') => {
                method = true;
                break;
            },
            Some(c) => {
                text.push(c);
            },
            None => {
                method = false;
                break;
            },
        }
    }
    if method {
        let mut args = Vec::new();
        loop {
            match chrs.peek() {
                Some(',') => {
                    chrs.next().unwrap();
                },
                None | Some(')') => {
                    let _ = chrs.next();
                    break;
                },
                _ => {
                    args.push(parse_property(chrs));
                }
            }
        }
        Property::Method(text, args)
    }
    else {
        if text.starts_with('#') {
            Property::Color(Color::from(text))
        }
        else {
            Property::Other(text)
        }
    }
}