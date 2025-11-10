use ark_ff::{BigInteger, Field, PrimeField, UniformRand};
use ark_std::{
    iterable::Iterable,
    rand::{SeedableRng, rngs::StdRng},
};
use sha2::{Digest, Sha256};

fn main() {
    println!("Hello, world!");
}

const D: usize = 64;
const KAPPA: usize = 13;
const M: usize = 2 << 26;

struct Rq<F: Field> {}
impl<F: Field> Rq<F> {
    pub fn reject_sampling(seed: &str, row: usize, column: usize) -> [F; D] {
        todo!()
    }
}

struct MatrixCommitmentScheme {
    seed: String,
}

impl<F: PrimeField> MatrixCommitmentScheme {
    pub fn new(seed: String) -> Self {
        Self { seed }
    }

    pub fn commit(&self, z: Vec<F>) -> Vec<[F; D]> {
        (0..KAPPA)
            // per_iter
            .map(|row| {
                (0..M)
                    .map(|column| {
                        let rq = Rq::<F>::reject_sampling(&self.seed, row, column);
                        let mut sum = [F::ZERO; D];
                        for col_idx in bit_index(z[column]) {
                            for (row_idx, elm) in sum.iter_mut().enumerate() {
                                // col_idxとrow_idxから、rqの回転行列の要素を取り出して、elmに足しくわえる
                            }
                        }
                        sum
                    })
                    .reduce(|acc, x| {
                        let mut sum = [F::ZERO; D];
                        for (a, b) in acc.iter().zip(x) {}
                        sum
                    })
            })
            .collect()
    }
}

pub fn bit_index<F: PrimeField>(element: F) -> Vec<usize> {
    let bit_len = F::MODULUS_BIT_SIZE as usize;
    let bigint = element.into_bigint();
    (0..bit_len)
        .filter_map(|bit_idx| {
            if bigint.get_bit(bit_idx) {
                Some(bit_idx)
            } else {
                None
            }
        })
        .collect()
}

pub fn rot_block<F: Field>(base_column: &[F; N], use_negacyclic: bool) -> Vec<[F; N]> {
    let mut block = vec![[F::ZERO; N]; N];

    for (row_idx, row) in block.iter_mut().enumerate() {
        for (col_idx, cell) in row.iter_mut().enumerate() {
            let offset = ((row_idx as isize - col_idx as isize).rem_euclid(N as isize)) as usize;
            let sign = if use_negacyclic && row_idx < col_idx {
                -F::ONE
            } else {
                F::ONE
            };
            *cell = sign * base_column[offset];
        }
    }
    block
}
