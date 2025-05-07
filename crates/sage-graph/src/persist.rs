//! Tiny helper for reading / writing the graph JSON file.

use anyhow::{Context, Result};
use std::{fs, path::PathBuf, process::Command};

use crate::SageGraph;

/// `.git/sage_graph.json` (located via `git rev-parse`).
pub(crate) fn file_path() -> PathBuf {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .expect("failed to run `git rev-parse --show-toplevel`");
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    PathBuf::from(root).join(".git").join("sage_graph.json")
}

pub(crate) fn load() -> Result<SageGraph> {
    let location = file_path();
    match fs::read_to_string(location) {
        Ok(raw) => Ok(serde_json::from_str(&raw)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(SageGraph::default()),
        Err(e) => Err(e).context("reading stack_graph file"),
    }
}

pub(crate) fn save(graph: &SageGraph) -> Result<()> {
    let location = file_path();
    let raw = serde_json::to_string_pretty(graph)?;
    fs::write(location, raw).context("writing stack_graph file")?;
    Ok(())
}
