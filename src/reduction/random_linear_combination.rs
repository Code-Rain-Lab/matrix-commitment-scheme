use std::{
    iter,
    ops::{Add, Mul},
    process::Output,
};

use ark_ff::PrimeField;
use itertools::izip;
use num_traits::{One, Zero};

use crate::{
    K, Rq,
    fold::{CircuitVariable, Var},
    fq2::Fq2,
    mat::Mat,
    matrix::Matrix,
    vector::Vector,
};

type C<F> = Matrix<Var<F>>;
type X<F> = Matrix<Var<F>>;
type Y<F> = Matrix<Vector<Fq2<Var<F>>>>;
type Z<F> = Matrix<F>; // Z^T

pub fn aaa<F: PrimeField>(
    c: Vec<C<F>>,
    x: Vec<X<F>>,
    y: Vec<Y<F>>,
    z: Vec<Z<bool>>, // Z^T
) -> (C<F>, X<F>, Y<F>, Z<F>) {
    let rho: Vec<Matrix<_>> = challenge::<Vector<Var<F>>>().take(K).map(_rot).collect();

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

    // let y = y.iter().map(|mat| {});
    let z: Matrix<_> = z
        .into_iter()
        .zip(&rho)
        .map(|(z_i, rho_i)| {
            let rho_i = rho_i.value();
            let mat = z_i.rows().into_iter().map(|cf| &rho_i * cf).collect();
            Matrix(mat)
        })
        .sum();

    todo!()
}

pub fn random_linear_combination_reduction<F: PrimeField>(
    c: Vec<Vec<Var<F>>>,
    x: Vec<Vec<Var<F>>>,
    y: Vec<Vec<Vec<Fq2<Var<F>>>>>,
    z: Vec<Mat<F>>,
) -> (Vec<Var<F>>, Vec<Var<F>>, Vec<Vec<Fq2<Var<F>>>>, Vec<Mat<F>>) {
    assert!(c.len() == x.len() && c.len() == z.len() && c.len() == y.len());

    // Verifier
    let rho: Vec<_> = challenge::<Vec<Var<F>>>().take(K).map(rot).collect();
    let c: Vec<Var<F>> = c
        .iter()
        .zip(&rho)
        .map(|(c_i, rho_i)| rho_i * c_i)
        .reduce(add_vector)
        .unwrap();
    let x: Vec<Var<F>> = x
        .iter()
        .zip(&rho)
        .map(|(x_i, rho_i)| rho_i * x_i)
        .reduce(add_vector)
        .unwrap();
    let y = y
        .iter()
        .zip(&rho)
        .map(|(y_i, rho_i)| {
            y_i.iter()
                .map(|y_i_j| rho_i * y_i_j)
                .collect::<Vec<Vec<_>>>()
        })
        .fold(vec![vec![Fq2::zero(); K + 1]; 3], |acc, x| {
            acc.into_iter()
                .zip(x)
                .map(|(a, b)| add_vector(a, b))
                .collect()
        });

    // Prover
    let z: Vec<_> = z
        .into_iter()
        .zip(rho)
        .map(|(z_i, rho_i)| fast_mul(rho_i.value(), z_i))
        .collect();

    (c, x, y, z)
}

fn add_vector<F: Add<Output = F>>(a: Vec<F>, b: Vec<F>) -> Vec<F> {
    a.into_iter().zip(b).map(|(a, b)| a + b).collect()
}

pub fn _rot<F: PrimeField>(a: Vector<Var<F>>) -> Matrix<Var<F>> {
    todo!()
}

pub fn rot<F: PrimeField>(a: Vec<Var<F>>) -> Mat<Var<F>> {
    todo!()
}

// ρ * Z は並列で行いたいので処理をわける
pub fn fast_mul<F: PrimeField>(rho: Mat<F>, z: Mat<F>) -> Mat<F> {
    todo!()
}

pub fn challenge<T>() -> impl Iterator<Item = T> {
    // ダミー
    // ここで、チャレンジを生成する。hashとかで
    iter::successors(None, move |p| None)
}

// impl<F: PrimeField> Reduction<F> {
//     pub fn random_linear_combination_reduction(&mut self, me: ME<F>) -> SingleMe<F> {
//         // circuit generates random
//         // let rho = vec![..];
//         // c, x, y をrhoで結合する。
//
//         // proverがそのrho.valueを使って、Zを結合する
//
//         todo!()
//     }
// }
