use std::ops::Mul;

use ark_ff::PrimeField;

use crate::fq2::Fq2;

pub struct Mat<T>(Vec<Vec<T>>);

impl<T> Mat<T> {
    pub fn flatten(&self) -> Vec<&T> {
        self.0.iter().flatten().collect()
    }
}

// FqMatrix * FqMatrix -> FqMatrix
// FqMatrix * Fq2Vector -> Fq2Vector
impl<F: PrimeField> Mul<&Mat<F>> for &Mat<F> {
    type Output = Mat<F>;

    fn mul(self, rhs: &Mat<F>) -> Self::Output {
        todo!()
    }
}

impl<F: PrimeField> Mul<&Vec<Fq2<F>>> for &Mat<F> {
    type Output = Vec<Fq2<F>>;

    fn mul(self, rhs: &Vec<Fq2<F>>) -> Self::Output {
        todo!()
    }
}
