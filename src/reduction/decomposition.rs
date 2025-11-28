use ark_ff::PrimeField;
use waseki::Var;

use crate::{
    K,
    fold::CircuitVariable,
    fq2::Fq2,
    matrix::Matrix,
    reduction::{
        ccs::r_hat,
        random_linear_combination::{challenge, rot},
    },
    vector::Vector,
};

type C<F> = Matrix<Var<F>>;
type X<F> = Matrix<Var<F>>;
type R<F> = Vector<Fq2<F>>;
type Y<F> = Vec<Vector<Fq2<Var<F>>>>; // y_j
type Z<F> = Matrix<F>; // Z^T

pub fn decompose_reduction<F: PrimeField>(
    c_in: C<F>,
    x_in: X<F>,
    r: R<F>,
    y_in: Y<F>,
    z_in: Z<F>,         // Z^T
    m: [&Matrix<F>; 4], // m_0 は単位行列なので、4つ必要か？
) -> (Vec<C<F>>, Vec<X<F>>, Vec<Y<F>>) {
    let r_hat = Vector(r_hat(&r.0));
    let z = z_in.bit_split();
    let c: Vec<C<F>> = z.iter().map(commit).map(|v| v.alloc()).collect();
    let y: Vec<Y<F>> = z
        .iter()
        .map(|z_i| {
            m.into_iter()
                // (M * Z^T)^T * r = Z * M^T * r
                .map(|m_j| (m_j * z_i).transpose() * &r_hat)
                .map(|v| v.alloc())
                .collect()
        })
        .collect();

    let rho: Vec<Matrix<_>> = challenge::<Vector<Var<F>>>().take(K).map(rot).collect();

    // Verify
    // let x = x_in.bit_split(); // 回路内でどうやってXをビット分解する？

    c.iter()
        .zip(&rho)
        .map(|(c_i, rho_i)| {
            let mat = c_i.rows().into_iter().map(|cf| rho_i * &cf).collect();
            Matrix(mat)
        })
        .sum::<C<F>>()
        .equal(c_in);

    y.iter()
        .zip(&rho)
        .map(|(y_i, rho_i)| y_i.into_iter().map(|y_i_j| rho_i * y_i_j).collect())
        .reduce(|acc: Vec<_>, x| acc.into_iter().zip(x).map(|(a, b)| a + b).collect())
        .unwrap()
        .iter()
        .zip(y_in)
        .map(|(y_out_j, y_in_j)| y_out_j.equal(&y_in_j));

    todo!()
}

pub fn commit<F: PrimeField>(z: &Matrix<bool>) -> Matrix<F> {
    todo!()
}
