use crate::{Color, Number, Property};
use serde::de::DeserializeOwned;
/// Wraps a ron value and is used to support conversion to different types.
pub struct Value(pub ron::Value);

impl Value {
    /// Converts the internal value to the given type.
    pub fn get<T>(self) -> T
    where
        T: Default + DeserializeOwned,
    {
        if let Ok(value) = self.0.into_rust::<T>() {
            return value;
        }

        T::default()
    }

    pub fn color(&self) -> Option<Color> {
        let prop = match &self.0 {
            ron::Value::String(s) => Property::from(&s[..]),
            _ => return None,
        };
        match prop {
            Property::Color(color) => Some(color),
            Property::Method(name, args) => {
                for arg in args.iter() {
                    match arg {
                        Property::Number(_, t) if t.is_empty() => {}
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
}

impl From<ron::Value> for Value {
    fn from(v: ron::Value) -> Self {
        Value(v)
    }
}

impl Into<String> for Value {
    fn into(self) -> String {
        self.get::<String>()
    }
}

impl Into<f64> for Value {
    fn into(self) -> f64 {
        self.get::<f64>()
    }
}

impl Into<f32> for Value {
    fn into(self) -> f32 {
        self.get::<f32>()
    }
}
