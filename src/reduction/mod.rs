use ark_ff::{Field, PrimeField};
use itertools::Itertools;

use crate::{D, K, MatrixCommitmentScheme, Rq, T};

pub struct MCS<F: PrimeField> {
    c: Vec<Rq<F>>,
    x: Vec<F>,
    w: Vec<F>,
}

pub struct ME<F: PrimeField> {
    c: Vec<Rq<F>>,
    z: Vec<Rq<F>>,
    r: Vec<bool>,      // the size is log N, N is the number of constraints
    y: Vec<[bool; D]>, // the size is t, which is number of CCS matrix
}

pub struct Reduction<F: PrimeField> {
    ccs_matrix: [Vec<Vec<F>>; T],
}

impl<F: PrimeField> Reduction<F> {
    pub fn ccs_reduction(&self, mcs: MCS<F>, me: Vec<ME<F>>) {
        assert!(me.len() == K - 1);

        // setup
        let z_1 = [vec![F::ONE], mcs.x, mcs.w].concat();
        let alpha: Vec<bool> = vec![]; // size log_d bool random vector
        let beta: Vec<bool> = vec![]; // size log_dn bool random vector
        let gamma: bool = true; // random 0 or 1

        // ccs_matrixとz_1とのMELのclosureを定義する。
        // Σ M(x,y)*z(y)
        let mz = self
            .ccs_matrix
            .iter()
            .map(|matrix| {
                let u: Vec<F> = matrix
                    .iter() // &Vec<Vec<F>> → &Vec<F>
                    .map(|row| {
                        // 行と z の内積
                        row.iter()
                            .copied() // &F → F
                            .zip(z_1.iter().copied()) // &F → F
                            .fold(F::ZERO, |acc, (a_ij, z_j)| acc + a_ij * z_j)
                    })
                    .collect();

                mle_vector(u)
            })
            .collect::<Vec<_>>();
        let f = move |x: &[F]| mz[0](x) * mz[1](x) - mz[2](x); // R1CS

        // ZのMLEを作る。
        // n == m を仮定して良いらしい。つまり、制約数と変数の数が同じになって、Mが正方行列
        let z_1 = MatrixCommitmentScheme::bit_decompose_witness(&z_1);
        let nc = std::iter::once(z_1.as_slice())
            .chain(me.iter().map(|me| me.z.as_slice()))
            .map(|z_i| {
                let z_i = z_i.iter().flat_map(|rq| *rq.coeffs()).collect();
                let z_i = mle_vector(z_i);
                move |x: &[F]| {
                    let v = z_i(x);
                    let zero = F::ZERO;
                    let one = F::ONE;
                    let two = F::from(2);
                    // b is 2 in the current setting
                    (v - two) * (v - one) * (v + zero) * (v + one) * (v + two)
                }
            });
    }
}

fn mle_vector<F: Field>(vector: Vec<F>) -> impl Fn(&[F]) -> F {
    move |x: &[F]| {
        assert_eq!(vector.len(), 1 << x.len());
        (0..1 << x.len())
            .map(|idx| vector[idx] * eq(x, &bits(idx, x.len())))
            .fold(F::ZERO, |acc, x| acc + x)
    }
}

fn mle_matrix<F: Field>(matrix: Vec<Vec<F>>) -> impl Fn(&[F], &[F]) -> F {
    move |x: &[F], y: &[F]| {
        let rows = 1 << x.len();
        let cols = 1 << y.len();

        assert_eq!(matrix.len(), rows, "row count must be 2^|x|");
        assert!(!matrix.is_empty(), "matrix must be non-empty");
        for r in &matrix {
            assert_eq!(r.len(), cols, "col count must be 2^|y| for every row");
        }

        // ∑_{i,j} A[i][j] · eq(x, i_bits) · eq(y, j_bits)
        (0..rows).fold(F::ZERO, |acc_rows, i| {
            let wx = eq(x, &bits::<F>(i, x.len())); // 行の重み
            let row = &matrix[i];

            let inner = (0..cols).fold(F::ZERO, |acc_cols, j| {
                let wy = eq(y, &bits::<F>(j, y.len())); // 列の重み
                acc_cols + row[j] * wy
            });

            acc_rows + inner * wx
        })
    }
}

fn bits<F: Field>(v: usize, len: usize) -> Vec<F> {
    (0..len)
        .map(|i| F::from(((v >> i) & 1) == 1)) // 下位ビットから
        .collect()
}

fn eq<F: Field>(x: &[F], e: &[F]) -> F {
    assert!(x.len() == e.len());
    x.iter()
        .zip(e)
        .map(|(x_i, e_i)| *e_i * *x_i + (F::ONE - x_i) * (F::ONE - e_i))
        .fold(F::ONE, |acc, x| acc * x)
}

fn bool_patterns(n: usize) -> impl Iterator<Item = Vec<bool>> {
    std::iter::repeat([false, true]) // ① [false,true] という“候補集合”を無限に繰り返すイテレータ
        .take(n) // ② その先頭 n 個だけ取る → 「n つの軸」になる
        .multi_cartesian_product() // ③ 直積をとる → 各軸から1つずつ選ぶ全組合せを生成
        .map(|v| v) // ④ v は Vec<bool>（そのまま返すだけ）
}
