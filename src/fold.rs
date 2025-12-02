use std::{
    marker::PhantomData,
    ops::{Add, Mul, Sub},
};

use ark_ff::PrimeField;
use waseki::Var;

use crate::{fq2::Fq2, matrix::Matrix, vector::Vector};

pub struct Fold<F: PrimeField> {
    a: PhantomData<F>, // scheme: MatrixCommitmentScheme<F>,
}

impl<F: PrimeField> Fold<F> {
    pub fn step() {
        //
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
    pub fn augmented_synthesizer() {
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
}

impl<F: PrimeField> Add<F> for Fq2<F> {
    type Output = Self;

    fn add(self, rhs: F) -> Self::Output {
        todo!()
    }
}
impl<F: PrimeField> Sub<F> for Fq2<F> {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        todo!()
    }
}
impl<F: PrimeField> Sub<F> for Fq2<Var<F>> {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        todo!()
    }
}

impl<F: PrimeField> Mul<Fq2<F>> for Fq2<Var<F>> {
    type Output = Self;

    fn mul(self, rhs: Fq2<F>) -> Self::Output {
        todo!()
    }
}

impl<F: PrimeField> Add<Fq2<F>> for Fq2<Var<F>> {
    type Output = Self;

    fn add(self, rhs: Fq2<F>) -> Self::Output {
        todo!()
    }
}

pub fn alloc_fq2_vec<F: PrimeField>(vec: Vec<Fq2<F>>) -> Vec<Fq2<Var<F>>> {
    todo!()
}

pub trait CircuitVariable<T> {
    fn value(&self) -> T;
    fn equal(&self, rhs: Self);
}

impl<F: PrimeField> CircuitVariable<Fq2<F>> for Fq2<Var<F>> {
    fn value(&self) -> Fq2<F> {
        todo!()
    }
    fn equal(&self, rhs: Self) {
        todo!()
    }
}

impl<F: PrimeField> CircuitVariable<Vector<Fq2<F>>> for &Vector<Fq2<Var<F>>> {
    fn value(&self) -> Vector<Fq2<F>> {
        todo!()
    }
    fn equal(&self, rhs: Self) {
        todo!()
    }
}

impl<F: PrimeField> CircuitVariable<Vec<Fq2<F>>> for Vec<Fq2<Var<F>>> {
    fn value(&self) -> Vec<Fq2<F>> {
        todo!()
    }
    fn equal(&self, rhs: Self) {
        todo!()
    }
}

impl<F: PrimeField> CircuitVariable<Vec<Fq2<F>>> for &Vec<Fq2<Var<F>>> {
    fn value(&self) -> Vec<Fq2<F>> {
        todo!()
    }
    fn equal(&self, rhs: Self) {
        todo!()
    }
}

impl<F: PrimeField> CircuitVariable<Vec<F>> for Vec<Var<F>> {
    fn value(&self) -> Vec<F> {
        todo!()
    }
    fn equal(&self, rhs: Self) {
        todo!()
    }
}
