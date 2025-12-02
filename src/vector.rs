use std::{
    iter::{self, Sum},
    ops::Add,
};

use ark_ff::PrimeField;
use waseki::Var;

use crate::{
    fold::CircuitVariable,
    fq::{AlmostGoldilocksField, GoldilocksField},
    fq2::Fq2,
    matrix::Matrix,
};

#[derive(Clone)]
pub struct Vector<T>(pub Vec<T>);

impl<T> Add for Vector<T>
where
    T: Add<Output = T>,
{
    type Output = Vector<T>;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.0.len(), rhs.0.len());
        let vec = self.0.into_iter().zip(rhs.0).map(|(x, y)| x + y).collect();
        Self(vec)
    }
}

impl<T> Sum for Vector<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        todo!()
    }
}

impl<F: PrimeField> CircuitVariable<Matrix<F>> for Matrix<Var<F>> {
    fn value(&self) -> Matrix<F> {
        todo!()
    }

    fn equal(&self, rhs: Self) {
        todo!()
    }
}
