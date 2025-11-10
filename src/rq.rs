use ark_ff::{BigInteger, PrimeField};
use ark_std::rand::{Rng, SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};

use crate::D;

#[derive(Clone, Debug, PartialEq)]
pub struct Rq<F: PrimeField> {
    coeffs: [F; D],
}

impl<F: PrimeField> Rq<F> {
    pub fn zero() -> Self {
        Self {
            coeffs: [F::ZERO; D],
        }
    }

    pub fn coeffs(&self) -> &[F; D] {
        &self.coeffs
    }

    /// Negacyclic convolution: self * rhs mod x^D + 1
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut result = [F::ZERO; D];
        for (i, a_coeff) in self.coeffs.iter().enumerate() {
            for (j, b_coeff) in rhs.coeffs.iter().enumerate() {
                if b_coeff == &F::ZERO {
                    continue;
                }
                let idx = (i + j) % D;
                let sign = if i + j >= D { -F::ONE } else { F::ONE };
                let mut term = *a_coeff;
                term.mul_assign(b_coeff);
                term.mul_assign(&sign);
                result[idx].add_assign(&term);
            }
        }
        Self { coeffs: result }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        let mut c = self.coeffs;
        for (ci, b) in c.iter_mut().zip(rhs.coeffs.iter()) {
            ci.add_assign(b);
        }
        Self { coeffs: c }
    }

    pub fn mul_vector(&self, rhs: &Vec<Self>) -> Vec<Self> {
        rhs.iter().map(|a| self.mul(a)).collect()
    }

    pub fn from_field_element(zq: F) -> Self {
        let bigint = zq.into_bigint();
        let mut coeffs = [F::ZERO; D];
        for (i, coeff) in coeffs.iter_mut().enumerate() {
            if bigint.get_bit(i) {
                *coeff = F::ONE;
            }
        }
        Self { coeffs }
    }

    pub fn reject_sampling(seed: &str, row: usize, column: usize) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        hasher.update(row.to_le_bytes());
        hasher.update(column.to_le_bytes());
        let digest = hasher.finalize();
        let mut rng_seed = [0u8; 32];
        rng_seed.copy_from_slice(&digest);
        let mut rng = StdRng::from_seed(rng_seed);

        let mut column_vec = [F::ZERO; D];
        for element in column_vec.iter_mut() {
            let sampled = loop {
                let candidate = F::rand(&mut rng);
                if !candidate.is_zero() {
                    break candidate;
                }
            };
            *element = sampled;
        }
        Self { coeffs: column_vec }
    }

    pub fn challenge(seed: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let digest = hasher.finalize();
        let mut rng_seed = [0u8; 32];
        rng_seed.copy_from_slice(&digest);
        let mut rng = StdRng::from_seed(rng_seed);

        let mut column_vec = [F::ZERO; D];
        for coeff in column_vec.iter_mut() {
            let draw = rng.gen_range(0..4);
            *coeff = match draw {
                0 => -F::ONE,
                1 => F::ZERO,
                2 => F::ONE,
                _ => F::from(2u64),
            };
        }
        Self { coeffs: column_vec }
    }

    pub fn negacyclic_rot_block(base_column: &[F; D]) -> [[F; D]; D] {
        core::array::from_fn(|row_idx| {
            core::array::from_fn(|col_idx| {
                let offset =
                    ((row_idx as isize - col_idx as isize).rem_euclid(D as isize)) as usize;
                let mut value = base_column[offset];
                if row_idx < col_idx {
                    value = -value;
                }
                value
            })
        })
    }
}

impl<F: PrimeField> Default for Rq<F> {
    fn default() -> Self {
        Self::zero()
    }
}
