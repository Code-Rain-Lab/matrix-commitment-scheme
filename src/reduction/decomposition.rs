use ark_ff::{Field, PrimeField};

use crate::{
    MatrixCommitmentScheme, Rq,
    almost_goldilock::Fq2,
    reduction::{
        Reduction,
        ccs::ME,
        random_linear_combination::{self, SingleMe},
    },
};

impl<F: PrimeField> Reduction<F> {
    pub fn decompose_reduction(&mut self, me: SingleMe<F>) -> ME<F> {
        let z = split(me.z);
        todo!()
    }
}

pub fn split<F: PrimeField>(z: Vec<Rq<F>>) -> Vec<Vec<Vec<F>>> {
    let z: Vec<_> = z
        .iter()
        .map(|z_i| MatrixCommitmentScheme::bit_decompose_witness(&z_i.coeffs()[..]))
        .collect();
    // matrix Zの全ての要素をビット分解して、k個のmatrixを作る。

    let mut z_0 = vec![];
    let mut z_1 = vec![];
    for z in z {
        let mut z_0_row = vec![];
        let mut z_1_row = vec![];
        for z in z {
            let coeff = z.coeffs();
            z_0_row.push(coeff[0]);
            z_1_row.push(coeff[1]);
        }
        z_0.push(z_0_row);
        z_1.push(z_1_row);
        // k個やる
    }

    vec![z_0, z_1]
}
