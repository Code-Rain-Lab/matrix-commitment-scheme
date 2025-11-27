pub mod ccs;
pub mod decomposition;
pub mod random_linear_combination;
use std::{iter, marker::PhantomData};

use ark_ff::{Field, PrimeField, UniformRand};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use itertools::Itertools;
use waseki::Var;

use crate::{D, K, LOG_D, LOG_DN, LOG_N, M, MatrixCommitmentScheme, Rq, T, fq2::Fq2};

pub struct Reduction<F: PrimeField> {
    ccs_matrix: [Vec<Vec<F>>; T],
    ccs_f: fn(&[Fq2<F>]) -> Fq2<F>,
}

pub struct Transcript<F>
where
    F: Field + UniformRand,
{
    rng: StdRng,
    _f: PhantomData<F>,
}

impl<F> Transcript<F>
where
    F: Field + UniformRand,
{
    pub fn new(seed: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        seed.hash(&mut h);
        let s = h.finish();
        Self {
            rng: StdRng::seed_from_u64(s),
            _f: PhantomData,
        }
    }

    pub fn get(&mut self) -> Fq2<Var<F>> {
        // Fq2::<F>::new(F::rand(&mut self.rng), F::rand(&mut self.rng))
        todo!()
    }

    pub fn get_vec(&mut self, len: usize) -> Vec<Fq2<Var<F>>> {
        // (0..len)
        //     .map(|_| Fq2::<F>::new(F::rand(&mut self.rng), F::rand(&mut self.rng)))
        //     .collect()
        todo!()
    }

    pub fn fill(&mut self, out: &mut [Fq2<F>]) {
        for x in out {
            *x = Fq2::<F>::new(F::rand(&mut self.rng), F::rand(&mut self.rng));
        }
    }
}
