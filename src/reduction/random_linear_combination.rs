use ark_ff::PrimeField;

use crate::{
    Rq,
    almost_goldilock::Fq2,
    reduction::{Reduction, random_linear_combination},
};

pub struct SingleMe<F: PrimeField> {
    pub c: Vec<Rq<F>>,
    pub z: Vec<Rq<F>>,
    pub r: Fq2<F>,           // the size is log N, N is the number of constraints
    pub y: Vec<Vec<Fq2<F>>>, // the size is t, which is number of CCS matrix
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
