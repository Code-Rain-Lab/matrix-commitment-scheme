use std::time::Duration;

use ark_ff::UniformRand;
use ark_std::rand::{Rng, SeedableRng, rngs::StdRng};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

use matrix_commitment_scheme::{
    commit::{Commit, CommitmentScheme},
    fq::GLFq,
    matrix::Matrix,
    vector::Vector,
};

fn bench_commit(c: &mut Criterion) {
    const M: usize = 2 << 24;

    let mut rng = StdRng::seed_from_u64(42);
    let z: Vec<GLFq> = (0..M).map(|_| GLFq::rand(&mut rng)).collect();
    let z = Matrix::<bool>::from(&z); // bit-split into bool columns
    let scheme = CommitmentScheme::<GLFq>::new("bench-seed");

    let mut group = c.benchmark_group("commit");
    group
        .warm_up_time(Duration::from_millis(50))
        .measurement_time(Duration::from_millis(300))
        .sample_size(10);

    group.bench_function("commit/Matrix<bool>", |b| {
        b.iter(|| {
            let res = scheme.commit(black_box(&z));
            black_box(res);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_commit);
criterion_main!(benches);
