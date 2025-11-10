use ark_ff::{BigInteger, Field, PrimeField, UniformRand};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use core::marker::PhantomData;
use sha2::{Digest, Sha256};

const D: usize = 64;
const KAPPA: usize = 13;
const M: usize = 2 << 26;

struct Rq;
impl Rq {
    pub fn reject_sampling<F: Field>(seed: &str, row: usize, column: usize) -> [F; D] {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        hasher.update(row.to_le_bytes());
        hasher.update(column.to_le_bytes());
        let digest = hasher.finalize();
        let mut rng_seed = [0u8; 32];
        rng_seed.copy_from_slice(&digest);
        let mut rng = StdRng::from_seed(rng_seed);

        let mut column_vec = [F::ZERO; D];
        for element in column_vec.iter_mut() {
            // Reject zero samples to avoid degenerate columns.
            let sampled = loop {
                let candidate = F::rand(&mut rng);
                if !candidate.is_zero() {
                    break candidate;
                }
            };
            *element = sampled;
        }
        column_vec
    }
}

struct MatrixCommitmentScheme<F: PrimeField> {
    seed: String,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> MatrixCommitmentScheme<F> {
    pub fn new(seed: String) -> Self {
        Self {
            seed,
            _marker: PhantomData,
        }
    }

    pub fn commit(&self, z: Vec<F>) -> Vec<[F; D]> {
        (0..KAPPA)
            .map(|row| {
                let mut aggregate = [F::ZERO; D];
                for (column, element) in z.iter().enumerate().take(M) {
                    let rq_column = Rq::reject_sampling::<F>(&self.seed, row, column);
                    let rotations = rot_block::<F, D>(&rq_column, true);
                    for bit_idx in bit_index(*element) {
                        if bit_idx >= D {
                            continue;
                        }
                        for (row_idx, value) in aggregate.iter_mut().enumerate() {
                            *value += rotations[row_idx][bit_idx];
                        }
                    }
                }
                aggregate
            })
            .collect()
    }
}

pub fn bit_index<F: PrimeField>(element: F) -> Vec<usize> {
    let bit_len = F::MODULUS_BIT_SIZE as usize;
    let bigint = element.into_bigint();
    (0..bit_len)
        .filter(|bit_idx| bigint.get_bit(*bit_idx))
        .collect()
}

pub fn rot_block<F: Field, const N: usize>(
    base_column: &[F; N],
    use_negacyclic: bool,
) -> Vec<[F; N]> {
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

fn main() {
    use ark_test_curves::bls12_381::Fr;

    let mut rng = StdRng::seed_from_u64(2024);
    let witness: Vec<Fr> = (0..32).map(|_| Fr::rand(&mut rng)).collect();

    let scheme = MatrixCommitmentScheme::<Fr>::new("matrix_commitment_seed".into());
    let commitments = scheme.commit(witness);

    for (row_idx, row) in commitments.iter().enumerate().take(3) {
        println!("row {row_idx} first four entries: {:?}", &row[..4]);
    }
}
