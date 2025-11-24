use core::ops::{Add, Mul, Sub};

use num_traits::{One, Zero};

use crate::fq2::Fq2;

pub fn mle<F, T>(vector: Vec<T>) -> impl Fn(&[Fq2<F>]) -> Fq2<F>
where
    F: Copy + Zero + One + Add<Output = F> + Sub<Output = F> + Mul<Output = F>,
    T: Into<Fq2<F>>,
{
    // ここで一度だけ全部 Fq2<F> に落とし込む
    let vector: Vec<Fq2<F>> = vector.into_iter().map(Into::into).collect();

    move |x: &[Fq2<F>]| {
        assert_eq!(vector.len(), 1 << x.len());
        (0..(1 << x.len()))
            .map(|idx| vector[idx] * eq(x, &bits(idx, x.len())))
            .fold(Fq2::<F>::zero(), |acc, x| acc + x)
    }
}

fn bits<F>(v: usize, len: usize) -> Vec<Fq2<F>>
where
    F: Copy + Zero + One,
{
    // 回路の場合、oneは定数じゃないとwitnessに自由度が生まれてしまう
    let one = F::one();
    let zero = F::zero();
    (0..len)
        .map(|i| {
            let bit = if (v >> i) & 1 == 1 { one } else { zero };
            Fq2::<F>::new(bit, zero) // 下位ビットから
        })
        .collect()
}

pub fn eq<F>(x: &[Fq2<F>], e: &[Fq2<F>]) -> Fq2<F>
where
    F: Copy + Zero + One + Add<Output = F> + Sub<Output = F> + Mul<Output = F>,
{
    assert!(x.len() == e.len());
    let one = Fq2::<F>::one();
    x.iter()
        .zip(e)
        .map(|(x_i, e_i)| *e_i * *x_i + (one - *x_i) * (one - *e_i))
        .fold(one, |acc, x| acc * x)
}
