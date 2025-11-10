use ark_ff::PrimeField;

use crate::{D, KAPPA, M, rq::Rq};

pub struct MatrixCommitmentScheme<F: PrimeField> {
    matrix: Vec<Vec<Rq<F>>>,
}
impl<F: PrimeField> MatrixCommitmentScheme<F> {
    pub fn new(seed: &str) -> Self {
        let matrix = (0..KAPPA)
            .map(|row| {
                (0..M)
                    .map(|col| Rq::<F>::reject_sampling(seed, row, col))
                    .collect()
            })
            .collect();
        Self { matrix }
    }

    pub fn commit(&self, z: Vec<Rq<F>>) -> [Rq<F>; KAPPA] {
        let c: Vec<_> = self
            .matrix
            .iter()
            .map(|row_vec| {
                row_vec
                    .iter()
                    .zip(&z)
                    .map(|(a, b)| a.mul(b))
                    .reduce(|acc, x| acc.add(&x))
            })
            .collect();
        c
    }

    pub fn challenge(seed: &str) -> [Rq<F>; D] {
        // [-1, 0, 1, 2] の中からランダムで、[F; D]を作って、Rqにする。それをD個作って返す。
        todo!()
    }
}
