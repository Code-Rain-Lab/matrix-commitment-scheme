use std::iter;

use ark_ff::{Field, PrimeField};
use itertools::Itertools;

use crate::{
    D, K, LOG_D, LOG_DN, LOG_N, M, MatrixCommitmentScheme, Rq, T,
    almost_goldilock::{Fq, Fq2},
    reduction::{Reduction, Transcript},
};

pub struct MCS<F: PrimeField> {
    c: Vec<Rq<F>>,
    x: Vec<F>,
    w: Vec<F>,
}

pub struct ME<F: PrimeField> {
    c: Vec<Vec<Rq<F>>>,
    z: Vec<Vec<Rq<F>>>,
    r: Vec<Fq2<F>>,           // the size is log N, N is the number of constraints
    y: Vec<Vec<Vec<Fq2<F>>>>, // the size is t, which is number of CCS matrix
}

impl<F: PrimeField> Reduction<F> {
    pub fn prove_ccs_reduction(&self, mcs: MCS<F>, me: ME<F>) -> (ME<F>, Vec<[Fq2<F>; 2]>) {
        let ME {
            z: z_1_to_k,
            c: c_1_to_k,
            y: _y_1_to_k,
            r,
        } = me;

        // setup
        let mut transcript = Transcript::new("somthing seeed");
        let alpha = transcript.get_vec(LOG_D);
        let beta = transcript.get_vec(LOG_DN);
        let gamma = transcript.get();
        let sumcheck_challenge: Vec<Fq2<F>> = [alpha.clone(), r.clone()].concat();

        let w = [vec![F::ONE], mcs.x, mcs.w].concat();

        let z_0 = MatrixCommitmentScheme::bit_decompose_witness(&w);
        let z_all = [vec![z_0], z_1_to_k].concat();
        let c_all = [vec![mcs.c], c_1_to_k].concat();
        let zm_all = self.mul_transposed_m(&z_all);

        // n == m を仮定して良いらしい。つまり、制約数と変数の数が同じになって、Mが正方行列
        let poly_f = self.poly_f(&w);
        let poly_nc: Vec<_> = z_all.iter().map(|z_i| self.poly_nc_i(z_i)).collect();
        let poly_eval: Vec<_> = zm_all[1..]
            .iter()
            .flat_map(|zm_i| {
                zm_i.iter()
                    .map(|zm_i_j| self.poly_eval_i_j(zm_i_j, &sumcheck_challenge))
            })
            .collect();

        // Q(X): eq(X, β) * {F(X[log_dn+1..]) + Σ γ^i+1 * nc_i(X)} + Σ γ^i+k+1.. * eval_i_j(X)
        let poly_q = |x: &[Fq2<F>]| {
            let mut pow = powers_of(gamma);
            pow.next(); // γ^0は使わない。
            eq(x, &beta)
                * (poly_f(&x[LOG_D..])
                    + poly_nc
                        .iter()
                        .zip(pow.by_ref())
                        .map(|(nc_i, pow)| pow * nc_i(x))
                        .sum())
                + poly_eval
                    .iter()
                    .zip(pow.by_ref())
                    .map(|(eval_i_j, pow)| pow * eval_i_j(x))
                    .sum()
        };

        // SumCheck proof
        let sumcheck_challenge = transcript.get_vec(LOG_DN);
        let s: Vec<_> = (0..LOG_DN)
            .map(|i| {
                let s_i = |x_i| {
                    all_bool_patterns::<F>(LOG_DN - i - 1)
                        .map(|rest| poly_q(&[&sumcheck_challenge[..i], &[x_i], &rest].concat()))
                        .sum()
                };
                coeffs_from_evaluation(s_i(Fq2::<F>::zero()), s_i(Fq2::<F>::one()))
            })
            .collect();

        // y' = ZM^T r^ を求める
        let r_ = &sumcheck_challenge[LOG_D..];
        let r_hat = r_hat(r_);
        let y_all: Vec<Vec<Vec<_>>> = zm_all
            .iter()
            .map(|zm_i| {
                zm_i.iter()
                    .map(|zm_i_j| {
                        // multiply ZM^T * r_hat
                        zm_i_j
                            .iter()
                            .map(|row| row.iter().zip(r_hat.iter()).map(|(&a, &b)| b * a).sum())
                            .collect()
                    })
                    .collect()
            })
            .collect();

        (
            ME {
                c: c_all,
                z: z_all.clone(),
                y: y_all,
                r: r_.to_vec(),
            },
            s,
        )
    }

    pub fn verify(&self, mcs: MCS<F>, me: ME<F>, me_: ME<F>, s: Vec<[Fq2<F>; 2]>) {
        // // Verifierがこれを計算する
        // // T =
        // let gamma_pow_of_i: Vec<Fq2<F>> = vec![]; // todo
        // let poly_y: Vec<Vec<_>> = y_1_to_k
        //     .iter()
        //     .map(|y_i| y_i.iter().map(|y| mle(y.to_vec())).collect::<Vec<_>>())
        //     .collect();
        // let t = poly_y
        //     .iter()
        //     .flatten()
        //     .enumerate()
        //     .map(|(i, y_i_j)| gamma_pow_of_i[K + i + 1] * y_i_j(&alpha))
        //     .sum::<Fq2<F>>();
        //
        // // T == Qの{0,1}^log_dn, n==m in this setting
        // // assert_eq!(t, all_bool_patterns(LOG_DN).map(|x| poly_q(&x)).sum::<F>());
        //
        // // ---- Verify sum-check ----
        // // prev = Σ_{x∈{0,1}^m} Q(x)
        // let mut prev = t;
        //
        // for (i, &[a, b]) in s.iter().enumerate() {
        //     let g = move |x: Fq2<F>| a + b * x;
        //     // 境界チェック: g(0)+g(1) == prev
        //     let boundary = g(Fq2::<F>::zero()) + g(Fq2::<F>::one());
        //     assert_eq!(boundary, prev, "boundary check failed at round {}", i);
        //
        //     // 次の主張値へ更新: prev = g(r_i)
        //     let r_i = sumcheck_challenge[i];
        //     prev = g(r_i);
        // }
        //
        // // 最終チェック: prev == Q(r_)
        // let q_at_random_points = poly_q(&sumcheck_challenge);
        // assert_eq!(prev, q_at_random_points, "final check failed");
        //
        // // Verify
        // let poly_y_: Vec<Vec<_>> = y_
        //     .clone()
        //     .into_iter()
        //     .map(|y_i| y_i.into_iter().map(|y_i_j| mle(y_i_j)).collect())
        //     .collect();
        //
        // let one = Fq2::<F>::one();
        // let b = one + one;
        // let mut b_pow_of_i = [one];
        // for i in 1..D {
        //     b_pow_of_i[i] = b_pow_of_i[i - 1] * b;
        // }
        // let m_0: Vec<_> = y_[0]
        //     .clone()
        //     .into_iter()
        //     .map(|y_0_j| {
        //         y_0_j
        //             .into_iter()
        //             .enumerate()
        //             .map(|(l, y_0_j_l)| b_pow_of_i[l] * y_0_j_l)
        //             .sum::<Fq2<F>>()
        //     })
        //     .collect();
        //
        // let f = (self.ccs_f)(&m_0);

        // todo
        // - r'で、y'などを生成
        // - Vがそれらをチェック
    }

    pub fn poly_f(&self, z: &[F]) -> impl Fn(&[Fq2<F>]) -> Fq2<F> {
        // Σ M(x,y)*z(y)
        let mz = self
            .ccs_matrix
            .iter()
            .map(|matrix| {
                let u: Vec<Fq2<F>> = matrix
                    .iter() // &Vec<Vec<F>> → &Vec<F>
                    .map(|row| {
                        // 行と z の内積
                        row.iter()
                            .copied() // &F → F
                            .zip(z.iter().copied()) // &F → F
                            .fold(F::ZERO, |acc, (a_ij, z_j)| acc + a_ij * z_j)
                    })
                    .map(|v| Fq2::<F>::new(v, F::ZERO))
                    .collect();

                mle(u)
            })
            .collect::<Vec<_>>();
        // move |x: &[F]| mz[0](x) * mz[1](x) - mz[2](x) // R1CS
        move |x: &[Fq2<F>]| {
            let mz: Vec<_> = mz.iter().map(|mz_i| mz_i(x)).collect();
            (self.ccs_f)(&mz)
        }
    }

    pub fn poly_nc_i(&self, z: &[Rq<F>]) -> impl Fn(&[Fq2<F>]) -> Fq2<F> {
        let flat = z
            .iter()
            .flat_map(|rq| rq.coeffs().to_vec())
            .map(|v| v.into())
            .collect();
        let mle = mle(flat);
        let one = Fq2::<F>::one();
        let two = one + one;
        move |x: &[Fq2<F>]| {
            // b is 2 in the current setting
            let v = mle(x);
            (v - two) * (v - one) * v * (v + one) * (v + two)
        }
    }

    pub fn poly_eval_i_j(
        &self,
        matrix: &[Vec<Fq2<F>>],
        e: &[Fq2<F>],
    ) -> impl Fn(&[Fq2<F>]) -> Fq2<F> {
        let vector: Vec<_> = matrix.iter().flatten().cloned().collect();
        let mle = mle(vector);
        move |x: &[Fq2<F>]| eq(x, e) * mle(x)
    }

    pub fn mul_transposed_m(&self, z: &[Vec<Rq<F>>]) -> Vec<Vec<Vec<Vec<Fq2<F>>>>> {
        let zm: Vec<Vec<_>> = z
            .iter()
            .map(|z_i| {
                // z_i: d×m 行列（各行は Rq で、長さ m = D の係数）
                // 行ごとの係数スライスにそろえる
                let rows: Vec<&[F]> = z_i.iter().map(|rq| &rq.coeffs()[..]).collect();
                let d = rows.len();
                let m = rows.first().map(|r| r.len()).unwrap_or(0);

                self.ccs_matrix
                    .iter() // 各 m_j: n×m
                    .map(|m_j| {
                        let n = m_j.len();
                        let mj_m = m_j.first().map(|row| row.len()).unwrap_or(0);
                        assert!(d > 0 && n > 0, "empty matrix");
                        assert_eq!(mj_m, m, "dimension mismatch: m_j cols vs z_i cols");

                        // Z_i * M_j^T
                        let zm: Vec<Vec<_>> = (0..d)
                            .map(|a| {
                                (0..n)
                                    .map(|r| {
                                        (0..m).fold(F::ZERO, |acc, c| acc + rows[a][c] * m_j[r][c])
                                    })
                                    .map(|v| v.into())
                                    .collect()
                            })
                            .collect();
                        zm
                    })
                    .collect()
            })
            .collect();
        zm
    }
}

pub fn mle<F: Field>(vector: Vec<Fq2<F>>) -> impl Fn(&[Fq2<F>]) -> Fq2<F> {
    move |x: &[Fq2<F>]| {
        assert_eq!(vector.len(), 1 << x.len());
        (0..1 << x.len())
            .map(|idx| vector[idx] * eq(x, &bits(idx, x.len())))
            .fold(Fq2::<F>::one(), |acc, x| acc + x)
    }
}

fn bits<F: Field>(v: usize, len: usize) -> Vec<Fq2<F>> {
    (0..len)
        .map(|i| Fq2::<F>::new(F::from(((v >> i) & 1) == 1), F::ZERO)) // 下位ビットから
        .collect()
}

fn eq<F: Field>(x: &[Fq2<F>], e: &[Fq2<F>]) -> Fq2<F> {
    assert!(x.len() == e.len());
    let one = Fq2::<F>::one();
    x.iter()
        .zip(e)
        .map(|(x_i, e_i)| *e_i * *x_i + (one - *x_i) * (one - *e_i))
        .fold(one, |acc, x| acc * x)
}

fn all_bool_patterns<F: Field>(n: usize) -> impl Iterator<Item = Vec<Fq2<F>>> {
    std::iter::repeat_n([false, true], n)
        .multi_cartesian_product()
        .map(|v| {
            v.into_iter()
                .map(|b| F::from(b))
                .map(|v| Fq2::<F>::new(v, F::ZERO))
                .collect()
        })
}

fn powers_of<F: Field>(gamma: Fq2<F>) -> impl Iterator<Item = Fq2<F>> {
    iter::successors(Some(Fq2::<F>::one()), move |p| Some(*p * gamma))
}

fn powers<F: Field>(b: Fq2<F>, n: usize) -> Vec<Fq2<F>> {
    let mut pow_of_i = vec![Fq2::<F>::one()];
    for i in 1..n {
        pow_of_i[i] = pow_of_i[i - 1] * b;
    }
    pow_of_i
}

#[inline]
pub fn coeffs_from_evaluation<F: Field>(eval_at_0: Fq2<F>, eval_at_1: Fq2<F>) -> [Fq2<F>; 2] {
    let a = eval_at_0;
    let b = eval_at_1 - eval_at_0;
    [a, b]
}

/// r_hat(r) = ⊗_{i=1}^m (r_i, 1 - r_i)
/// 戻り値の長さは 2^m（m = r.len()）
pub fn r_hat<F: Field>(r: &[Fq2<F>]) -> Vec<Fq2<F>> {
    let one = Fq2::<F>::one();
    let mut v = vec![one];
    for &t in r {
        let mut next = Vec::with_capacity(v.len() * 2);
        for &x in &v {
            // 0 → r_i, 1 → 1 - r_i の順で Kronecker 積を展開
            next.push(x * t);
            next.push(x * (one - t));
        }
        v = next;
    }
    v
}
