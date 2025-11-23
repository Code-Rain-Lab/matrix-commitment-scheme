use std::{iter, ops::Mul};

use ark_ff::{Field, PrimeField};
use ark_std::iterable::Iterable;
use itertools::Itertools;

use crate::{
    D, K, LOG_D, LOG_DN, LOG_N, M, MatrixCommitmentScheme, Rq, T,
    fold::{Fq2Var, Value, Var},
    fq2::Fq2,
    mat::Mat,
    mle::{eq, mle},
    reduction::{Reduction, Transcript},
};

pub fn ccs_reduction<F: PrimeField>(
    z: Vec<Mat<F>>,
    r: Vec<Fq2Var<F>>,
    y: Vec<Vec<Vec<Fq2Var<F>>>>,
    lc: [Vec<F>; 3],
    mt: [Mat<F>; 3],
) {
    // CCS Reduction
    let mut transcript = Transcript::new("somthing seeed");
    let alpha = transcript.get_vec(LOG_D);
    let beta = transcript.get_vec(LOG_DN);
    let gamma = transcript.get();
    let alpha_and_r = [alpha.clone(), r.clone()].concat();

    let zmt: Vec<Vec<_>> = z
        .iter()
        .map(|z_i| mt.iter().map(|mt_j| z_i * mt_j).collect())
        .collect();
    let lc: Vec<_> = lc.into_iter().map(mle).collect();
    // poly
    let poly_f = |x: &[Fq2<F>]| lc[0](x) * lc[1](x) - lc[2](x);
    let poly_nc: Vec<_> = z
        .iter()
        .map(|z_i| {
            |x: &[Fq2<F>]| {
                let v = mle(z_i.flatten())(x);
                (v - Fq2::two()) * (v - Fq2::one()) * v * (v + Fq2::one()) * (v + Fq2::two())
            }
        })
        .collect();
    let poly_eval: Vec<_> = zmt[1..]
        .iter()
        .flatten()
        .map(|zmt_i_j| |x: &[Fq2<F>]| eq(x, &alpha_and_r.value()) * mle(zmt_i_j.flatten())(x))
        .collect();
    let poly_q = |x: &[Fq2<F>]| {
        let mut pow = powers_of(gamma.value());
        eq(x, &beta.value())
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

    let t: Fq2Var<F> = {
        let mut pow = powers_of(gamma);
        (0..K).zip(pow.by_ref()); // 必要ない分を消費する
        // y.iter().flatten().zip(pow.by_ref()).map(|(y_i_j, pow)| pow * ).sum()
        Fq2Var::default() // コンパイル通すためのダミー
        // Varに対応したmleを作らないといけない。
    };

    // Sumcheck
    let mut challenges: Vec<Fq2Var<F>> = vec![];
    let mut expected_values = vec![t];
    for i in 0..LOG_DN {
        let (eval_at_0, eval_at_1) = all_bool_patterns::<F>(LOG_DN - i - 1)
            .map(|rest| {
                let x_0 = [challenges.value(), vec![Fq2::zero()], rest.clone()].concat();
                let x_1 = [challenges.value(), vec![Fq2::one()], rest.clone()].concat();
                (poly_q(&x_0), poly_q(&x_1))
            })
            .fold((Fq2::zero(), Fq2::zero()), |acc, x| {
                (acc.0 + x.0, acc.1 + x.1)
            });
        let (a, b) = coeffs_from_evaluation(eval_at_0, eval_at_1);

        // ここでverifyする。
        let s = |x: Fq2Var<F>| x * a + b;
        let zero = Fq2Var::<F>::zero();
        let one = Fq2Var::<F>::one();
        expected_values[i].equal(s(zero) + s(one));

        let challenge = Fq2Var::default(); // ダミー
        expected_values.push(s(challenge));
        challenges.push(challenge);
    }

    let alpha_prime = challenges[..LOG_D].to_vec();
    let r_prime = challenges[LOG_D..].to_vec();
    let r_hat_prime = r_hat(&r_prime.value());

    let y_prime: Vec<Vec<_>> = zmt
        .iter()
        .map(|zmt_i| zmt_i.iter().map(|zmt_i_j| zmt_i_j * &r_hat_prime).collect())
        .collect();

    // v を y_primeから復元できるかを確かめる
    let v = expected_values.pop().unwrap();

    // Fq2のジェネリクスがVarを受け付けるようにしたい。mleをVarに対応させる
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

pub fn powers_of<T>(gamma: T) -> impl Iterator<Item = T>
where
    T: Copy + Mul<Output = T>,
{
    iter::successors(Some(gamma), move |p| Some(*p * gamma))
}

fn powers<F: Field>(b: Fq2<F>, n: usize) -> Vec<Fq2<F>> {
    let mut pow_of_i = vec![Fq2::<F>::one()];
    for i in 1..n {
        pow_of_i[i] = pow_of_i[i - 1] * b;
    }
    pow_of_i
}

#[inline]
pub fn coeffs_from_evaluation<F: Field>(eval_at_0: Fq2<F>, eval_at_1: Fq2<F>) -> (Fq2<F>, Fq2<F>) {
    let a = eval_at_1 - eval_at_0;
    let b = eval_at_0;
    (a, b)
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
