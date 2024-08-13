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

// TODO: Allow this to be configurable, or determined by source files themselves through `meta`.
static TEMP_DEFAULT_LOCALE: &str = "en-US";
