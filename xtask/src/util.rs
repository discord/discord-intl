use quote::format_ident;
use std::ops::Deref;
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

pub fn format_files<'a>(paths: impl IntoIterator<Item = &'a PathBuf>) -> anyhow::Result<()> {
    let shell = Shell::new()?;
    cmd!(shell, "cargo fmt -- {paths...}").run()?;
    Ok(())
}

pub fn as_ident(name: &str) -> proc_macro2::Ident {
    format_ident!("{}", &*name)
}

pub struct Codegen {
    root: PathBuf,
    files: Vec<PathBuf>,
}

impl Codegen {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: vec![],
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn write_file<'a, P: Into<PathBuf>, S: Deref<Target = str>>(
        &mut self,
        path: P,
        contents: S,
    ) -> anyhow::Result<()> {
        let mut target_path = path.into();
        if target_path.is_relative() {
            target_path = self.root.join(target_path);
        }
        if let Some(target_dir) = target_path.parent() {
            std::fs::create_dir_all(target_dir)?;
        }

        std::fs::write(&target_path, &*contents)?;
        self.files.push(target_path);
        Ok(())
    }

    pub fn finish(self) -> anyhow::Result<()> {
        format_files(&self.files)
    }
}
