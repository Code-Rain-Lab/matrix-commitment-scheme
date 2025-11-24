use ark_ff::{Field, One, PrimeField};
use itertools::izip;

use crate::{
    K,
    fold::{CircuitVariable, Var},
    fq2::Fq2,
    mat::Mat,
    reduction::ccs::{powers_of, r_hat},
};

pub fn decompose_reduction<F: PrimeField>(
    c: Vec<Var<F>>,
    x: Mat<Var<F>>,
    r: Vec<Fq2<Var<F>>>,
    y: Vec<Fq2<Var<F>>>,
    z: Mat<F>,
    mt: [Mat<F>; 3],
) {
    let r_hat = r_hat(&r.value());
    let z = split(z);
    let c: Vec<_> = z.iter().map(commit).collect();
    let y: Vec<Vec<_>> = z
        .iter()
        .map(|z_i| mt.iter().map(|mt_j| &(z_i * mt_j) * &r_hat).collect())
        .collect();

    // verify
    let b = Var::<F>::one() + Var::<F>::one();
    let mut pow_b = powers_of(b).take(K);

    c.iter().zip(pow_b).map(|(c_i, b_i)| {
        // let c = b_i * c_i;
    });
    // .fold().equal()

    // yはj個で、jのうちのiをbで結合する
}

pub fn commit<F: PrimeField>(z: &Mat<F>) -> Vec<Var<F>> {
    todo!()
}

pub fn split<F: PrimeField>(z: Mat<F>) -> Vec<Mat<F>> {
    todo!()
}

// impl<F: PrimeField> Reduction<F> {
//     pub fn decompose_reduction(&mut self, me: SingleMe<F>) -> ME<F> {
//         // prover
//         // let z = split(me.z);
//         // let c = z.iter().map(|z_i| commit).collect();
//         // let y =
//
//         // verifier
//         // c_i を2^iで結合
//         // y_i も同様
//         // 入力のc,yとの一致をみる
//         todo!()
//     }
// }

// pub fn split<F: PrimeField>(z: Vec<Rq<F>>) -> Vec<Vec<Vec<F>>> {
//     let z: Vec<_> = z
//         .iter()
//         .map(|z_i| MatrixCommitmentScheme::bit_decompose_witness(&z_i.coeffs()[..]))
//         .collect();
//     // matrix Zの全ての要素をビット分解して、k個のmatrixを作る。
//
//     let mut z_0 = vec![];
//     let mut z_1 = vec![];
//     for z in z {
//         let mut z_0_row = vec![];
//         let mut z_1_row = vec![];
//         for z in z {
//             let coeff = z.coeffs();
//             z_0_row.push(coeff[0]);
//             z_1_row.push(coeff[1]);
//         }
//         z_0.push(z_0_row);
//         z_1.push(z_1_row);
//         // k個やる
//     }
//
//     vec![z_0, z_1]
// }
