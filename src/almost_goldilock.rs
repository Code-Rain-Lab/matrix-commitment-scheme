use ark_ff::{Field, Fp64, MontBackend};

// Almost Goldilock
#[derive(ark_ff::MontConfig)]
#[modulus = "18446744069414584289"] // = (2^64 − 2^32 + 1) − 32
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// x = c0 + c1 * u, where u^2 = 3 (Δ = 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fq2<F: Field> {
    pub c0: F,
    pub c1: F,
}

impl<F: Field> Fq2<F> {
    #[inline]
    pub fn zero() -> Self {
        Self {
            c0: F::zero(),
            c1: F::zero(),
        }
    }
    #[inline]
    pub fn one() -> Self {
        Self {
            c0: F::one(),
            c1: F::zero(),
        }
    }

    #[inline]
    pub fn new(c0: F, c1: F) -> Self {
        Self { c0, c1 }
    }

    #[inline]
    pub fn inv(self) -> Option<Self> {
        // (a + b u)^{-1} = (a - b u) / (a^2 - 3 b^2)
        let a2 = self.c0.square();
        let b2 = self.c1.square();
        let three_b2 = b2 + b2 + b2;
        let denom = a2 - three_b2;
        denom.inverse().map(|inv| Self {
            c0: self.c0 * inv,
            c1: -self.c1 * inv,
        })
    }
}

/* ---------- Add / Sub / Neg ---------- */

impl<F: Field> Add for Fq2<F> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            c0: self.c0 + rhs.c0,
            c1: self.c1 + rhs.c1,
        }
    }
}
impl<F: Field> AddAssign for Fq2<F> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.c0 += rhs.c0;
        self.c1 += rhs.c1;
    }
}

impl<F: Field> Sub for Fq2<F> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            c0: self.c0 - rhs.c0,
            c1: self.c1 - rhs.c1,
        }
    }
}
impl<F: Field> SubAssign for Fq2<F> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.c0 -= rhs.c0;
        self.c1 -= rhs.c1;
    }
}

impl<F: Field> Neg for Fq2<F> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            c0: -self.c0,
            c1: -self.c1,
        }
    }
}

/* ---------- Mul (Karatsuba 3 muls, Δ=3) ---------- */

impl<F: Field> Mul for Fq2<F> {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        // (a + b u)(c + d u) = (ac + 3 bd) + (ad + bc) u
        let (a, b) = (self.c0, self.c1);
        let (c, d) = (rhs.c0, rhs.c1);
        let t0 = a * c; // ac
        let t1 = b * d; // bd
        let t2 = (a + b) * (c + d); // (a+b)(c+d) = ac + ad + bc + bd
        let three_t1 = t1 + t1 + t1; // 3*bd
        let c0 = t0 + three_t1; // ac + 3bd
        let c1 = t2 - t0 - t1; // ad + bc
        Self { c0, c1 }
    }
}
impl<F: Field> MulAssign for Fq2<F> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        let (a, b) = (self.c0, self.c1);
        let (c, d) = (rhs.c0, rhs.c1);
        let t0 = a * c;
        let t1 = b * d;
        let t2 = (a + b) * (c + d);
        let three_t1 = t1 + t1 + t1;
        self.c0 = t0 + three_t1;
        self.c1 = t2 - t0 - t1;
    }
}

/* ---------- Scalar mul/div over base field F ---------- */

impl<F: Field> Mul<F> for Fq2<F> {
    type Output = Self;
    #[inline]
    fn mul(self, s: F) -> Self::Output {
        Self {
            c0: self.c0 * s,
            c1: self.c1 * s,
        }
    }
}
impl<F: Field> MulAssign<F> for Fq2<F> {
    #[inline]
    fn mul_assign(&mut self, s: F) {
        self.c0 *= s;
        self.c1 *= s;
    }
}
