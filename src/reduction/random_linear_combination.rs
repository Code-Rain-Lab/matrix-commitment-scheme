use ark_ff::PrimeField;

use crate::reduction::{Reduction, ccs::ME, random_linear_combination};

impl<F: PrimeField> Reduction<F> {
    pub fn random_linear_combination_reduction(&mut self, me: ME<F>) {}
}
