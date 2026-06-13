//! The core domain model of the synthetic spectral sequence.
//!
//! - [`e1`]: the E1 page — the fixed set of generators and lookup indices.
//! - [`model`]: [`model::SyntheticSS`], the user-asserted differentials and
//!   tau-multiplications layered on top of an E1 page.
//! - [`process`]: turns a `SyntheticSS` into computed [`ss::SSPages`] by applying
//!   those facts page by page, reporting any [`crate::solve::issues::Issue`]s.
//! - [`ss`]: [`ss::SSPages`], the per-generator state across pages after computation.

pub mod e1;
pub mod model;
pub mod process;
pub mod ss;
