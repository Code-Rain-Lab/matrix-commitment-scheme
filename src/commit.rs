use ark_ff::{AdditiveGroup, Field, PrimeField};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use core::marker::PhantomData;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::ops::Mul;
use waseki::Var;

use crate::{
    KAPPA, M,
    fq::{AGFq, GLFq},
    fq2::Fq2,
    matrix::Matrix,
    rq::Rq,
    vector::Vector,
};

pub struct CommitmentScheme<F: PrimeField> {
    seed: String,
    _marker: PhantomData<F>,
}

pub trait Commit<F> {
    fn commit(&self, z: &Matrix<bool>) -> Matrix<F>;
}

impl Commit<GLFq> for CommitmentScheme<GLFq> {
    fn commit(&self, z: &Matrix<bool>) -> Matrix<GLFq> {
        const M: usize = 100;
        const KAPPA: usize = 16;
        const D: usize = 54;
        let seed = "aaaaaaa";
        let mat: Vec<Vec<Vector<GLFq>>> = (0..KAPPA)
            .map(|r| {
                (0..M)
                    .map(|c| reject_sampling::<GLFq, D>(seed, r, c))
                    .collect()
            })
            .collect();
        let vec: Vec<Vector<bool>> = z.rows();

        // mat * vec
        let vec: Vec<_> = mat
            .iter()
            .map(|row| {
                row.iter()
                    .zip(vec.iter())
                    .fold(Vector(vec![GLFq::ZERO; D]), |acc, (a_ij, x_j)| {
                        acc + (a_ij * x_j)
                    })
            })
            .collect();
        Matrix(vec)
    }
}

impl<F: Field> Mul<&Vector<bool>> for &Vector<F> {
    type Output = Vector<F>;

    fn mul(self, rhs: &Vector<bool>) -> Self::Output {
        // rhsがboolであるこを活かして、効率的に内積をする
        todo!()
    }
}

// Vector<GLFq> 用の rot
impl Vector<GLFq> {
    fn rot(&self) -> Matrix<GLFq> {
        todo!()
    }
}

// Vector<AGFq> 用の rot
impl Vector<AGFq> {
    fn rot(&self) -> Matrix<AGFq> {
        todo!()
    }
}

pub fn reject_sampling<F: Field, const D: usize>(
    seed: &str,
    row_idx: usize,
    column_idx: usize,
) -> Vector<F> {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.update(row_idx.to_le_bytes());
    hasher.update(column_idx.to_le_bytes());
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
    Vector(column_vec.to_vec())
}

//------//

impl<F: PrimeField> Vector<Fq2<F>> {
    pub fn alloc(&self) -> Vector<Fq2<Var<F>>> {
        todo!()
    }
}

pub struct MatrixCommitmentScheme<F: PrimeField> {
    seed: String,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> MatrixCommitmentScheme<F> {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            _marker: PhantomData,
        }
    }

    pub fn commit(&self, z: &[Rq<F>]) -> Vec<Rq<F>> {
        (0..KAPPA)
            .into_par_iter()
            .map(|row| self.commit_row(row, z))
            .collect()
    }

    pub fn bit_decompose_witness(witness: &[F]) -> Vec<Rq<F>> {
        witness
            .iter()
            .map(|value| Rq::from_field_element(*value))
            .collect()
    }

    fn commit_row(&self, row: usize, z: &[Rq<F>]) -> Rq<F> {
        z.iter()
            .enumerate()
            .take(M)
            .fold(Rq::zero(), |acc, (column, witness_column)| {
                let matrix_entry = Rq::reject_sampling(&self.seed, row, column);
                let product = matrix_entry.mul(witness_column);
                acc.add(&product)
            })
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use ark_std::test_rng;

    use super::*;
    use ark_ff::{Fp64, MontBackend};

    // Almost Goldilock
    #[derive(ark_ff::MontConfig)]
    #[modulus = "18446744069414584289"] // = (2^64 − 2^32 + 1) − 32
    #[generator = "3"]
    pub struct FqConfig;
    pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

    fn add_vectors(lhs: &[Rq<Fq>], rhs: &[Rq<Fq>]) -> Vec<Rq<Fq>> {
        assert_eq!(lhs.len(), rhs.len());
        lhs.iter().zip(rhs.iter()).map(|(a, b)| a.add(b)).collect()
    }

    #[test]
    pub fn random_linear_combination() {
        let mut rng = test_rng();
        let r = Rq::<Fq>::challenge("seed");

        let witness_len = 32;
        let a: Vec<Fq> = (0..witness_len).map(|_| Fq::rand(&mut rng)).collect();
        let b: Vec<Fq> = (0..witness_len).map(|_| Fq::rand(&mut rng)).collect();

        let a = MatrixCommitmentScheme::bit_decompose_witness(&a);
        let b = MatrixCommitmentScheme::bit_decompose_witness(&b);
        let c = add_vectors(&a, &r.mul_vector(&b));

        let scheme = MatrixCommitmentScheme::new("seed");
        let com_a = scheme.commit(&a);
        let com_b = scheme.commit(&b);
        let com_c = add_vectors(&com_a, &r.mul_vector(&com_b));

        assert_eq!(com_c, scheme.commit(&c))
    }
}
