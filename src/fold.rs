use std::ops::Mul;

use ark_ff::PrimeField;
use ark_test_curves::PrimeGroup;

use crate::{
    LOG_D, LOG_DN, MatrixCommitmentScheme, Rq,
    almost_goldilock::Fq2,
    mat::Mat,
    reduction::{
        Transcript,
        ccs::{eq, mle, powers_of},
    },
};

pub struct Fold<F: PrimeField> {
    scheme: MatrixCommitmentScheme<F>,
}

impl<F: PrimeField> Fold<F> {
    pub fn step() {
        //
    }
}

//   ┌───────┐ Running Inst ┌───────┐
//   │Circuit│─────────────►│Circuit│
//   └───────┘              └───▲───┘
//  └────┬────┘                 │
//       │       i-th Inst      │
//       └──────────────────────┘
//
// Suffix
// _r: running
// _i: i-th
// _n: next
// _b: base
// _f: fold
pub fn augmented_synthesizer<F: PrimeField>() {
    // let step_input_b = init_step_input.alloc();
    // let step_input_i = step_input.alloc();
    // let step_input_n = step_circuit(step_input);

    // let step_count_i = Var::from(step_count);
    // let step_count_n = step_count_i + 1;
    // let is_base_step = is_zero(step_count_i);

    // let instance_r = select(is_base_step, instance_r.alloc(), instance_b);
    // let instance_i = instance_i.alloc();

    // let x_i = hash(step_count_i, step_input_b, step_input_i, instance_r);
    // let x_i = x_i * (1 - is_base_step);
    // x_i.equal(instance_i.x);

    // let instance_f = fold(instance_r, instance_i);

    // let x_n = hash(step_count_n, step_input_b, step_input_n, instance_f);

    // cs.inputize(x_n);

    // let step_count = step_input_n.value();
    // let step_input = step_input_n.value();
    // let instance_r = instance_f.value();
    // let x = x_n.value();
}

pub struct Var<F>(F); // wasekiから持ってくる
pub struct RqVar<F>(F);
pub struct Fq2Var<F>(F);

pub struct IncomingInstance<F: PrimeField> {
    c: Vec<Rq<F>>,
    z: Mat<F>,
    x: Mat<F>,
    lc: (Vec<F>, Vec<F>, Vec<F>),
}
pub struct IncomingInstanceVar<F: PrimeField> {
    c: Vec<RqVar<F>>,
    z: Mat<F>,
    x: Mat<F>,
    lc: (Vec<F>, Vec<F>, Vec<F>),
}

pub struct RunningInstance<F: PrimeField> {
    c: Vec<Vec<Rq<F>>>,
    z: Vec<Mat<F>>,
    x: Vec<Mat<F>>,
    r: Vec<Fq2<F>>,           // the size is log N, N is the number of constraints
    y: Vec<Vec<Vec<Fq2<F>>>>, // the size is t, which is number of CCS matrix
}

pub struct RunningInstanceVar<F: PrimeField> {
    c: Vec<Vec<RqVar<F>>>,
    x: Vec<Vec<RqVar<F>>>,
    z: Vec<Vec<Rq<F>>>,
    r: Vec<Fq2Var<F>>, // the size is log N, N is the number of constraints
    y: Vec<Vec<Vec<Fq2Var<F>>>>, // the size is t, which is number of CCS matrix
}
