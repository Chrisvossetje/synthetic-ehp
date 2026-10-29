//! Static input data and the parsers that turn it into domain objects.
//!
//! - [`curtis`]: parses the Curtis tables into the algebraic E1 model and the
//!   lazily-initialized `MODEL`/`DATA` statics used throughout the crate.
//! - [`naming`]: string helpers and stable Adams E2 names indexed by `(stem, af)`.
//! - [`r#static`]: comparison data loaded from CSV plus assorted lookup tables.

pub mod r#static;
pub mod curtis;
pub mod naming;
pub mod stable_names;
