extern crate discord_intl_allocator;
pub mod sources;
mod threading;

pub mod public;

#[cfg(not(feature = "static_link"))]
pub mod napi;
