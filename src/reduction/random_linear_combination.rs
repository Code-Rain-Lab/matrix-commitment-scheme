use std::{iter, ops::Add};

use ark_ff::PrimeField;
use waseki::Var;

use crate::{K, Rq, fold::CircuitVariable, fq2::Fq2, matrix::Matrix, vector::Vector};

type C<F> = Matrix<Var<F>>;
type X<F> = Matrix<Var<F>>;
type Y<F> = Vec<Vector<Fq2<Var<F>>>>; // y_j
type Z<F> = Matrix<F>; // Z^T

pub fn random_linear_combination_reduction<F: PrimeField>(
    c: Vec<C<F>>,
    x: Vec<X<F>>,
    y: Vec<Y<F>>,
    z: Vec<Z<bool>>, // Z^T
) -> (C<F>, X<F>, Y<F>, Z<F>) {
    let rho: Vec<Matrix<_>> = challenge::<Vector<Var<F>>>().take(K).map(rot).collect();

    let c: Matrix<_> = c
        .into_iter()
        .zip(&rho)
        .map(|(c_i, rho_i)| {
            let mat = c_i.rows().into_iter().map(|cf| rho_i * &cf).collect();
            Matrix(mat)
        })
        .sum();

    let x: Matrix<_> = x
        .into_iter()
        .zip(&rho)
        .map(|(x_i, rho_i)| {
            let mat = x_i.rows().into_iter().map(|cf| rho_i * &cf).collect();
            Matrix(mat)
        })
        .sum();

    let y = y
        .into_iter()
        .zip(&rho)
        .map(|(y_i, rho_i)| y_i.into_iter().map(|y_i_j| rho_i * y_i_j).collect())
        .reduce(|acc: Vec<_>, x| acc.into_iter().zip(x).map(|(a, b)| a + b).collect())
        .unwrap();

    let z: Matrix<_> = z
        .into_iter()
        .zip(&rho)
        .map(|(z_i, rho_i)| {
            let rho_i = rho_i.value();
            let mat = z_i.rows().into_iter().map(|cf| rho_i.clone() * cf).collect();
            Matrix(mat)
        })
        .sum();

    (c, x, y, z)
}

pub fn rot<F: PrimeField>(a: Vector<Var<F>>) -> Matrix<Var<F>> {
    todo!()
}

pub fn challenge<T>() -> impl Iterator<Item = T> {
    // ダミー
    // ここで、チャレンジを生成する。hashとかで
    iter::successors(None, move |p| None)
}
