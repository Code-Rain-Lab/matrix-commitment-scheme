use ark_ff::PrimeField;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use num_traits::{Inv, One, Zero};

/// x = c0 + c1 * u, where u^2 = 3 (Δ = 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Fq2<F> {
    pub c0: F,
    pub c1: F,
}

impl<F> Fq2<F> {
    #[inline]
    pub fn new(c0: F, c1: F) -> Self {
        Self { c0, c1 }
    }
}

/* ---------- From ---------- */

impl<F: PrimeField> From<F> for Fq2<F>
where
    F: Zero,
{
    #[inline]
    fn from(value: F) -> Self {
        Self::new(value, F::zero())
    }
}

impl<F> From<&F> for Fq2<F>
where
    F: Zero + Clone,
{
    #[inline]
    fn from(value: &F) -> Self {
        Self::new(value.clone(), F::zero())
    }
}

/* ---------- basic constants (0,1,2) ---------- */

impl<F> Fq2<F>
where
    F: Copy + Zero,
{
    #[inline]
    pub fn zero() -> Self {
        Self {
            c0: F::zero(),
            c1: F::zero(),
        }
    }
}

impl<F> Fq2<F>
where
    F: Copy + Zero + One,
{
    #[inline]
    pub fn one() -> Self {
        Self {
            c0: F::one(),
            c1: F::zero(),
        }
    }
}

impl<F> Fq2<F>
where
    F: Copy + Zero + One + Add<Output = F>,
{
    #[inline]
    pub fn two() -> Self {
        let one = F::one();
        Self {
            c0: one + one,
            c1: F::zero(),
        }
    }
}

/* ---------- pow (自前で square = x*x) ---------- */

impl<F> Fq2<F>
where
    F: Copy + Zero + One + Add<Output = F> + Sub<Output = F> + Mul<Output = F> + Neg<Output = F>,
{
    #[inline]
    pub fn pow<T>(&self, exp: T) -> Self
    where
        T: AsRef<[u64]>,
    {
        self.pow_from_bits(exp.as_ref())
    }

    #[inline]
    pub fn pow_u64(&self, exp: u64) -> Self {
        self.pow([exp])
    }

    fn pow_from_bits(&self, bits: &[u64]) -> Self {
        let mut acc = Self::one();
        let mut base = *self;
        for &word in bits {
            let mut mask = word;
            for _ in 0..64 {
                if mask & 1 == 1 {
                    acc = acc * base;
                }
                base = base * base; // square
                mask >>= 1;
            }
        }
        acc
    }
}

/* ---------- inv (num_traits::Inv を使う) ---------- */

impl<F> Fq2<F>
where
    F: Copy
        + Zero
        + One
        + Add<Output = F>
        + Sub<Output = F>
        + Mul<Output = F>
        + Neg<Output = F>
        + Inv<Output = F>,
{
    /// (a + b u)^{-1} = (a - b u) / (a^2 - 3 b^2)
    #[inline]
    pub fn inv(self) -> Option<Self> {
        let a2 = self.c0 * self.c0;
        let b2 = self.c1 * self.c1;
        let three_b2 = b2 + b2 + b2;
        let denom = a2 - three_b2;

        if denom.is_zero() {
            return None;
        }

        let inv = denom.inv();
        Some(Self {
            c0: self.c0 * inv,
            c1: -self.c1 * inv,
        })
    }
}

/* ---------- Add / Sub / Neg ---------- */

impl<F> Add for Fq2<F>
where
    F: Add<Output = F>,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            c0: self.c0 + rhs.c0,
            c1: self.c1 + rhs.c1,
        }
    }
}

impl<F> AddAssign for Fq2<F>
where
    F: AddAssign,
{
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.c0 += rhs.c0;
        self.c1 += rhs.c1;
    }
}

impl<F> Sub for Fq2<F>
where
    F: Sub<Output = F>,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            c0: self.c0 - rhs.c0,
            c1: self.c1 - rhs.c1,
        }
    }
}

impl<F> SubAssign for Fq2<F>
where
    F: SubAssign,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.c0 -= rhs.c0;
        self.c1 -= rhs.c1;
    }
}

impl<F> Neg for Fq2<F>
where
    F: Neg<Output = F>,
{
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            c0: -self.c0,
            c1: -self.c1,
        }
    }
}

/* ---------- Mul (Karatsuba, Δ=3) ---------- */

impl<F> Mul for Fq2<F>
where
    F: Copy + Add<Output = F> + Sub<Output = F> + Mul<Output = F>,
{
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

impl<F> MulAssign for Fq2<F>
where
    F: Copy + Add<Output = F> + Sub<Output = F> + Mul<Output = F>,
{
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

/* ---------- scalar mul/div over base field F ---------- */

impl<F> Mul<F> for Fq2<F>
where
    F: Copy + Mul<Output = F>,
{
    type Output = Self;
    #[inline]
    fn mul(self, s: F) -> Self::Output {
        Self {
            c0: self.c0 * s,
            c1: self.c1 * s,
        }
    }
}

impl<F> MulAssign<F> for Fq2<F>
where
    F: MulAssign + Copy,
{
    #[inline]
    fn mul_assign(&mut self, s: F) {
        self.c0 *= s;
        self.c1 *= s;
    }
}

/* ---------- Sum ---------- */

impl<F> Sum for Fq2<F>
where
    F: Copy + Zero + Add<Output = F>,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, value| acc + value)
    }
}

impl<'a, F> Sum<&'a Fq2<F>> for Fq2<F>
where
    F: Copy + Zero + Add<Output = F>,
{
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, value| acc + *value)
    }
}
