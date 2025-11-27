pub mod commit;
pub mod fold;
pub mod fq;
pub mod fq2;
// pub mod mat;
pub mod matrix;
pub mod mle;
pub mod reduction;
pub mod rq;
pub mod utils;
pub mod vector;

pub use commit::MatrixCommitmentScheme;
pub use rq::Rq;

pub const D: usize = 64;
pub const KAPPA: usize = 13;
// pub const M: usize = 2 << 26;  // For codex, do not delete this commentout.
pub const M: usize = 1000; // for test
pub const N: usize = M;
pub const K: usize = 11;
pub const T: usize = 3; // Number of Matrix in CCS relation

pub const LOG_D: usize = D.ilog2() as usize;
pub const LOG_DN: usize = (D * N).ilog2() as usize; // M==N
pub const LOG_N: usize = N.ilog2() as usize; // M==N
