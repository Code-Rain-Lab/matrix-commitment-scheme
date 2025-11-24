use ark_ff::{Field, Fp64, MontBackend};

// Almost Goldilock
#[derive(ark_ff::MontConfig)]
#[modulus = "18446744069414584289"] // = (2^64 − 2^32 + 1) − 32
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;
