use ark_ff::PrimeField;
use core::marker::PhantomData;

use crate::{rq::Rq, D, KAPPA, M};

pub struct MatrixCommitmentScheme<F: PrimeField> {
    seed: String,
    _marker: PhantomData<F>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::UniformRand;
    use ark_std::rand::{SeedableRng, rngs::StdRng};
    use ark_test_curves::bls12_381::Fr;

    fn random_witness(rng: &mut StdRng, len: usize) -> Vec<Rq<Fr>> {
        (0..len)
            .map(|_| {
                let value = Fr::rand(rng);
                Rq::from_field_element(value)
            })
            .collect()
    }

    fn add_rq_vectors(lhs: &[Rq<Fr>], rhs: &[Rq<Fr>]) -> Vec<Rq<Fr>> {
        assert_eq!(lhs.len(), rhs.len());
        lhs.iter()
            .zip(rhs.iter())
            .map(|(a, b)| a.add(b))
            .collect()
    }

    fn add_commitments(lhs: &[Rq<Fr>; KAPPA], rhs: &[Rq<Fr>; KAPPA]) -> [Rq<Fr>; KAPPA] {
        core::array::from_fn(|idx| lhs[idx].add(&rhs[idx]))
    }

    fn apply_challenge_to_commitment(
        challenge: &[[Fr; D]; D],
        commitment: &[Rq<Fr>; KAPPA],
    ) -> [Rq<Fr>; KAPPA] {
        core::array::from_fn(|idx| Rq::apply_rotation(challenge, &commitment[idx]))
    }

    #[test]
    fn commitment_is_additively_homomorphic() {
        let mut rng = StdRng::seed_from_u64(42);
        let scheme = MatrixCommitmentScheme::<Fr>::new("homomorphism_test_seed");
        let witness_len = 32;

        let witness_a = random_witness(&mut rng, witness_len);
        let witness_b = random_witness(&mut rng, witness_len);
        let witness_sum = add_rq_vectors(&witness_a, &witness_b);

        let commitment_a = scheme.commit(&witness_a);
        let commitment_b = scheme.commit(&witness_b);
        let commitment_sum = scheme.commit(&witness_sum);
        let expected_sum = add_commitments(&commitment_a, &commitment_b);

        for (lhs, rhs) in expected_sum.iter().zip(commitment_sum.iter()) {
            assert_eq!(lhs.coeffs(), rhs.coeffs());
        }

        let challenge = MatrixCommitmentScheme::<Fr>::challenge("rotation_challenge_seed");
        let rotated_expected = apply_challenge_to_commitment(&challenge, &expected_sum);
        let rotated_sum = apply_challenge_to_commitment(&challenge, &commitment_sum);

        for (lhs, rhs) in rotated_expected.iter().zip(rotated_sum.iter()) {
            assert_eq!(lhs.coeffs(), rhs.coeffs());
        }
    }
}

impl<F: PrimeField> MatrixCommitmentScheme<F> {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            _marker: PhantomData,
        }
    }

    pub fn commit(&self, z: &[Rq<F>]) -> [Rq<F>; KAPPA] {
        core::array::from_fn(|row| self.commit_row(row, z))
    }

    pub fn bit_decompose_witness(witness: &[F]) -> Vec<Rq<F>> {
        witness
            .iter()
            .map(|value| Rq::from_field_element(*value))
            .collect()
    }

    pub fn challenge(seed: &str) -> [[F; D]; D] {
        Rq::rotation_matrix_from_seed(seed)
    }

    fn commit_row(&self, row: usize, z: &[Rq<F>]) -> Rq<F> {
        z.iter()
            .enumerate()
            .take(M)
            .fold(Rq::zero(), |acc, (column, witness_column)| {
                let matrix_entry = Rq::reject_sampling(&self.seed, row, column);
                let product = matrix_entry.mul(witness_column);
                acc.add(&product)
            })
    }
}
