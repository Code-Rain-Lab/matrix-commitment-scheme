use ark_ff::PrimeField;
use core::marker::PhantomData;
use rayon::prelude::*;

use crate::{rq::Rq, KAPPA, M};

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
