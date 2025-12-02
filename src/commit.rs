use ark_ff::{AdditiveGroup, BigInteger, Field, PrimeField};
use ark_std::rand::{Rng, SeedableRng, rngs::StdRng};
use core::marker::PhantomData;
use sha2::{Digest, Sha256};
use std::ops::{Add, Mul};

use crate::{fq::GLFq, matrix::Matrix, vector::Vector};

pub struct CommitmentScheme<F: PrimeField> {
    seed: String,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> CommitmentScheme<F> {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            _marker: PhantomData,
        }
    }
}

pub trait Commit<F> {
    fn commit(&self, z: &Matrix<bool>) -> Matrix<F>;
}

impl Commit<GLFq> for CommitmentScheme<GLFq> {
    fn commit(&self, z: &Matrix<bool>) -> Matrix<GLFq> {
        const M: usize = 100;
        const KAPPA: usize = 16;
        const D: usize = 54;
        let seed = "aaaaaaa";
        let mat: Vec<Vec<Vector<GLFq>>> = (0..KAPPA)
            .map(|r| {
                (0..M)
                    .map(|c| reject_sampling::<GLFq, D>(seed, r, c))
                    .collect()
            })
            .collect();
        let vec: Vec<Vector<bool>> = z.rows();

        // mat * vec
        let vec: Vec<_> = mat
            .iter()
            .map(|row| {
                row.iter()
                    .zip(vec.iter())
                    .fold(Vector(vec![GLFq::ZERO; D]), |acc, (a_ij, x_j)| {
                        acc + (a_ij.rot() * x_j)
                    })
            })
            .collect();
        Matrix(vec)
    }
}

impl<F: Field> Mul<&Vector<bool>> for Matrix<F> {
    type Output = Vector<F>;

    fn mul(self, rhs: &Vector<bool>) -> Self::Output {
        let n_cols = self.0[0].0.len();
        let n_rows = self.0.len();

        // 列数と rhs の長さをチェック
        assert_eq!(
            n_cols,
            rhs.0.len(),
            "matrix columns ({}) and vector length ({}) mismatch",
            n_cols,
            rhs.0.len()
        );

        // 結果ベクトル: 各行に対応する要素
        let mut result = vec![F::ZERO; n_rows];

        // rhs_j が true の列 j だけを加算する
        for (j, &b) in rhs.0.iter().enumerate() {
            if !b {
                continue; // 0 なので無視
            }
            // 列 j を結果に足す
            for (i, row) in self.0.iter().enumerate() {
                result[i] += row.0[j]; // 1 * a_{i,j} = a_{i,j}
            }
        }

        Vector(result)
    }
}

// impl<F: Field> Mul<Vector<F>> for &Matrix<F> {
//     type Output = Vector<F>;
//
//     fn mul(self, rhs: Vector<F>) -> Self::Output {
//         todo!()
//     }
// }

// impl<F: Field> Add<Matrix<F>> for Matrix<bool> {
//     type Output = Matrix<F>;
//
//     fn add(self, rhs: Matrix<F>) -> Self::Output {
//         todo!()
//     }
// }

impl Vector<GLFq> {
    pub fn rot(&self) -> Matrix<GLFq> {
        const D: usize = 54;

        assert_eq!(self.0.len(), D);

        // Φη = X^54 + X^27 + 1 の -c ベクトルを作る
        let mut minus_c = vec![GLFq::ZERO; D];
        minus_c[0] = -GLFq::ONE; // -c0 = -1
        minus_c[27] = -GLFq::ONE; // -c27 = -1

        // F を一回作用させる関数: v -> Fv
        fn apply_f(v: &[GLFq], minus_c: &[GLFq]) -> Vec<GLFq> {
            let d = v.len();
            let last = v[d - 1];

            let mut res = vec![GLFq::ZERO; d];

            // [0, a0, ..., a_{d-2}]
            for i in 1..d {
                res[i] = v[i - 1];
            }

            // + last * (-c)
            for i in 0..d {
                res[i] += last * minus_c[i];
            }

            res
        }

        // まず v0 = cf(a)
        let mut col = self.0.clone();

        // 列ベクトルたち v0, Fv0, F^2 v0, ..., F^{d-1}v0 を集める
        let mut cols: Vec<Vec<GLFq>> = Vec::with_capacity(D);
        for _ in 0..D {
            cols.push(col.clone());
            col = apply_f(&col, &minus_c);
        }

        // ここでは Matrix<GLFq> を「行の配列」とみなしているので転置する
        let mut rows: Vec<Vector<GLFq>> = Vec::with_capacity(D);
        for i in 0..D {
            let mut row = Vec::with_capacity(D);
            for j in 0..D {
                row.push(cols[j][i]); // (i 行 j 列) = j 列ベクトルの i 番目
            }
            rows.push(Vector(row));
        }

        Matrix(rows)
    }
}

impl Vector<bool> {
    pub fn from(v: &GLFq) -> Self {
        const D: usize = 54;
        let bigint = v.into_bigint();
        let mut coeffs = [false; D];
        for (i, coeff) in coeffs.iter_mut().enumerate() {
            *coeff = bigint.get_bit(i);
        }
        Vector(coeffs.to_vec())
    }
}

impl Matrix<bool> {
    pub fn from(m: &[GLFq]) -> Self {
        let vec: Vec<_> = m.iter().map(|m| Vector::<bool>::from(m)).collect();
        Matrix(vec)
    }
}

pub fn reject_sampling<F: Field, const D: usize>(
    seed: &str,
    row_idx: usize,
    column_idx: usize,
) -> Vector<F> {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.update(row_idx.to_le_bytes());
    hasher.update(column_idx.to_le_bytes());
    let digest = hasher.finalize();
    let mut rng_seed = [0u8; 32];
    rng_seed.copy_from_slice(&digest);
    let mut rng = StdRng::from_seed(rng_seed);

    let mut column_vec = [F::ZERO; D];
    for element in column_vec.iter_mut() {
        let sampled = loop {
            let candidate = F::rand(&mut rng);
            if !candidate.is_zero() {
                break candidate;
            }
        };
        *element = sampled;
    }
    Vector(column_vec.to_vec())
}

impl Vector<GLFq> {
    pub fn challenge(seed: &str) -> Vector<GLFq> {
        const D: usize = 54;

        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let digest = hasher.finalize();
        let mut rng_seed = [0u8; 32];
        rng_seed.copy_from_slice(&digest);
        let mut rng = StdRng::from_seed(rng_seed);

        let mut column_vec = [GLFq::ZERO; D];
        for coeff in column_vec.iter_mut() {
            let draw = rng.gen_range(0..4);
            *coeff = match draw {
                0 => -GLFq::ONE,
                1 => GLFq::ZERO,
                2 => GLFq::ONE,
                _ => GLFq::from(2u64),
            };
        }
        Vector(column_vec.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use ark_std::test_rng;

    use super::*;

    type Fq = GLFq;

    #[test]
    pub fn random_linear_combination() {
        let mut rng = test_rng();

        let witness_len = 32;
        let a: Vec<Fq> = (0..witness_len).map(|_| Fq::rand(&mut rng)).collect();
        let b: Vec<Fq> = (0..witness_len).map(|_| Fq::rand(&mut rng)).collect();

        let r = Vector::<Fq>::challenge("aaaaaa").rot();

        let a = Matrix::<bool>::from(&a);
        let b = Matrix::<bool>::from(&b);

        let scheme = CommitmentScheme::new("seed");

        let com_a = scheme.commit(&a);
        let com_b = scheme.commit(&b);
        let com_c = com_a + Matrix(com_b.rows().into_iter().map(|rq| &r * rq).collect());

        // let c = a + Matrix(b.rows().into_iter().map(|rq| &r * rq).collect());

        // assert_eq!(com_c, scheme.commit(&c))
    }
}

/*
 * 問題: Matrix<bool>以外にコミットメントすることはないのだが、準同型性の確認のために、チャレンジをかけた結果であるMatrix<F>に対して、コミットメントをしないといけない。
 *
 *
 */
