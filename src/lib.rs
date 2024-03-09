mod traits;
mod wrappers;

#[cfg(feature = "tokio")]
pub mod tokio;

pub use traits::*;
pub use wrappers::*;

pub const MAX_CHUNK_SIZE: usize = 0x7ffff0000;
