use std::cmp;
use std::ops;

#[derive(Clone, Debug, Copy)]
pub struct FPNum {
    is_negative: bool,
    value: u64,
}

impl FPNum {
    fn new(numerator: i32, denominator: u32) -> Self {
        FPNum::from(numerator) / denominator
    }

    fn signum(&self) -> i8 {
        if self.is_negative {
            -1
        } else {
            1
        }
    }

    fn is_negative(&self) -> bool {
        self.is_negative
    }

    fn is_positive(&self) -> bool {
        !self.is_negative
    }

    fn is_zero(&self) -> bool {
        self.value == 0
    }

    fn abs(&self) -> Self {
        Self {
            is_negative: false,
            value: self.value,
        }
    }

    fn round(&self) -> i64 {
        if self.is_negative {
            -((self.value >> 32) as i64)
        } else {
            (self.value >> 32) as i64
        }
    }

    fn sqr(&self) -> Self {
        Self {
            is_negative: false,
            value: ((self.value as u128).pow(2) >> 32) as u64,
        }
    }

    fn sqrt(&self) -> Self {
        debug_assert!(!self.is_negative);

        let mut t: u64 = 0x4000000000000000;
        let mut r: u64 = 0;
        let mut q = self.value;

        for _ in 0..32 {
            let s = r + t;
            r >>= 1;

            if s <= q {
                q -= s;
                r += t;
            }
            t >>= 2;
        }

        Self {
            is_negative: false,
            value: r << 16,
        }
    }
}

impl From<i32> for FPNum {
    #[inline]
    fn from(n: i32) -> Self {
        FPNum {
            is_negative: n < 0,
            value: (n.abs() as u64) << 32,
        }
    }
}

impl From<u32> for FPNum {
    #[inline]
    fn from(n: u32) -> Self {
        Self {
            is_negative: false,
            value: (n as u64) << 32,
        }
    }
}

impl From<FPNum> for f64 {
    #[inline]
    fn from(n: FPNum) -> Self {
        if n.is_negative {
            n.value as f64 / (-0x10000000 as f64)
        } else {
            n.value as f64 / 0x10000000 as f64
        }
    }
}

impl PartialEq for FPNum {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && (self.is_negative == other.is_negative || self.value == 0)
    }
}

impl Eq for FPNum {}

impl PartialOrd for FPNum {
    fn partial_cmp(&self, rhs: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for FPNum {
    #[inline]
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        if self.value == 0 && rhs.value == 0 {
            cmp::Ordering::Equal
        } else if self.is_negative != rhs.is_negative {
            if self.is_negative {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            }
        } else if self.value == rhs.value {
            cmp::Ordering::Equal
        } else if self.is_negative {
            if self.value > rhs.value {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            }
        } else {
            if self.value < rhs.value {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            }
        }
    }
}

impl ops::Add for FPNum {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        if self.is_negative == rhs.is_negative {
            Self {
                is_negative: self.is_negative,
                value: self.value + rhs.value,
            }
        } else if self.value > rhs.value {
            Self {
                is_negative: self.is_negative,
                value: self.value - rhs.value,
            }
        } else {
            Self {
                is_negative: rhs.is_negative,
                value: rhs.value - self.value,
            }
        }
    }
}

impl ops::Sub for FPNum {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        if self.is_negative == rhs.is_negative {
            if self.value > rhs.value {
                Self {
                    is_negative: self.is_negative,
                    value: self.value - rhs.value,
                }
            } else {
                Self {
                    is_negative: !rhs.is_negative,
                    value: rhs.value - self.value,
                }
            }
        } else {
            Self {
                is_negative: self.is_negative,
                value: self.value + rhs.value,
            }
        }
    }
}

impl ops::Neg for FPNum {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            is_negative: !self.is_negative,
            value: self.value,
        }
    }
}

impl ops::Mul for FPNum {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            is_negative: self.is_negative ^ rhs.is_negative,
            value: ((self.value as u128 * rhs.value as u128) >> 32) as u64,
        }
    }
}

impl ops::Mul<i32> for FPNum {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: i32) -> Self {
        Self {
            is_negative: self.is_negative ^ (rhs < 0),
            value: self.value * rhs.abs() as u64,
        }
    }
}

impl ops::Div for FPNum {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        Self {
            is_negative: self.is_negative ^ rhs.is_negative,
            value: (((self.value as u128) << 32) / rhs.value as u128) as u64,
        }
    }
}

impl ops::Div<i32> for FPNum {
    type Output = Self;

    #[inline]
    fn div(self, rhs: i32) -> Self {
        Self {
            is_negative: self.is_negative ^ (rhs < 0),
            value: self.value / rhs.abs() as u64,
        }
    }
}

impl ops::Div<u32> for FPNum {
    type Output = Self;

    #[inline]
    fn div(self, rhs: u32) -> Self {
        Self {
            is_negative: self.is_negative,
            value: self.value / rhs as u64,
        }
    }
}

/* TODO:
 Distance
 DistanceI
 SignAs
 AngleSin
 AngleCos
*/

#[cfg(test)]
#[test]
fn basics() {
    let n = FPNum::new(15, 2);
    assert!(n.is_positive());
    assert!(!n.is_negative());

    assert!(!(-n).is_positive());
    assert!((-n).is_negative());

    assert_eq!(-(-n), n);
    assert_eq!((-n).abs(), n);
    assert_eq!(-n, FPNum::new(-15, 2));

    assert_eq!(n.round(), 7);
    assert_eq!((-n).round(), -7);
}

#[test]
fn zero() {
    let z = FPNum::from(0);
    let n = FPNum::new(15, 2);

    assert!(z.is_zero());
    assert!(z.is_positive());
    assert!((-z).is_negative);
    assert_eq!(n - n, z)
}

#[test]
fn arith() {
    let n1_5 = FPNum::new(3, 2);
    let n2_25 = FPNum::new(9, 4);

    assert_eq!(n1_5 + n1_5, FPNum::from(3));
    assert_eq!(-n1_5 - n1_5, FPNum::from(-3));

    assert_eq!(n1_5 * n1_5, n2_25);
    assert_eq!(-n1_5 * -n1_5, n2_25);
    assert_eq!(n1_5 * -n1_5, -n2_25);
    assert_eq!(-n1_5 * n1_5, -n2_25);

    assert_eq!(n1_5.sqr(), n2_25);
    assert_eq!((-n1_5).sqr(), n2_25);

    assert_eq!(n2_25.sqrt(), n1_5);

    assert_eq!((n1_5 * n1_5 * n1_5.sqr()).sqrt(), n2_25);
}
