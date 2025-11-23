use ark_ff::{Field, Fp64, MontBackend};

// Almost Goldilock
#[derive(ark_ff::MontConfig)]
#[modulus = "18446744069414584289"] // = (2^64 − 2^32 + 1) − 32
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

use core::{
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

// Note: 回路側でも結局同じことを実装するので、arkworksの拡大体のライブラリは使わない。できるだけ実装は揃えたい。

// /// x = c0 + c1 * u, where u^2 = 3 (Δ = 3).
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub struct Fq2<F: Field> {
//     pub c0: F,
//     pub c1: F,
// }
//
// impl<F: Field> From<F> for Fq2<F> {
//     fn from(value: F) -> Self {
//         Self::new(value, F::ZERO)
//     }
// }
//
// impl<F: Field> From<&F> for Fq2<F> {
//     fn from(value: &F) -> Self {
//         Self::new(*value, F::ZERO)
//     }
// }
//
// impl<F: Field> Fq2<F> {
//     #[inline]
//     pub fn zero() -> Self {
//         Self {
//             c0: F::zero(),
//             c1: F::zero(),
//         }
//     }
//     #[inline]
//     pub fn one() -> Self {
//         Self {
//             c0: F::one(),
//             c1: F::zero(),
//         }
//     }
//
//     #[inline]
//     pub fn two() -> Self {
//         Self::one() + Self::one()
//     }
//
//     #[inline]
//     pub fn new(c0: F, c1: F) -> Self {
//         Self { c0, c1 }
//     }
//
//     #[inline]
//     pub fn pow<T>(&self, exp: T) -> Self
//     where
//         T: AsRef<[u64]>,
//     {
//         self.pow_from_bits(exp.as_ref())
//     }
//
//     #[inline]
//     pub fn pow_u64(&self, exp: u64) -> Self {
//         self.pow([exp])
//     }
//
//     fn pow_from_bits(&self, bits: &[u64]) -> Self {
//         let mut acc = Self::one();
//         let mut base = *self;
//         for &word in bits.iter() {
//             let mut mask = word;
//             for _ in 0..64 {
//                 if mask & 1 == 1 {
//                     acc *= base;
//                 }
//                 base *= base;
//                 mask >>= 1;
//             }
//         }
//         acc
//     }
//
//     #[inline]
//     pub fn inv(self) -> Option<Self> {
//         // (a + b u)^{-1} = (a - b u) / (a^2 - 3 b^2)
//         let a2 = self.c0.square();
//         let b2 = self.c1.square();
//         let three_b2 = b2 + b2 + b2;
//         let denom = a2 - three_b2;
//         denom.inverse().map(|inv| Self {
//             c0: self.c0 * inv,
//             c1: -self.c1 * inv,
//         })
//     }
// }
//
// /* ---------- Add / Sub / Neg ---------- */
//
// impl<F: Field> Add for Fq2<F> {
//     type Output = Self;
//     #[inline]
//     fn add(self, rhs: Self) -> Self::Output {
//         Self {
//             c0: self.c0 + rhs.c0,
//             c1: self.c1 + rhs.c1,
//         }
//     }
// }
// impl<F: Field> AddAssign for Fq2<F> {
//     #[inline]
//     fn add_assign(&mut self, rhs: Self) {
//         self.c0 += rhs.c0;
//         self.c1 += rhs.c1;
//     }
// }
//
// impl<F: Field> Sub for Fq2<F> {
//     type Output = Self;
//     #[inline]
//     fn sub(self, rhs: Self) -> Self::Output {
//         Self {
//             c0: self.c0 - rhs.c0,
//             c1: self.c1 - rhs.c1,
//         }
//     }
// }
// impl<F: Field> SubAssign for Fq2<F> {
//     #[inline]
//     fn sub_assign(&mut self, rhs: Self) {
//         self.c0 -= rhs.c0;
//         self.c1 -= rhs.c1;
//     }
// }
//
// impl<F: Field> Neg for Fq2<F> {
//     type Output = Self;
//     #[inline]
//     fn neg(self) -> Self::Output {
//         Self {
//             c0: -self.c0,
//             c1: -self.c1,
//         }
//     }
// }
//
// /* ---------- Mul (Karatsuba 3 muls, Δ=3) ---------- */
//
// impl<F: Field> Mul for Fq2<F> {
//     type Output = Self;
//     #[inline]
//     fn mul(self, rhs: Self) -> Self::Output {
//         // (a + b u)(c + d u) = (ac + 3 bd) + (ad + bc) u
//         let (a, b) = (self.c0, self.c1);
//         let (c, d) = (rhs.c0, rhs.c1);
//         let t0 = a * c; // ac
//         let t1 = b * d; // bd
//         let t2 = (a + b) * (c + d); // (a+b)(c+d) = ac + ad + bc + bd
//         let three_t1 = t1 + t1 + t1; // 3*bd
//         let c0 = t0 + three_t1; // ac + 3bd
//         let c1 = t2 - t0 - t1; // ad + bc
//         Self { c0, c1 }
//     }
// }
// impl<F: Field> MulAssign for Fq2<F> {
//     #[inline]
//     fn mul_assign(&mut self, rhs: Self) {
//         let (a, b) = (self.c0, self.c1);
//         let (c, d) = (rhs.c0, rhs.c1);
//         let t0 = a * c;
//         let t1 = b * d;
//         let t2 = (a + b) * (c + d);
//         let three_t1 = t1 + t1 + t1;
//         self.c0 = t0 + three_t1;
//         self.c1 = t2 - t0 - t1;
//     }
// }
//
// /* ---------- Scalar mul/div over base field F ---------- */
//
// impl<F: Field> Mul<F> for Fq2<F> {
//     type Output = Self;
//     #[inline]
//     fn mul(self, s: F) -> Self::Output {
//         Self {
//             c0: self.c0 * s,
//             c1: self.c1 * s,
//         }
//     }
// }
// impl<F: Field> MulAssign<F> for Fq2<F> {
//     #[inline]
//     fn mul_assign(&mut self, s: F) {
//         self.c0 *= s;
//         self.c1 *= s;
//     }
// }
//
// impl<F: Field> Sum for Fq2<F> {
//     fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.fold(Self::zero(), |acc, value| acc + value)
//     }
// }
//
// impl<'a, F: Field> Sum<&'a Fq2<F>> for Fq2<F> {
//     fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
//         iter.fold(Self::zero(), |acc, value| acc + *value)
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ark_ff::UniformRand;
//     use ark_std::test_rng;
//
//     #[test]
//     fn fq2_add_sub_roundtrip() {
//         let mut rng = test_rng();
//         let a = Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng));
//         let b = Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng));
//
//         let sum = a + b;
//         let diff = sum - b;
//         assert_eq!(diff, a);
//
//         let mut assign_sum = a;
//         assign_sum += b;
//         assert_eq!(assign_sum, sum);
//     }
//
//     #[test]
//     fn fq2_mul_matches_formula() {
//         let mut rng = test_rng();
//         let a = Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng));
//         let b = Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng));
//
//         let prod = a * b;
//
//         let expected = {
//             let (ac, bd) = (a.c0 * b.c0, a.c1 * b.c1);
//             let three_bd = bd + bd + bd;
//             let c0 = ac + three_bd;
//             let c1 = (a.c0 + a.c1) * (b.c0 + b.c1) - ac - bd;
//             Fq2::new(c0, c1)
//         };
//         assert_eq!(prod, expected);
//
//         let mut assign = a;
//         assign *= b;
//         assert_eq!(assign, prod);
//     }
//
//     #[test]
//     fn fq2_inverse_exists_for_nonzero() {
//         let mut rng = test_rng();
//         for _ in 0..32 {
//             let candidate = loop {
//                 let a = Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng));
//                 if a != Fq2::zero() {
//                     break a;
//                 }
//             };
//             let inv = candidate.inv().expect("inverse should exist");
//             let check = candidate * inv;
//             assert_eq!(check, Fq2::one());
//         }
//     }
//
//     #[test]
//     fn fq2_pow_matches_repeated_mul() {
//         let mut rng = test_rng();
//         let base = Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng));
//         let mut manual = Fq2::one();
//         for _ in 0..5 {
//             manual *= base;
//         }
//         assert_eq!(base.pow_u64(5), manual);
//         assert_eq!(base.pow_u64(0), Fq2::one());
//         assert_eq!(base.pow_u64(1), base);
//     }
//
//     #[test]
//     fn fq2_sum_trait_accumulates() {
//         let mut rng = test_rng();
//         let values: Vec<_> = (0..16)
//             .map(|_| Fq2::new(Fq::rand(&mut rng), Fq::rand(&mut rng)))
//             .collect();
//         let iter_sum: Fq2<Fq> = values.clone().into_iter().sum();
//         let manual = values.iter().fold(Fq2::zero(), |acc, value| acc + *value);
//         assert_eq!(iter_sum, manual);
//     }
// }
