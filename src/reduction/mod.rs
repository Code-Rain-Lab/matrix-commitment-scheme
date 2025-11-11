use ark_ff::{Field, PrimeField};
use itertools::Itertools;

use crate::{D, K, M, MatrixCommitmentScheme, Rq, T};

pub struct MCS<F: PrimeField> {
    c: Vec<Rq<F>>,
    x: Vec<F>,
    w: Vec<F>,
}

pub struct ME<F: PrimeField> {
    c: Vec<Rq<F>>,
    z: Vec<Rq<F>>,
    r: Vec<F>,      // the size is log N, N is the number of constraints
    y: Vec<[F; D]>, // the size is t, which is number of CCS matrix
}

pub struct Reduction<F: PrimeField> {
    ccs_matrix: [Vec<Vec<F>>; T],
}

impl<F: PrimeField> Reduction<F> {
    pub fn ccs_reduction(&self, mcs: MCS<F>, me: Vec<ME<F>>) {
        assert!(me.len() == K - 1);

        // setup
        let z_1 = [vec![F::ONE], mcs.x, mcs.w].concat();
        // 0,1ではない気がする。
        let alpha: Vec<F> = vec![]; // size log_d bool random vector
        let beta: Vec<F> = vec![]; // size log_dn bool random vector
        let gamma: F = F::from(11111); // random
        let r = me[0].r.clone();

        // ccs_matrixとz_1とのMELのclosureを定義する。
        let poly_f = self.poly_f(&z_1);

        // ZのMLEを作る。
        // n == m を仮定して良いらしい。つまり、制約数と変数の数が同じになって、Mが正方行列
        let z1 = MatrixCommitmentScheme::bit_decompose_witness(&z_1);
        let z: Vec<&[Rq<F>]> = std::iter::once(z1.as_slice())
            .chain(me.iter().map(|me| me.z.as_slice()))
            .collect();

        let poly_nc = self.poly_nc(&z);

        let poly_eval = self.poly_eval(&alpha, &r, &z);

        // Q(X): eq(X, β) * (F(X[log_dn+1..]) + Σ γ^i+1 * nc_i(X)) + Σ γ^i+k+1.. * eval_i(X)
        let log_d = D.ilog2() as usize;
        let poly_q = |x: &[F]| {
            eq(x, &beta)
                * (poly_f(&x[log_d..])
                    + poly_nc
                        .iter()
                        .enumerate()
                        .map(|(i, nc_i)| gamma.pow([(i + 1) as u64]) * nc_i(x))
                        .sum::<F>())
                + poly_eval
                    .iter()
                    .flatten()
                    .enumerate()
                    .map(|(i, eval_i)| gamma.pow([(K + i + 1) as u64]) * eval_i(x))
                    .sum::<F>()
        };

        // Qの{0,1}^log_dn, n==m in this setting
        let log_dn = (D * M).ilog2() as usize;
        let t = all_bool_patterns(log_dn).map(|x| poly_q(&x)).sum::<F>();

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
            .map(|(i, y_i)| gamma.pow([(K + i + 1) as u64]) * y_i(&alpha))
            .sum::<F>();
    }

    pub fn poly_f(&self, z: &[F]) -> impl Fn(&[F]) -> F {
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
                            .zip(z.iter().copied()) // &F → F
                            .fold(F::ZERO, |acc, (a_ij, z_j)| acc + a_ij * z_j)
                    })
                    .collect();

                mle_vector(u)
            })
            .collect::<Vec<_>>();
        move |x: &[F]| mz[0](x) * mz[1](x) - mz[2](x) // R1CS
    }

    pub fn poly_nc(&self, z: &Vec<&[Rq<F>]>) -> Vec<impl Fn(&[F]) -> F> {
        z.iter()
            .map(|z_i| {
                let z_i = z_i.iter().flat_map(|rq| *rq.coeffs()).collect();
                let z_i = mle_vector(z_i);
                let two = F::from(2);
                move |x: &[F]| {
                    // b is 2 in the current setting
                    let v = z_i(x);
                    (v - two) * (v - F::ONE) * v * (v + F::ONE) * (v + two)
                }
            })
            .collect()
    }

    pub fn poly_eval(
        &self,
        alpha: &Vec<F>,
        r: &Vec<F>,
        z: &[&[Rq<F>]],
    ) -> Vec<Vec<impl Fn(&[F]) -> F>> {
        let alpha_and_r: Vec<F> = [alpha.clone(), r.clone()].concat();
        // eval[i][j](x) = eq(x, [alpha||r]) * MLE( Z_i * M_j^T )(x)
        let eval: Vec<Vec<_>> = z[1..]
            .iter() // z_2_k: Vec<Vec<Rq>> （Rq: .coeffs()->&[F; D]）
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
                        let mut zm_flat = Vec::with_capacity(d * n);
                        for a in 0..d {
                            for r in 0..n {
                                let dot =
                                    (0..m).fold(F::ZERO, |acc, c| acc + rows[a][c] * m_j[r][c]);
                                zm_flat.push(dot);
                            }
                        }

                        let zm_mle = mle_vector(zm_flat);
                        let target = alpha_and_r.clone(); // 各クロージャへムーブ

                        move |x: &[F]| eq(x, &target) * zm_mle(x)
                    })
                    .collect()
            })
            .collect();
        eval
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

fn all_bool_patterns<F: Field>(n: usize) -> impl Iterator<Item = Vec<F>> {
    std::iter::repeat([false, true])
        .take(n)
        .multi_cartesian_product()
        .map(|v| v.into_iter().map(|b| F::from(b)).collect())
}
