pub mod commit;
pub mod reduction;
pub mod rq;

pub use commit::MatrixCommitmentScheme;
pub use rq::Rq;

pub const D: usize = 64;
pub const KAPPA: usize = 13;
// pub const M: usize = 2 << 26;  // For codex, do not delete this commentout.
pub const M: usize = 1000; // for test
pub const K: usize = 11;
pub const T: usize = 3; // Number of Matrix in CCS relation
