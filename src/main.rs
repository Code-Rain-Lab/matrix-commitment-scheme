mod commit;
mod rq;

use crate::commit::MatrixCommitmentScheme;
use ark_ff::UniformRand;
use ark_std::rand::{SeedableRng, rngs::StdRng};

const D: usize = 64;
const KAPPA: usize = 13;
const M: usize = 2 << 26;

pub fn main() {
    use ark_test_curves::bls12_381::Fr;

    let mut rng = StdRng::seed_from_u64(2024);
    let witness: Vec<Fr> = (0..D).map(|_| Fr::rand(&mut rng)).collect();

    let scheme = MatrixCommitmentScheme::<Fr>::new("matrix_commitment_seed");
    let witness_rq = MatrixCommitmentScheme::<Fr>::bit_decompose_witness(&witness);
    let commitment = scheme.commit(&witness_rq);

    let challenge = MatrixCommitmentScheme::<Fr>::challenge("rotation_challenge_seed");

    println!("commitment rows (κ): {}", KAPPA);
    println!("rotation challenge dimension: {}x{}", challenge.len(), challenge[0].len());
    println!(
        "first commitment column first four coeffs: {:?}",
        &commitment[0].coeffs()[..4]
    );
}
