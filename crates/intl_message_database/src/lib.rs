use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod sources;
mod threading;

pub mod public;

#[cfg(not(feature = "static_link"))]
pub mod napi;
