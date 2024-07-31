use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod messages;
pub mod services;
pub mod sources;
mod threading;

pub mod napi;

#[cfg(test)]
pub mod test;
