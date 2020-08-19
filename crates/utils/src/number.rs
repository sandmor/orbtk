#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Number {
    Real(i64),
    Float(f64),
}

impl Default for Number {
    fn default() -> Self {
        Self::Real(0)
    }
}

macro_rules! impl_float {
    ($t:ty) => {
        impl From<$t> for Number {
            fn from(n: $t) -> Number {
                Number::Float(n as f64)
            }
        }
        
        impl Into<$t> for Number {
            fn into(self) -> $t {
                match self {
                    Number::Real(n) => n as $t,
                    Number::Float(n) => n as $t
                }
            }
        }
    };
}

macro_rules! impl_real {
    ($t:ty) => {
        impl From<$t> for Number {
            fn from(n: $t) -> Number {
                Number::Real(n as i64)
            }
        }

        impl Into<$t> for Number {
            fn into(self) -> $t {
                match self {
                    Number::Real(n) => n as $t,
                    Number::Float(n) => n as $t
                }
            }
        }
    };
}

impl_float!(f32);
impl_float!(f64);

impl_real!(u8);
impl_real!(i8);
impl_real!(u16);
impl_real!(i16);
impl_real!(u32);
impl_real!(i32);
impl_real!(u64);
impl_real!(i64);
