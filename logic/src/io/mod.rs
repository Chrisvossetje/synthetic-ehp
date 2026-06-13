//! Input/output: the interactive terminal menu (`cli`), serialization of the
//! computed spectral sequence to the website's TypeScript data files (`export`),
//! and loading saved action logs back in (`import`).

pub mod cli;
pub mod export;
pub mod import;