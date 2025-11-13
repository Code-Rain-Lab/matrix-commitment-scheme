use ark_ff::{Field, PrimeField};
use ark_std::test_rng;
use itertools::Itertools;

use crate::{
    D, K, LOG_D, LOG_DN, LOG_N, M, MatrixCommitmentScheme, Rq, T,
    almost_goldilock::{Fq, Fq2},
};

pub struct MCS<F: PrimeField> {
    c: Vec<Rq<F>>,
    x: Vec<F>,
    w: Vec<F>,
}

pub struct ME<F: PrimeField> {
    c: Vec<Rq<F>>,
    z: Vec<Rq<F>>,
    r: Vec<Fq2<F>>,      // the size is log N, N is the number of constraints
    y: Vec<[Fq2<F>; D]>, // the size is t, which is number of CCS matrix
}

pub struct Reduction<F: PrimeField> {
    ccs_matrix: [Vec<Vec<F>>; T],
    ccs_f: fn(&[Fq2<F>]) -> Fq2<F>,
}

impl<F: PrimeField> Reduction<F> {
    pub fn ccs_reduction(&self, mcs: MCS<F>, me: Vec<ME<F>>) {
        assert!(me.len() == K - 1);

        let mut rng = test_rng();

        // setup
        let z_1 = [vec![F::ONE], mcs.x, mcs.w].concat();
        //todo:  拡大体じゃないといけないっぽい
        let alpha: Vec<Fq2<F>> = (0..LOG_D)
            .map(|_| Fq2::<F>::new(F::rand(&mut rng), F::rand(&mut rng)))
            .collect();
        let beta: Vec<Fq2<F>> = (0..LOG_DN)
            .map(|_| Fq2::<F>::new(F::rand(&mut rng), F::rand(&mut rng)))
            .collect();
        let gamma: Fq2<F> = Fq2::<F>::new(F::rand(&mut rng), F::rand(&mut rng));
        let r = me[0].r.clone();

        // ccs_matrixとz_1とのMELのclosureを定義する。
        let poly_f = self.poly_f(&z_1);

        // ZのMLEを作る。
        // n == m を仮定して良いらしい。つまり、制約数と変数の数が同じになって、Mが正方行列
        let z = MatrixCommitmentScheme::bit_decompose_witness(&z_1);
        let z: Vec<&[Rq<F>]> = std::iter::once(z.as_slice())
            .chain(me.iter().map(|me| me.z.as_slice()))
            .collect();

        let poly_nc = self.poly_nc(&z);
        let zm = self.zm(&z);
        let poly_eval = self.poly_eval(&alpha, &r, &zm);

        // Q(X): eq(X, β) * (F(X[log_dn+1..]) + Σ γ^i+1 * nc_i(X)) + Σ γ^i+k+1.. * eval_i(X)
        let mut gamma_pow_of_i = vec![gamma];
        for i in 1..K * T {
            gamma_pow_of_i[i] = gamma_pow_of_i[i - 1] * gamma;
        }
        let poly_q = |x: &[Fq2<F>]| {
            eq(x, &beta)
                * (poly_f(&x[LOG_D..])
                    + poly_nc
                        .iter()
                        .enumerate()
                        .map(|(i, nc_i)| gamma_pow_of_i[i + 1] * nc_i(x))
                        .sum::<Fq2<F>>())
                + poly_eval
                    .iter()
                    .flatten()
                    .enumerate()
                    .map(|(i, eval_i)| gamma_pow_of_i[K + i + 1] * eval_i(x))
                    .sum::<Fq2<F>>()
        };

        // T =
        let poly_y: Vec<Vec<_>> = me
            .iter()
            .map(|me| {
                me.y.iter()
                    .map(|y| mle_vector(y.to_vec()))
                    .collect::<Vec<_>>()
            })
            .collect();
        let t = poly_y
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, y_i_j)| gamma_pow_of_i[K + i + 1] * y_i_j(&alpha))
            .sum::<Fq2<F>>();

        // T == Qの{0,1}^log_dn, n==m in this setting
        // assert_eq!(t, all_bool_patterns(LOG_DN).map(|x| poly_q(&x)).sum::<F>());

        // sum-check

        let alpha_: Vec<Fq2<F>> = (0..LOG_D)
            .map(|_| Fq2::<F>::new(F::rand(&mut rng), F::rand(&mut rng)))
            .collect();
        let r_: Vec<Fq2<F>> = (0..LOG_N)
            .map(|_| Fq2::<F>::new(F::rand(&mut rng), F::rand(&mut rng)))
            .collect();
        let random_points = [alpha_, r_].concat();
        let s: Vec<_> = (0..LOG_DN)
            .map(|i| {
                let s_i = |x_i: Fq2<F>| {
                    all_bool_patterns::<F>(LOG_DN - i - 1)
                        .map(|rest| {
                            let mut x = Vec::with_capacity(LOG_DN);
                            x.extend_from_slice(&random_points[..i]);
                            x.push(x_i);
                            x.extend(rest);
                            poly_q(&x)
                        })
                        .sum::<Fq2<F>>()
                };

                // 係数を求める。
                coeffs_from_evaluation(s_i(Fq2::<F>::zero()), s_i(Fq2::<F>::one()))
            })
            .collect();

        // ---- Verify sum-check ----
        // prev = Σ_{x∈{0,1}^m} Q(x)
        let mut prev = t;

        for (i, &[a, b]) in s.iter().enumerate() {
            let g = move |x: Fq2<F>| a + b * x;
            // 境界チェック: g(0)+g(1) == prev
            let boundary = g(Fq2::<F>::zero()) + g(Fq2::<F>::one());
            assert_eq!(boundary, prev, "boundary check failed at round {}", i);

            // 次の主張値へ更新: prev = g(r_i)
            let r_i = random_points[i];
            prev = g(r_i);
        }

        // 最終チェック: prev == Q(r_)
        let q_at_random_points = poly_q(&random_points);
        assert_eq!(prev, q_at_random_points, "final check failed");

        // let hat_r_ =
        let y_: Vec<Vec<_>> = zm
            .iter()
            .map(|zm_i| {
                zm_i.iter()
                    .map(|zm_i_j| {
                        // aaa
                    })
                    .collect()
            })
            .collect();

        // todo
        // - Tのチェック
        // - poly_qのsum-check
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

                mle_vector(u)
            })
            .collect::<Vec<_>>();
        // move |x: &[F]| mz[0](x) * mz[1](x) - mz[2](x) // R1CS
        move |x: &[Fq2<F>]| {
            let mz: Vec<_> = mz.iter().map(|mz_i| mz_i(x)).collect();
            (self.ccs_f)(&mz)
        }
    }

    pub fn poly_nc(&self, z: &Vec<&[Rq<F>]>) -> Vec<impl Fn(&[Fq2<F>]) -> Fq2<F>> {
        z.iter()
            .map(|z_i| {
                let z_i = z_i
                    .iter()
                    .flat_map(|rq| *rq.coeffs())
                    .map(|v| Fq2::<F>::new(v, F::ZERO))
                    .collect();
                let z_i = mle_vector(z_i);
                let one = Fq2::<F>::one();
                let two = one + one;
                move |x: &[Fq2<F>]| {
                    // b is 2 in the current setting
                    let v = z_i(x);
                    (v - two) * (v - one) * v * (v + one) * (v + two)
                }
            })
            .collect()
    }

    pub fn zm(&self, z: &[&[Rq<F>]]) -> Vec<Vec<Vec<Vec<F>>>> {
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

    pub fn poly_eval(
        &self,
        alpha: &Vec<Fq2<F>>,
        r: &Vec<Fq2<F>>,
        zm: &Vec<Vec<Vec<Vec<F>>>>,
        // z: &[&[Rq<F>]],
    ) -> Vec<Vec<impl Fn(&[Fq2<F>]) -> Fq2<F>>> {
        let alpha_and_r: Vec<Fq2<F>> = [alpha.clone(), r.clone()].concat();
        // eval[i][j](x) = eq(x, [alpha||r]) * MLE( Z_i * M_j^T )(x)
        let eval: Vec<Vec<_>> = zm[1..]
            .iter() // z_2_k: Vec<Vec<Rq>> （Rq: .coeffs()->&[F; D]）
            .map(|zm_i| {
                zm_i.iter()
                    .map(|zm_i_j| {
                        let zm_i_j: Vec<_> = zm_i_j
                            .iter()
                            .flatten()
                            .copied()
                            .map(|v| Fq2::<F>::new(v, F::ZERO))
                            .collect();
                        let zm_i_j = mle_vector(zm_i_j);
                        let target = alpha_and_r.clone(); // 各クロージャへムーブ
                        move |x: &[Fq2<F>]| eq(x, &target) * zm_i_j(x)
                    })
                    .collect()
            })
            .collect();
        eval
    }
}

fn mle_vector<F: Field>(vector: Vec<Fq2<F>>) -> impl Fn(&[Fq2<F>]) -> Fq2<F> {
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
    std::iter::repeat([false, true])
        .take(n)
        .multi_cartesian_product()
        .map(|v| {
            v.into_iter()
                .map(|b| F::from(b))
                .map(|v| Fq2::<F>::new(v, F::ZERO))
                .collect()
        })
}

#[inline]
pub fn coeffs_from_evaluation<F: Field>(eval_at_0: Fq2<F>, eval_at_1: Fq2<F>) -> [Fq2<F>; 2] {
    let a = eval_at_0;
    let b = eval_at_1 - eval_at_0;
    [a, b]
}
