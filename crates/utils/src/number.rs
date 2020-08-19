#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Number {
    Real(i64),
    Float(f64)
}