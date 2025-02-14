use std::ops::{Add, Sub, Mul, Div, BitAnd, BitOr, BitXor, Neg, Rem, Shl, Shr, };
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
enum Number {
    Int(i64),
    Float(f64),
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Number::Int(n)
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Number::Float(n)
    }
}

impl Number {
    fn as_f64(self) -> f64 {
        match self {
            Number::Int(n) => n as f64,
            Number::Float(n) => n,
        }
    }
}

macro_rules! impl_op {
    ($trait:ident, $method:ident) => {
        impl $trait for Number {
            type Output = Number;

            fn $method(self, other: Self) -> Self::Output {
                match (self, other) {
                    (Number::Int(a), Number::Int(b)) => Number::Int(a.$method(b)),
                    (a, b) => Number::Float(a.as_f64().$method(b.as_f64())),
                }
            }
        }
    };
}

impl_op!(Add, add);
impl_op!(Sub, sub);
impl_op!(Mul, mul);
impl_op!(Div, div);
impl_op!(Rem, rem);

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Number::Int(n) => Number::Int(-n),
            Number::Float(n) => Number::Float(-n),
        }
    }
}

macro_rules! impl_bitwise_op {
    ($trait:ident, $method:ident) => {
        impl $trait for Number {
            type Output = Number;
            
            fn $method(self, rhs: Self) -> Self::Output {
                let lhs = match self {
                    Number::Int(n) => n,
                    Number::Float(f) => f as i64, // Truncate float to integer
                };

                let rhs = match rhs {
                    Number::Int(n) => n,
                    Number::Float(f) => f as i64, // Truncate float to integer
                };

                Number::Int(lhs.$method(rhs)) // Perform bitwise operation
            }
        }
    };
}

impl_bitwise_op!(BitAnd, bitand);
impl_bitwise_op!(BitOr, bitor);
impl_bitwise_op!(BitXor, bitxor);
impl_bitwise_op!(Shr, shr);
impl_bitwise_op!(Shl, shl);

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.as_f64() == other.as_f64()
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_f64().partial_cmp(&other.as_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(Number::Int(10) + Number::Int(5), Number::Int(15));
        assert_eq!(Number::Int(10) + Number::Float(2.5), Number::Float(12.5));
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(Number::Int(10) - Number::Int(5), Number::Int(5));
        assert_eq!(Number::Int(10) - Number::Float(2.5), Number::Float(7.5));
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(Number::Int(10) * Number::Int(5), Number::Int(50));
        assert_eq!(Number::Int(10) * Number::Float(2.5), Number::Float(25.0));
    }

    #[test]
    fn test_division() {
        assert_eq!(Number::Int(10) / Number::Int(5), Number::Float(2.0)); // Always returns Float
        assert_eq!(Number::Int(10) / Number::Float(2.5), Number::Float(4.0));
    }

    #[test]
    fn test_remainder() {
        assert_eq!(Number::Int(10) % Number::Int(5), Number::Int(0));
        assert_eq!(Number::Float(5.0) % Number::Float(2.0), Number::Float(1.0));
    }

    #[test]
    fn test_nagation() {
        assert_eq!(Number::Float(4.0), -Number::Float(-4.0));
        assert_eq!(Number::Int(-2), -Number::Int(2));
    }

    #[test]
    fn test_comparisons() {
        assert!(Number::Int(10) > Number::Float(2.5));
        assert!(Number::Int(5) < Number::Int(10));
        assert!(Number::Float(3.5) <= Number::Float(3.5));
        assert!(Number::Int(10) >= Number::Float(10.0));
    }

    #[test]
    fn test_bitwise_operations() {
        assert_eq!(Number::Int(6) & Number::Int(3), Number::Int(2)); // 6 & 3 = 2
        assert_eq!(Number::Int(6) | Number::Int(3), Number::Int(7)); // 6 | 3 = 7
        assert_eq!(Number::Int(6) ^ Number::Int(3), Number::Int(5)); // 6 ^ 3 = 5
        assert_eq!(Number::Int(8) >> Number::Int(2), Number::Int(2)); // 8 >> 2 = 2
        assert_eq!(Number::Int(1) << Number::Int(3), Number::Int(8)); // 1 << 3 = 8
        assert_eq!(Number::Float(6.0) & Number::Float(3.2), Number::Int(2));
    }
}