// Re-export the procedural macros
pub use tomplate_macros::tomplate;
pub use tomplate_macros::tomplate_eager;

// Re-export builder utilities for use in build scripts
#[cfg(feature = "build")]
pub use tomplate_build::Builder;

// Re-export types for convenience
#[cfg(feature = "build")]
pub use tomplate_build::types::{Template, Error, Result};