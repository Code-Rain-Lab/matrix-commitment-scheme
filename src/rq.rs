use ark_ff::{BigInteger, PrimeField};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};

use crate::D;

pub struct Rq<F: PrimeField>([F; D]);

impl<F: PrimeField> Rq<F> {
    /// 畳み込み: self * rhs（mod x^D + 1）
    pub fn mul(&self, rhs: &Self) -> Self {
        let rot = self.rot();
        let mut c: [F; D] = core::array::from_fn(|_| F::zero());

        for i in 0..D {
            let b_i = &rhs.0[i];
            for j in 0..D {
                let mut t = rot[i][j];
                t.mul_assign(b_i);
                c[j].add_assign(&t);
            }
        }
        Self(c)
    }

    pub fn add(&self, rhs: &Self) -> Self {
        // self を土台にして、各要素に rhs を加算
        let mut c = self.0.clone(); // [F; D]: F は Clone（Field は Clone を継承）
        for (ci, b) in c.iter_mut().zip(rhs.0.iter()) {
            ci.add_assign(b); // *ci += *b
        }
        Self(c)
    }
    /// negacyclic 用の循環行列（右回転で折り返した成分はマイナス）
    /// 行 i は x^i * a (mod x^D + 1) に対応
    pub fn rot(&self) -> [[F; D]; D] {
        core::array::from_fn(|i| {
            let i = i % D;
            core::array::from_fn(|j| {
                // 右回転の元インデックス
                let idx = (j + D - i) % D;
                let mut v = self.0[idx];
                // 折り返し位置（j < i）だけ符号反転
                if j < i {
                    v = -v;
                }
                v
            })
        })
    }

    pub fn from_zq(zq: F) -> Self {
        let bigint = zq.into_bigint();
        let mut c: [F; D] = core::array::from_fn(|_| F::zero());

        for (i, c) in c.iter_mut().enumerate() {
            if bigint.get_bit(i) {
                *c = F::ONE;
            }
        }

        Self(c)
    }

    pub fn reject_sampling(seed: &str, row: usize, column: usize) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        hasher.update(row.to_le_bytes());
        hasher.update(column.to_le_bytes());
        let digest = hasher.finalize();
        let mut rng_seed = [0u8; 32];
        rng_seed.copy_from_slice(&digest);
        let mut rng = StdRng::from_seed(rng_seed);

        let mut column_vec = [F::ZERO; D];
        for element in column_vec.iter_mut() {
            // Reject zero samples to avoid degenerate columns.
            let sampled = loop {
                let candidate = F::rand(&mut rng);
                if !candidate.is_zero() {
                    break candidate;
                }
            };
            *element = sampled;
        }
        Self(column_vec)
    }
}
