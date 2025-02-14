use std::ops;

macro_rules! impl_operator {
    ($trait:ident, $method:ident, $op:tt) => {
        impl ops::$trait for Number {
            type Output = Self;
        
            fn $method(self, other: Self) -> Self {
                match (self, other) {
                    (Number::Float(a), Number::Float(b)) => Number::Float(a $op b),
                    (Number::Int(a), Number::Int(b)) => Number::Int(a $op b),
                    (Number::Float(a), Number::Int(b)) => Number::Float(a $op b as f64),
                    (Number::Int(a), Number::Float(b)) => Number::Float(a as f64 $op b),
                }
            }
        }
    };
}

#[derive(Clone)]
pub enum Number {
    Int(i64),
    Float(f64)
}

impl_operator!(Add, add, +);
impl_operator!(Sub, sub, -);
impl_operator!(Mul, mul, *);
impl_operator!(Div, div, /);
