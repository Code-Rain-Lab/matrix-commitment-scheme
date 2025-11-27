use std::{iter, ops::Mul};

use ark_ff::{Field, PrimeField};
use ark_std::iterable::Iterable;
use itertools::Itertools;
use num_traits::{One, Zero};
use waseki::Var;

use crate::{
    D, K, LOG_D, LOG_DN, LOG_N, M, MatrixCommitmentScheme, Rq, T,
    fold::{CircuitVariable, alloc_fq2_vec},
    fq2::Fq2,
    matrix::Matrix,
    mle::{eq, mle},
    reduction::{Reduction, Transcript},
    vector::Vector,
};

type R<F> = Vector<Fq2<Var<F>>>;

type Y<F> = Vec<Vector<Fq2<Var<F>>>>;
type Z<F> = Matrix<F>; // Z^T

pub fn ccs_reduction<F: PrimeField>(
    zt: &[Z<bool>], // Z^T
    r: R<F>,
    y: Vec<Y<F>>,
    lc: [Vec<F>; 4],
    m: [&Matrix<F>; 4], // m_0 は単位行列なので、4つ必要か？
) {
    // CCS Reduction
    let mut transcript = Transcript::new("somthing seeed");
    let alpha = transcript.get_vec(LOG_D);
    let beta = transcript.get_vec(LOG_DN);
    let gamma = transcript.get();
    let alpha_and_r = [alpha.clone(), r.0.clone()].concat();

    // Since ZMᵀ = (MZᵀ)ᵀ, we apply the tranposed Z to M from right.
    // However, from an implementation perspective, we don't transpose the MZᵀ to get the ZMᵀ at this point.
    let mzt: Vec<Vec<Matrix<F>>> = zt
        .iter()
        .map(|zt_i| m.iter().map(|m_j| m_j * zt_i).collect())
        .collect();

    // Sumcheck polynomials
    // M0は単位行列なのでR1CSを適用するのに、最初のやつは無視すればいいのか？
    let mz: Vec<_> = lc.into_iter().map(mle).collect();
    let poly_f = |x: &[Fq2<F>]| mz[1](x) * mz[2](x) - mz[3](x);
    let poly_nc: Vec<_> = zt
        .iter()
        .map(|zt_i| {
            |x: &[Fq2<F>]| {
                let v = mle(zt_i.flatten().into_iter().map(Into::<F>::into).collect())(x);
                (-3..1).map(Into::<F>::into).map(|j| v - j).sum::<Fq2<F>>()
            }
        })
        .collect();
    let poly_eval: Vec<_> = mzt[1..]
        .iter()
        .flatten()
        .map(|mzt_i_j| |x: &[Fq2<F>]| eq(x, &alpha_and_r.value()) * mle(mzt_i_j.flatten())(x))
        .collect();
    let poly_q = |x: &[Fq2<F>]| {
        let mut pow_g = powers_of(gamma.value());
        pow_g.next(); // skip the γ^0
        eq(x, &beta.value())
            * (poly_f(&x[LOG_D..])
                + poly_nc
                    .iter()
                    .zip(pow_g.by_ref())
                    .map(|(nc_i, g_i)| g_i * nc_i(x))
                    .sum::<Fq2<F>>())
            + poly_eval
                .iter()
                .zip(pow_g.by_ref())
                .map(|(eval_i_j, g_i_j)| g_i_j * eval_i_j(x))
                .sum::<Fq2<F>>()
    };

    // Sumcheck
    // T = Σ Q(x) = γ^k Σ γ^i y(α)
    let t: Fq2<Var<F>> = {
        let mut pow = powers_of(gamma);
        pow.by_ref().take(K + 1).for_each(|_| {}); // 必要ない分を消費する. γ^0も含めて.
        y.into_iter()
            .flatten()
            .zip(pow.by_ref())
            .map(|(y_i_j, pow)| pow * mle(y_i_j.0)(&alpha))
            .sum()
    };
    let (v, challenges) = sumcheck(poly_q, t);

    let alpha_prime = challenges[..LOG_D].to_vec();
    let r_prime = challenges[LOG_D..].to_vec();
    let r_hat_prime = Vector(r_hat(&r_prime.value()));

    // y' = ZMᵀr̂' = (MZᵀ)ᵀr̂'
    let y_prime: Vec<Vec<_>> = mzt
        .into_iter()
        .map(|mzt_i| {
            mzt_i
                .into_iter()
                .map(|mzt_i_j| mzt_i_j.transpose() * &r_hat_prime)
                .map(|v| v.alloc()) // y'は回路内でしか使わないので、この時点でallocしても問題ない。
                .collect()
        })
        .collect();

    // v を y_primeから復元できるかを確かめる
    let mut pow = powers_of(Fq2::<Var<F>>::two());
    let m: Vec<Fq2<Var<F>>> = y_prime[0]
        .iter()
        .map(|y_0_j| {
            y_0_j
                .0
                .iter()
                .zip(pow.by_ref())
                .map(|(&y_0_j_l, pow)| pow * y_0_j_l)
                .sum()
        })
        .collect();
    let value_f = m[1] * m[2] - m[3];
    let value_nc: Vec<_> = y_prime
        .iter()
        .map(|y_prime_i| {
            // なぜy'(i,1)だけなのかわからない。
            let v = mle(y_prime_i[0].0.clone())(&alpha_prime);
            (-3..1)
                .map(Into::<F>::into)
                .map(|j| v - j)
                .sum::<Fq2<Var<F>>>()
        })
        .collect();
    let alpha_and_r_prime = challenges;
    let value_eval: Vec<_> = y_prime[1..]
        .iter()
        .flatten()
        .map(|y_prime_i_j| {
            eq(&alpha_and_r_prime, &alpha_and_r) * mle(y_prime_i_j.0.clone())(&alpha_prime)
        })
        .collect();
    let value_q = {
        let mut pow = powers_of(gamma);
        pow.next(); // skip the γ^0
        eq(&alpha_and_r_prime, &beta)
            * (value_f
                + value_nc
                    .iter()
                    .zip(pow.by_ref())
                    .map(|(&nc_i, pow)| pow * nc_i)
                    .sum::<Fq2<Var<F>>>())
            + value_eval
                .iter()
                .zip(pow.by_ref())
                .map(|(&eval_i_j, pow)| pow * eval_i_j)
                .sum::<Fq2<Var<F>>>()
    };
    value_q.equal(v);

    // (r_prime, y_prime)
}

pub fn sumcheck<F: PrimeField>(
    poly_q: impl Fn(&[Fq2<F>]) -> Fq2<F>,
    t: Fq2<Var<F>>,
) -> (Fq2<Var<F>>, Vec<Fq2<Var<F>>>) {
    let mut challenges: Vec<Fq2<Var<F>>> = vec![];
    let mut expected_values = vec![t];
    for i in 0..LOG_DN {
        let (eval_at_0, eval_at_1) = all_bool_patterns::<F>(LOG_DN - i - 1)
            .map(|rest| {
                let x_0 = [challenges.value(), vec![Fq2::zero()], rest.clone()].concat();
                let x_1 = [challenges.value(), vec![Fq2::one()], rest.clone()].concat();
                (poly_q(&x_0), poly_q(&x_1))
            })
            .fold((Fq2::<F>::zero(), Fq2::zero()), |acc, x| {
                (acc.0 + x.0, acc.1 + x.1)
            });
        let (a, b) = coeffs_from_evaluation(eval_at_0, eval_at_1);

        // ここでverifyする。
        let s = |x: Fq2<Var<F>>| x * a + b;
        expected_values[i].equal(s(Fq2::zero()) + s(Fq2::one()));

        let challenge = Fq2::<Var<F>>::one(); // TODO ダミー
        expected_values.push(s(challenge));
        challenges.push(challenge);
    }
    let v = expected_values.pop().unwrap();
    (v, challenges)
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
    T: Copy + Mul<Output = T> + One,
{
    iter::successors(Some(T::one()), move |p| Some(*p * gamma))
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
