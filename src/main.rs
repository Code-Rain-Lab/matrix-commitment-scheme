use ark_ff::{BigInteger, Field, PrimeField, UniformRand};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};

fn main() {
    println!("Hello, world!");
}

pub fn rotation_matrix<F: Field>(c0: &[F], negacyclic: bool) -> Vec<Vec<F>> {
    let n = c0.len();
    assert!(n > 0);
    let mut m = vec![vec![F::ZERO; n]; n];

    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            let idx = ((i as isize - j as isize).rem_euclid(n as isize)) as usize;
            let sign = if negacyclic && i < j { -F::ONE } else { F::ONE };
            *cell = sign * c0[idx];
        }
    }
    m
}

fn derive_block_seed(seed: &str, block_idx: usize) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.update(block_idx.to_le_bytes());
    let digest = hasher.finalize();
    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&digest);
    seed_bytes
}

pub fn setup<F: Field + UniformRand>(n: usize, m: usize, seed: &str) -> Vec<Vec<F>> {
    assert!(n > 0, "matrix height must be non-zero");
    assert!(m > 0, "matrix width must be non-zero");
    assert!(
        m % n == 0,
        "the width must be an integer multiple of the rotation block size"
    );

    let blocks = m / n;
    let mut matrix = vec![Vec::with_capacity(m); n];

    for block_idx in 0..blocks {
        let mut rng = StdRng::from_seed(derive_block_seed(seed, block_idx));
        let base: Vec<F> = (0..n).map(|_| F::rand(&mut rng)).collect();
        let rotation_block = rotation_matrix(&base, true);

        for (row, block_row) in matrix.iter_mut().zip(rotation_block.into_iter()) {
            row.extend(block_row);
        }
    }

    matrix
}

pub fn decompose<F: PrimeField>(z: Vec<F>) -> Vec<Vec<F>> {
    if z.is_empty() {
        return Vec::new();
    }
    let bit_len = F::MODULUS_BIT_SIZE as usize;
    let mut rows = vec![Vec::with_capacity(z.len()); bit_len];

    for elem in z {
        let column_bits = bit_decompose_vector(elem);
        for (row, bit) in rows.iter_mut().zip(column_bits.into_iter()) {
            row.push(bit);
        }
    }

    rows
}

pub fn bit_decompose_vector<F: PrimeField>(element: F) -> Vec<F> {
    let bit_len = F::MODULUS_BIT_SIZE as usize;
    let bigint = element.into_bigint();
    (0..bit_len)
        .map(|bit_idx| {
            if bigint.get_bit(bit_idx) {
                F::ONE
            } else {
                F::ZERO
            }
        })
        .collect()
}

pub fn commit<F: Field>(a: Vec<Vec<F>>, z: Vec<F>) -> Vec<F> {
    assert!(!a.is_empty(), "matrix must contain at least one row");
    let width = a[0].len();
    assert!(
        width == z.len(),
        "matrix column count must match vector length"
    );
    for row in &a {
        assert!(
            row.len() == width,
            "matrix rows must all have the same length"
        );
    }

    let mut result = Vec::with_capacity(a.len());
    for row in &a {
        let mut acc = F::ZERO;
        for (coef, z_val) in row.iter().zip(z.iter()) {
            acc += *coef * *z_val;
        }
        result.push(acc);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::AdditiveGroup;
    use ark_test_curves::bls12_381::Fr;

    #[test]
    fn rotation_matrix_cyclic_and_negacyclic() {
        let base = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)];
        let cyclic = rotation_matrix(&base, false);
        assert_eq!(
            cyclic,
            vec![
                vec![Fr::from(1u64), Fr::from(3u64), Fr::from(2u64)],
                vec![Fr::from(2u64), Fr::from(1u64), Fr::from(3u64)],
                vec![Fr::from(3u64), Fr::from(2u64), Fr::from(1u64)]
            ]
        );

        let negacyclic = rotation_matrix(&base, true);
        assert_eq!(
            negacyclic,
            vec![
                vec![Fr::from(1u64), -Fr::from(3u64), -Fr::from(2u64)],
                vec![Fr::from(2u64), Fr::from(1u64), -Fr::from(3u64)],
                vec![Fr::from(3u64), Fr::from(2u64), Fr::from(1u64)]
            ]
        );
    }

    #[test]
    fn setup_builds_expected_dimensions_and_is_deterministic() {
        let first = setup::<Fr>(2, 4, "seed");
        let second = setup::<Fr>(2, 4, "seed");
        assert_eq!(first, second, "setup must be deterministic");
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].len(), 4);
    }

    #[test]
    fn commit_multiplies_matrix_and_vector() {
        let matrix = vec![
            vec![Fr::from(1u64), Fr::from(2u64)],
            vec![Fr::from(3u64), Fr::from(4u64)],
        ];
        let vector = vec![Fr::from(5u64), Fr::from(6u64)];
        let commitment = commit(matrix, vector);
        assert_eq!(commitment, vec![Fr::from(17u64), Fr::from(39u64)]);
    }

    #[test]
    fn bit_decompose_vector_matches_binary_expansion() {
        let bits = bit_decompose_vector(Fr::from(5u64));
        assert_eq!(bits[0], Fr::ONE); // 1
        assert_eq!(bits[1], Fr::ZERO); // 0
        assert_eq!(bits[2], Fr::ONE); // 1
    }

    #[test]
    fn decompose_stacks_bit_columns() {
        let rows = decompose(vec![Fr::from(1u64), Fr::from(2u64)]);
        assert_eq!(rows[0][0], Fr::ONE); // bit 0 of 1
        assert_eq!(rows[0][1], Fr::ZERO); // bit 0 of 2
        assert_eq!(rows[1][0], Fr::ZERO); // bit 1 of 1
        assert_eq!(rows[1][1], Fr::ONE); // bit 1 of 2
    }
}
