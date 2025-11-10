use ark_ff::{BigInteger, Field, PrimeField, UniformRand};
use ark_std::rand::{Rng, SeedableRng, rngs::StdRng};
use core::marker::PhantomData;
use rayon::prelude::*;
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
            .into_par_iter()
            .map(|row| {
                let mut aggregate = [F::ZERO; D];
                for (column, element) in z.iter().enumerate().take(M) {
                    let rq_column = Rq::reject_sampling::<F>(&self.seed, row, column);
                    for bit_idx in bit_index(*element) {
                        if bit_idx >= D {
                            continue;
                        }
                        for (row_idx, value) in aggregate.iter_mut().enumerate() {
                            let offset = ((row_idx as isize - bit_idx as isize)
                                .rem_euclid(D as isize))
                                as usize;
                            let sign = if row_idx < bit_idx { -F::ONE } else { F::ONE };
                            *value += rq_column[offset] * sign;
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

pub fn random_rotation_matrix<F: Field, R: Rng + ?Sized>(rng: &mut R) -> Vec<[F; D]> {
    let base_column = {
        let mut column = [F::ZERO; D];
        for element in column.iter_mut() {
            let draw = rng.next_u32() % 4;
            *element = match draw {
                0 => -F::ONE,
                1 => F::ZERO,
                2 => F::ONE,
                _ => F::from(2u64),
            };
        }
        column
    };
    negacyclic_rot_block(&base_column)
}

fn negacyclic_rot_block<F: Field>(base_column: &[F; D]) -> Vec<[F; D]> {
    let mut block = vec![[F::ZERO; D]; D];
    for (row_idx, row) in block.iter_mut().enumerate() {
        for (col_idx, cell) in row.iter_mut().enumerate() {
            let offset = ((row_idx as isize - col_idx as isize).rem_euclid(D as isize)) as usize;
            let sign = if row_idx < col_idx { -F::ONE } else { F::ONE };
            *cell = sign * base_column[offset];
        }
    }
    block
}

// I wnat to apply the d*d rotation matrix to d*m matrix. (not to the ring)
fn apply_rotation<F: Field>(rotation: &[[F; D]], vector: &[F; D]) -> [F; D] {
    let mut result = [F::ZERO; D];
    for (row_idx, row) in rotation.iter().enumerate() {
        let mut acc = F::ZERO;
        for (col_idx, coeff) in row.iter().enumerate() {
            acc += *coeff * vector[col_idx];
        }
        result[row_idx] = acc;
    }
    result
}

fn negacyclic_mul<F: Field>(a: &[F; D], b: &[F; D]) -> [F; D] {
    let mut result = [F::ZERO; D];
    for (i, a_coeff) in a.iter().enumerate() {
        for (j, b_coeff) in b.iter().enumerate() {
            let idx = (i + j) % D;
            let sign = if i + j >= D { -F::ONE } else { F::ONE };
            result[idx] += *a_coeff * *b_coeff * sign;
        }
    }
    result
}

fn random_ring_element<F: Field, R: Rng + ?Sized>(rng: &mut R) -> [F; D] {
    let mut element = [F::ZERO; D];
    for coeff in element.iter_mut() {
        *coeff = F::rand(rng);
    }
    element
}

fn check_rotation_identity<F: Field, R: Rng + ?Sized>(rng: &mut R) -> bool {
    let a = random_ring_element::<F, _>(rng);
    let b = random_ring_element::<F, _>(rng);
    let rotation = negacyclic_rot_block(&a);
    let lhs = apply_rotation(rotation.as_slice(), &b);
    let rhs = negacyclic_mul(&a, &b);
    lhs == rhs
}

fn verify_challenge_linearity<F: PrimeField>(
    scheme: &MatrixCommitmentScheme<F>,
    rotation: &[[F; D]],
    witness: &[F],
) -> bool {
    assert_eq!(
        witness.len(),
        D,
        "linearity check currently expects a D-sized witness"
    );
    let commitments = scheme.commit(witness.to_vec());
    let lhs: Vec<[F; D]> = commitments
        .iter()
        .map(|row| apply_rotation(rotation, row))
        .collect();

    let mut witness_array = [F::ZERO; D];
    witness_array.copy_from_slice(witness);
    let challenged_witness = apply_rotation(rotation, &witness_array);
    let rhs = scheme.commit(challenged_witness.to_vec());
    lhs == rhs
}

fn main() {
    use ark_test_curves::bls12_381::Fr;

    let mut rng = StdRng::seed_from_u64(2024);
    let witness: Vec<Fr> = (0..D).map(|_| Fr::rand(&mut rng)).collect();

    let scheme = MatrixCommitmentScheme::<Fr>::new("matrix_commitment_seed".into());
    let commitments = scheme.commit(witness.clone());
    let rotation_matrix = random_rotation_matrix::<Fr, _>(&mut rng);
    let rot_slice = rotation_matrix.as_slice();

    for (row_idx, row) in commitments.iter().enumerate().take(3) {
        println!("row {row_idx} first four entries: {:?}", &row[..4]);
    }
    println!(
        "challenge rotation first row first four: {:?}",
        &rotation_matrix[0][..4]
    );

    let rot_identity = check_rotation_identity::<Fr, _>(&mut rng);
    println!("rot(a)*cf(b) == cf(a*b)? {rot_identity}");

    let linearity_holds = verify_challenge_linearity(&scheme, rot_slice, &witness);
    println!("challenge * commit(z) == commit(challenge * z)? {linearity_holds}");
}
