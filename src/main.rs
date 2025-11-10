mod commit;
mod rq;

use crate::commit::MatrixCommitmentScheme;
use ark_ff::UniformRand;
use ark_std::rand::{SeedableRng, rngs::StdRng};

const D: usize = 64;
const KAPPA: usize = 13;
// const M: usize = 2 << 26;
const M: usize = 1000;

pub fn main() {}
