use std::path::PathBuf;
use xshell::{cmd, Shell};

pub fn repo_root() -> PathBuf {
    std::path::Path::new(
        &std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}

pub fn format_file(path: &PathBuf) -> anyhow::Result<()> {
    let shell = Shell::new()?;
    cmd!(shell, "cargo fmt -- {path}").run()?;
    Ok(())
}

pub fn format_files<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> anyhow::Result<()> {
    let shell = Shell::new()?;
    cmd!(shell, "cargo fmt -- {paths...}").run()?;
    Ok(())
}
