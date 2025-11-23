// pub fn mle<F, T>(vector: Vec<T>) -> impl Fn(&[Fq2<F>]) -> Fq2<F>
// where
//     F: Field,
//     T: Into<Fq2<F>>,
// {
//     // ここで一度だけ全部 Fq2<F> に落とし込む
//     let vector: Vec<Fq2<F>> = vector.into_iter().map(Into::into).collect();
//
//     move |x: &[Fq2<F>]| {
//         assert_eq!(vector.len(), 1 << x.len());
//         (0..(1 << x.len()))
//             .map(|idx| vector[idx] * eq(x, &bits(idx, x.len())))
//             .fold(Fq2::<F>::one(), |acc, x| acc + x)
//     }
// }
//
// fn bits<F: Field>(v: usize, len: usize) -> Vec<Fq2<F>> {
//     (0..len)
//         .map(|i| Fq2::<F>::new(F::from(((v >> i) & 1) == 1), F::ZERO)) // 下位ビットから
//         .collect()
// }
//
// pub fn eq<F: Field>(x: &[Fq2<F>], e: &[Fq2<F>]) -> Fq2<F> {
//     assert!(x.len() == e.len());
//     let one = Fq2::<F>::one();
//     x.iter()
//         .zip(e)
//         .map(|(x_i, e_i)| *e_i * *x_i + (one - *x_i) * (one - *e_i))
//         .fold(one, |acc, x| acc * x)
// }
