use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_ff::{Field, Fp64, MontBackend, PrimeField};

use crate::utils::ark_poseidon_hash::poseidon_custom_config;

// Almost Goldilock
#[derive(ark_ff::MontConfig)]
#[modulus = "18446744069414584289"] // = (2^64 − 2^32 + 1) − 32
#[generator = "3"]
pub struct AGFqConfig;
pub type AlmostGoldilocksField = Fp64<MontBackend<AGFqConfig, 1>>;
pub type AGFq = AlmostGoldilocksField; // いったん

// WARN: This is not an appropriate parameter.
pub fn poseidon_canonical_config<F: PrimeField>() -> PoseidonConfig<F> {
    let full_rounds = 8;
    let partial_rounds = 60;
    let alpha = 5;
    let rate = 4;

    poseidon_custom_config(full_rounds, partial_rounds, alpha, rate, 1)
}

// Goldilocks Field
#[derive(ark_ff::MontConfig)]
#[modulus = "18446744069414584321"] // = 2^64 − 2^32 + 1 (Goldilocks)
#[generator = "7"]
pub struct GLFqConfig;

pub type GLFq = Fp64<MontBackend<GLFqConfig, 1>>;
