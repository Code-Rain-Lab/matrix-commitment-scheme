use std::{iter, ops::Mul};

use ark_ff::PrimeField;
use itertools::izip;
use num_traits::{One, Zero};

use crate::{Rq, fold::Var, fq2::Fq2, mat::Mat};

pub fn random_linear_combination_reduction<F: PrimeField>(
    c: Vec<Mat<Var<F>>>,
    x: Vec<Mat<Var<F>>>,
    r: Vec<Fq2<Var<F>>>,
    y: Vec<Vec<Fq2<Var<F>>>>,
    z: Vec<Mat<F>>,
) {
    assert!(c.len() == x.len() && c.len() == z.len() && c.len() == y.len());

    let mut rho = challenge::<Fq2<Var<F>>>(); // チャレンジは行列のはず
    izip!(c, x, y, z, rho.by_ref()).map(|(c_i, x_i, y_i, z_i, rho_i)| {
        // let c = rho_i * c_i;
        // let x = rho_i * x_i;
        // let y = rho_i * y_i;
        // let z = rho_i.value() * z_i;
        // (c, x, z)
    });
    // .fold()
}

pub fn challenge<T>() -> impl Iterator<Item = T>
where
    T: Copy + Mul<Output = T> + One,
{
    // ダミー
    // ここで、チャレンジを生成する。hashとかで
    iter::successors(Some(T::one()), move |p| Some(*p))
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
