use ark_ff::PrimeField;
use core::marker::PhantomData;

use crate::{D, KAPPA, M, rq::Rq};

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

    pub fn commit(&self, z: &[Rq<F>]) -> [Rq<F>; KAPPA] {
        core::array::from_fn(|row| self.commit_row(row, z))
    }

    pub fn bit_decompose_witness(witness: &[F]) -> Vec<Rq<F>> {
        witness
            .iter()
            .map(|value| Rq::from_field_element(*value))
            .collect()
    }

    pub fn challenge(seed: &str) -> [Rq<F>; D] {
        Rq::rotation_block_from_seed(seed)
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
