use std::{
    iter::Sum,
    ops::{Add, Mul},
};

use ark_ff::PrimeField;
use waseki::Var;

use crate::{fq2::Fq2, vector::Vector};

#[derive(Clone, Debug)]
pub struct Matrix<T>(pub Vec<Vector<T>>);

// これはCCSのMがスパースなのでこれを用意する
#[derive(Clone)]
pub struct SparseMatrix<F: PrimeField>(pub Vec<Vec<(usize, F)>>);

impl<T: Clone> Matrix<T> {
    pub fn rows(&self) -> Vec<Vector<T>> {
        self.0.clone()
    }

    pub fn transpose(self) -> Self {
        todo!()
    }

    pub fn flatten(&self) -> Vec<T> {
        todo!()
    }
}

impl<F: PrimeField> Matrix<F> {
    pub fn bit_split(self) -> Vec<Matrix<bool>> {
        todo!()
    }

    pub fn alloc(&self) -> Matrix<Var<F>> {
        todo!()
    }
}

// Matrix<F> * Vector<Bool> -> Vector<F>
impl<F: PrimeField> Mul<Vector<bool>> for Matrix<F> {
    type Output = Vector<F>;

    // bool であることを利用して最適化する
    fn mul(self, rhs: Vector<bool>) -> Self::Output {
        todo!()
    }
}

// &Matrix<F> * Vector<Bool> -> Vector<F>
// impl<F: PrimeField> Mul<Vector<bool>> for &Matrix<F> {
//     type Output = Vector<F>;
//
//     // bool であることを利用して最適化する
//     fn mul(self, rhs: Vector<bool>) -> Self::Output {
//         todo!()
//     }
// }

// Matrix<F> * &Vector<Fq2> -> Vector<Fq2>
impl<F: PrimeField> Mul<&Vector<Fq2<F>>> for Matrix<F> {
    type Output = Vector<Fq2<F>>;

    // bool であることを利用して最適化する
    fn mul(self, rhs: &Vector<Fq2<F>>) -> Self::Output {
        todo!()
    }
}

// Matrix<F> * Matrix<Bool> -> Vector<F>
impl<F: PrimeField> Mul<Matrix<bool>> for Matrix<F> {
    type Output = Matrix<F>;

    fn mul(self, rhs: Matrix<bool>) -> Self::Output {
        todo!()
    }
}

// &Matrix<F> * &Matrix<Bool> -> Vector<F>
impl<F: PrimeField> Mul<&Matrix<bool>> for &Matrix<F> {
    type Output = Matrix<F>;

    fn mul(self, rhs: &Matrix<bool>) -> Self::Output {
        todo!()
    }
}

// Matrix<Var> * Vector<Var> -> Vector<Var>
impl<F: PrimeField> Mul<Vector<Var<F>>> for Matrix<Var<F>> {
    type Output = Vector<Var<F>>;

    fn mul(self, rhs: Vector<Var<F>>) -> Self::Output {
        todo!()
    }
}

// &Matrix<Var> * &Vector<Var> -> Vector<Var>
impl<F: PrimeField> Mul<&Vector<Var<F>>> for &Matrix<Var<F>> {
    type Output = Vector<Var<F>>;

    fn mul(self, rhs: &Vector<Var<F>>) -> Self::Output {
        todo!()
    }
}

// &Matrix<Fq2<Var>> * Vector<Var> -> Vector<Var>
impl<F: PrimeField> Mul<Vector<Fq2<Var<F>>>> for &Matrix<Var<F>> {
    type Output = Vector<Fq2<Var<F>>>;

    fn mul(self, rhs: Vector<Fq2<Var<F>>>) -> Self::Output {
        todo!()
    }
}

// &Matrix<Fq2<Var>> * &Vector<Var> -> Vector<Var>
impl<F: PrimeField> Mul<&Vector<Fq2<Var<F>>>> for &Matrix<Var<F>> {
    type Output = Vector<Fq2<Var<F>>>;

    fn mul(self, rhs: &Vector<Fq2<Var<F>>>) -> Self::Output {
        todo!()
    }
}

impl<T> Add for Matrix<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.0.len(), rhs.0.len(), "matrix row count mismatch");
        let rows = self.0.into_iter().zip(rhs.0).map(|(l, r)| l + r).collect();
        Matrix(rows)
    }
}

impl<T> Sum for Matrix<T>
where
    T: Add<Output = T>,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut it = iter;
        let mut acc = match it.next() {
            Some(first) => first,
            None => return Matrix(Vec::new()),
        };
        for m in it {
            acc = acc + m;
        }
        acc
    }
}
