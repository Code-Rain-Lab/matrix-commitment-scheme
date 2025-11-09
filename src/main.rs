use ark_ff::{BigInteger, Field, PrimeField, UniformRand};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};

fn main() {
    println!("Hello, world!");
}

pub fn rot_block<F: Field>(base_column: &[F], use_negacyclic: bool) -> Vec<Vec<F>> {
    let dimension = base_column.len();
    assert!(dimension > 0);
    let mut block = vec![vec![F::ZERO; dimension]; dimension];

    for (row_idx, row) in block.iter_mut().enumerate() {
        for (col_idx, cell) in row.iter_mut().enumerate() {
            let offset =
                ((row_idx as isize - col_idx as isize).rem_euclid(dimension as isize)) as usize;
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

fn derive_block_seed(seed: &str, block_idx: usize) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.update(block_idx.to_le_bytes());
    let digest = hasher.finalize();
    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&digest);
    seed_bytes
}

pub fn commit_matrix<F: Field + UniformRand>(
    rows: usize,
    cols: usize,
    seed: &str,
) -> Vec<Vec<F>> {
    assert!(rows > 0, "matrix height must be non-zero");
    assert!(cols > 0, "matrix width must be non-zero");
    assert!(
        cols % rows == 0,
        "the width must be an integer multiple of the rotation block size"
    );

    let block_count = cols / rows;
    let mut matrix = vec![Vec::with_capacity(cols); rows];

    for block_idx in 0..block_count {
        let mut rng = StdRng::from_seed(derive_block_seed(seed, block_idx));
        let base_column: Vec<F> = (0..rows).map(|_| F::rand(&mut rng)).collect();
        let rotation_block = rot_block(&base_column, true);

        for (row, block_row) in matrix.iter_mut().zip(rotation_block.into_iter()) {
            row.extend(block_row);
        }
    }

    matrix
}

pub fn bit_cols<F: PrimeField>(vectors: Vec<F>) -> Vec<Vec<F>> {
    if vectors.is_empty() {
        return Vec::new();
    }
    let bit_len = F::MODULUS_BIT_SIZE as usize;
    let mut rows = vec![Vec::with_capacity(vectors.len()); bit_len];

    for element in vectors {
        let column_bits = bit_vec(element);
        for (row, bit) in rows.iter_mut().zip(column_bits.into_iter()) {
            row.push(bit);
        }
    }

    rows
}

pub fn bit_vec<F: PrimeField>(element: F) -> Vec<F> {
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

pub fn apply_commit<F: Field>(matrix: Vec<Vec<F>>, vector: Vec<F>) -> Vec<F> {
    assert!(!matrix.is_empty(), "matrix must contain at least one row");
    let width = matrix[0].len();
    assert!(
        width == vector.len(),
        "matrix column count must match vector length"
    );
    for row in &matrix {
        assert!(
            row.len() == width,
            "matrix rows must all have the same length"
        );
    }

    let mut result = Vec::with_capacity(matrix.len());
    for row in &matrix {
        let mut accumulator = F::ZERO;
        for (coef, value) in row.iter().zip(vector.iter()) {
            accumulator += *coef * *value;
        }
        result.push(accumulator);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::AdditiveGroup;
    use ark_test_curves::bls12_381::Fr;

    #[test]
    fn rot_block_modes() {
        let base = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)];
        let cyclic = rot_block(&base, false);
        assert_eq!(
            cyclic,
            vec![
                vec![Fr::from(1u64), Fr::from(3u64), Fr::from(2u64)],
                vec![Fr::from(2u64), Fr::from(1u64), Fr::from(3u64)],
                vec![Fr::from(3u64), Fr::from(2u64), Fr::from(1u64)]
            ]
        );

        let negacyclic = rot_block(&base, true);
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
    fn commit_matrix_is_deterministic() {
        let first = commit_matrix::<Fr>(2, 4, "seed");
        let second = commit_matrix::<Fr>(2, 4, "seed");
        assert_eq!(first, second, "setup must be deterministic");
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].len(), 4);
    }

    #[test]
    fn apply_commit_multiplies_matrix_and_vector() {
        let matrix = vec![
            vec![Fr::from(1u64), Fr::from(2u64)],
            vec![Fr::from(3u64), Fr::from(4u64)],
        ];
        let vector = vec![Fr::from(5u64), Fr::from(6u64)];
        let commitment = apply_commit(matrix, vector);
        assert_eq!(commitment, vec![Fr::from(17u64), Fr::from(39u64)]);
    }

    #[test]
    fn bit_vec_matches_binary_expansion() {
        let bits = bit_vec(Fr::from(5u64));
        assert_eq!(bits[0], Fr::ONE); // 1
        assert_eq!(bits[1], Fr::ZERO); // 0
        assert_eq!(bits[2], Fr::ONE); // 1
    }

    #[test]
    fn bit_cols_stack_columns() {
        let rows = bit_cols(vec![Fr::from(1u64), Fr::from(2u64)]);
        assert_eq!(rows[0][0], Fr::ONE); // bit 0 of 1
        assert_eq!(rows[0][1], Fr::ZERO); // bit 0 of 2
        assert_eq!(rows[1][0], Fr::ZERO); // bit 1 of 1
        assert_eq!(rows[1][1], Fr::ONE); // bit 1 of 2
    }
}
