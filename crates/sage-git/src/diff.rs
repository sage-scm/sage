use std::fmt::Write;

use anyhow::{Context, Result, anyhow, bail};
use gix::ObjectId;
use gix::bstr::{BStr, BString, ByteSlice};
use gix::diff::blob::unified_diff::{ContextSize, NewlineSeparator};
use gix::diff::blob::{self, ResourceKind, UnifiedDiff};
use gix::diff::index::Action as DiffAction;
use gix::index::entry::Mode as IndexMode;
use gix::status::tree_index::TrackRenames;

use crate::Repo;

struct Side {
    path: BString,
    mode: IndexMode,
    id: ObjectId,
    kind: gix::object::tree::EntryKind,
}

struct ChangeData {
    old: Option<Side>,
    new: Option<Side>,
    copy: bool,
}

impl Repo {
    pub fn diff_ai(&self) -> Result<String> {
        let index = match self.repo.open_index() {
            Ok(index) => index,
            Err(err) => {
                if let gix::worktree::open_index::Error::IndexFile(
                    gix::index::file::init::Error::Io(io_err),
                ) = &err
                {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        bail!("No staged changes to diff.");
                    }
                }
                Err(err).context("failed to open git index")?
            }
        };

        let head_tree_id = self.repo.head_tree_id_or_empty()?.detach();
        let mut changes = Vec::new();

        self.repo
            .tree_index_status(
                head_tree_id.as_ref(),
                &index,
                None,
                TrackRenames::AsConfigured,
                |change, _, _| {
                    changes.push(change.into_owned());
                    Ok::<_, std::convert::Infallible>(DiffAction::Continue)
                },
            )
            .context("failed to compute staged diff via gix")?;

        if changes.is_empty() {
            bail!("No staged changes to diff.");
        }

        let mut cache = self
            .repo
            .diff_resource_cache_for_tree_diff()
            .context("failed to prepare diff cache")?;
        cache.options.skip_internal_diff_if_external_is_configured = false;

        let hash_kind = self.repo.object_hash();
        let mut pieces = Vec::with_capacity(changes.len());

        for change in changes {
            cache.clear_resource_cache_keep_allocation();
            pieces.push(render_change(&self.repo, hash_kind, &mut cache, change)?);
        }

        let body = pieces.join("\n");
        if body.trim().is_empty() {
            bail!("No staged changes to diff.");
        }

        Ok(format!("# Diff Content\n{body}"))
    }
}

fn render_change(
    repo: &gix::Repository,
    hash_kind: gix::hash::Kind,
    cache: &mut blob::Platform,
    change: gix::diff::index::Change,
) -> Result<String> {
    let data = build_change(change)?;

    if is_submodule(&data) {
        return Ok(render_submodule(&data));
    }

    set_side(
        cache,
        repo,
        hash_kind,
        ResourceKind::OldOrSource,
        data.old.as_ref(),
        data.new.as_ref(),
    )?;
    set_side(
        cache,
        repo,
        hash_kind,
        ResourceKind::NewOrDestination,
        data.new.as_ref(),
        data.old.as_ref(),
    )?;

    let outcome = cache.prepare_diff()?;
    match &outcome.operation {
        blob::platform::prepare_diff::Operation::InternalDiff { algorithm } => {
            let tokens = outcome.interned_input();
            let sink = UnifiedDiff::new(
                &tokens,
                String::new(),
                NewlineSeparator::AfterHeaderAndLine("\n"),
                ContextSize::symmetrical(10),
            );
            let diff_text = blob::diff(*algorithm, &tokens, sink)?;
            let mut body = build_header(&data);
            body.push_str(&diff_text);
            Ok(body)
        }
        blob::platform::prepare_diff::Operation::SourceOrDestinationIsBinary => {
            let mut body = build_header(&data);
            let old_name = marker_name(data.old.as_ref(), "a");
            let new_name = marker_name(data.new.as_ref(), "b");
            writeln!(body, "Binary files {old_name} and {new_name} differ")?;
            Ok(body)
        }
        blob::platform::prepare_diff::Operation::ExternalCommand { .. } => {
            Err(anyhow!("External diff drivers are not supported"))
        }
    }
}

fn build_change(change: gix::diff::index::Change) -> Result<ChangeData> {
    match change {
        gix::diff::index::Change::Addition {
            location,
            entry_mode,
            id,
            ..
        } => {
            let new_side = build_side(location.as_ref(), entry_mode, id.into_owned())?;
            Ok(ChangeData {
                old: None,
                new: Some(new_side),
                copy: false,
            })
        }
        gix::diff::index::Change::Deletion {
            location,
            entry_mode,
            id,
            ..
        } => {
            let old_side = build_side(location.as_ref(), entry_mode, id.into_owned())?;
            Ok(ChangeData {
                old: Some(old_side),
                new: None,
                copy: false,
            })
        }
        gix::diff::index::Change::Modification {
            location,
            previous_entry_mode,
            previous_id,
            entry_mode,
            id,
            ..
        } => {
            let old_side = build_side(
                location.as_ref(),
                previous_entry_mode,
                previous_id.into_owned(),
            )?;
            let new_side = build_side(location.as_ref(), entry_mode, id.into_owned())?;
            Ok(ChangeData {
                old: Some(old_side),
                new: Some(new_side),
                copy: false,
            })
        }
        gix::diff::index::Change::Rewrite {
            source_location,
            source_entry_mode,
            source_id,
            location,
            entry_mode,
            id,
            copy,
            ..
        } => {
            let old_side = build_side(
                source_location.as_ref(),
                source_entry_mode,
                source_id.into_owned(),
            )?;
            let new_side = build_side(location.as_ref(), entry_mode, id.into_owned())?;
            Ok(ChangeData {
                old: Some(old_side),
                new: Some(new_side),
                copy,
            })
        }
    }
}

fn build_side(path: &BStr, mode: IndexMode, id: ObjectId) -> Result<Side> {
    let kind = mode
        .to_tree_entry_mode()
        .map(|entry_mode: gix::object::tree::EntryMode| entry_mode.kind())
        .ok_or_else(|| {
            anyhow!(
                "Unsupported index mode for '{}': {:o}",
                format_path(path),
                mode.bits()
            )
        })?;
    Ok(Side {
        path: path.to_owned(),
        mode,
        id,
        kind,
    })
}

fn set_side(
    cache: &mut blob::Platform,
    repo: &gix::Repository,
    hash_kind: gix::hash::Kind,
    role: ResourceKind,
    primary: Option<&Side>,
    fallback: Option<&Side>,
) -> Result<()> {
    if let Some(side) = primary {
        cache.set_resource(side.id.clone(), side.kind, side.path.as_bstr(), role, repo)?;
    } else if let Some(side) = fallback {
        cache.set_resource(hash_kind.null(), side.kind, side.path.as_bstr(), role, repo)?;
    }
    Ok(())
}

fn is_submodule(data: &ChangeData) -> bool {
    matches!(
        data.old.as_ref().map(|s| s.kind),
        Some(gix::object::tree::EntryKind::Commit)
    ) || matches!(
        data.new.as_ref().map(|s| s.kind),
        Some(gix::object::tree::EntryKind::Commit)
    )
}

fn render_submodule(data: &ChangeData) -> String {
    let mut body = build_header(data);
    match (&data.old, &data.new) {
        (None, Some(new_side)) => {
            writeln!(body, "Submodule added at {}", new_side.id.to_hex()).unwrap();
        }
        (Some(old_side), None) => {
            writeln!(body, "Submodule removed (was {})", old_side.id.to_hex()).unwrap();
        }
        (Some(old_side), Some(new_side)) => {
            writeln!(
                body,
                "Submodule updated {} -> {}",
                old_side.id.to_hex(),
                new_side.id.to_hex()
            )
            .unwrap();
        }
        (None, None) => {}
    }
    body
}
fn build_header(data: &ChangeData) -> String {
    let old_path = data.old.as_ref().map(|s| format_path(s.path.as_bstr()));
    let new_path = data.new.as_ref().map(|s| format_path(s.path.as_bstr()));
    let diff_old = old_path
        .as_deref()
        .unwrap_or_else(|| new_path.as_deref().unwrap());
    let diff_new = new_path
        .as_deref()
        .unwrap_or_else(|| old_path.as_deref().unwrap());

    let mut header = String::new();
    writeln!(header, "diff --git a/{diff_old} b/{diff_new}").unwrap();

    match (&data.old, &data.new) {
        (None, Some(new_side)) => {
            writeln!(header, "new file mode {:06o}", new_side.mode.bits()).unwrap();
        }
        (Some(old_side), None) => {
            writeln!(header, "deleted file mode {:06o}", old_side.mode.bits()).unwrap();
        }
        (Some(old_side), Some(new_side)) => {
            if old_side.path != new_side.path {
                if data.copy {
                    writeln!(header, "copy from {}", format_path(old_side.path.as_bstr())).unwrap();
                    writeln!(header, "copy to {}", format_path(new_side.path.as_bstr())).unwrap();
                } else {
                    writeln!(
                        header,
                        "rename from {}",
                        format_path(old_side.path.as_bstr())
                    )
                    .unwrap();
                    writeln!(header, "rename to {}", format_path(new_side.path.as_bstr())).unwrap();
                }
            }
            if old_side.mode.bits() != new_side.mode.bits() {
                writeln!(header, "old mode {:06o}", old_side.mode.bits()).unwrap();
                writeln!(header, "new mode {:06o}", new_side.mode.bits()).unwrap();
            }
        }
        (None, None) => {}
    }

    let old_marker = if data.old.is_some() {
        format!("a/{diff_old}")
    } else {
        "/dev/null".to_string()
    };
    let new_marker = if data.new.is_some() {
        format!("b/{diff_new}")
    } else {
        "/dev/null".to_string()
    };

    writeln!(header, "--- {old_marker}").unwrap();
    writeln!(header, "+++ {new_marker}").unwrap();
    header
}

fn marker_name(side: Option<&Side>, prefix: &str) -> String {
    match side {
        Some(s) => format!("{prefix}/{}", format_path(s.path.as_bstr())),
        None => "/dev/null".to_string(),
    }
}

fn format_path(path: &BStr) -> String {
    path.to_str_lossy().into_owned()
}
