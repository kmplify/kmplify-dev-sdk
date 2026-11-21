//! KMPLIFY developer SDK, command-line side: the `kmplify.app/v1` manifest,
//! developer keys and the signed envelope. `main.rs` is a thin shell over
//! this library so the same rules can be tested and reused.

pub mod manifest;
pub mod scaffold;
pub mod signing;
