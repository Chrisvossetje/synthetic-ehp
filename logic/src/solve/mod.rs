//! The solving engine: verifying a partially-filled spectral sequence against
//! the known answers and searching for the facts that make it consistent.
//!
//! - [`issues`]: the [`issues::Issue`] enum and the comparison routines that
//!   detect when computed pages disagree with the expected convergence.
//! - [`action`]: the [`action::Action`] log entries (add differential / tau /
//!   generator / induced name / revert) and how each is applied to the data.
//! - [`ahss`] / [`ehp`]: per-sequence issue-finding (`find_*_issues`).
//! - [`ehp_ahss`]: relating the unstable EHP sequence to the stable AHSS one.
//! - [`generate`] / [`solve`]: proposing candidate differentials and taus, and
//!   auto-deducing forced solutions to issues.
//! - [`search`]: the parallel speculative branch-and-bound primitives.
//! - [`automated_common`]: logic shared by the two automated solvers.
//! - [`automated_ahss`] / [`automated_ehp`]: the automated solvers that drive
//!   the search to fill in the AHSS and EHP sequences.

pub mod action;
pub mod ahss;
pub mod automated_ahss;
pub mod automated;
pub mod automated_ehp;
pub mod ehp;
pub mod ehp_ahss;
pub mod generate;
pub mod issues;
pub mod search;
pub mod solve;
