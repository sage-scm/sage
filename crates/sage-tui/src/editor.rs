use anyhow::{Result, bail};
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Opens text in the user's preferred editor
pub struct TextEditor {
    /// Initial content to edit
    initial_content: String,
    /// Template content (appears before initial content)
    template: Option<String>,
    /// Comment lines to add at the top (prefixed with #)
    comments: Vec<String>,
    /// File extension for syntax highlighting
    suffix: String,
    /// Validation function
    validate: Option<Box<dyn Fn(&str) -> Result<()>>>,
    /// Strip comment lines from result
    strip_comments: bool,
    /// Editor command (defaults to $EDITOR)
    editor_cmd: Option<String>,
}

impl TextEditor {
    /// Create a new text editor
    pub fn new() -> Self {
        Self {
            initial_content: String::new(),
            template: None,
            comments: Vec::new(),
            suffix: ".txt".into(),
            validate: None,
            strip_comments: true,
            editor_cmd: None,
        }
    }

    /// Set initial content
    pub fn initial_content(mut self, content: impl Into<String>) -> Self {
        self.initial_content = content.into();
        self
    }

    /// Set template content (appears before initial content)
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add comment lines at the top (will be prefixed with #)
    pub fn comments(mut self, comments: Vec<impl Into<String>>) -> Self {
        self.comments = comments.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add a single comment line
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comments.push(comment.into());
        self
    }

    /// Set file syntax for highlighting (markdown, json, yaml, toml, rust, etc.)
    pub fn syntax(mut self, syntax: &str) -> Self {
        self.suffix = match syntax.to_lowercase().as_str() {
            "markdown" | "md" => ".md".into(),
            "json" => ".json".into(),
            "yaml" | "yml" => ".yml".into(),
            "toml" => ".toml".into(),
            "rust" | "rs" => ".rs".into(),
            "python" | "py" => ".py".into(),
            "javascript" | "js" => ".js".into(),
            "typescript" | "ts" => ".ts".into(),
            "shell" | "sh" | "bash" => ".sh".into(),
            "text" | "txt" => ".txt".into(),
            ext => format!(".{}", ext),
        };
        self
    }

    /// Set validation function
    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<()> + 'static,
    {
        self.validate = Some(Box::new(f));
        self
    }

    /// Set whether to strip comment lines from result (default: true)
    pub fn strip_comments(mut self, strip: bool) -> Self {
        self.strip_comments = strip;
        self
    }

    /// Set specific editor command (overrides $EDITOR)
    pub fn editor(mut self, cmd: impl Into<String>) -> Self {
        self.editor_cmd = Some(cmd.into());
        self
    }

    /// Open the editor and return the edited content
    pub fn run(self) -> Result<String> {
        // Create temp file with appropriate extension
        let tmp = NamedTempFile::with_suffix(&self.suffix)?;
        let tmp_path = tmp.path().to_owned();

        // Build content
        let mut content = String::new();

        // Add comments
        for comment in &self.comments {
            content.push_str("# ");
            content.push_str(comment);
            content.push('\n');
        }

        if !self.comments.is_empty()
            && (self.template.is_some() || !self.initial_content.is_empty())
        {
            content.push('\n');
        }

        // Add template
        if let Some(template) = &self.template {
            content.push_str(template);
            if !template.ends_with('\n') && !self.initial_content.is_empty() {
                content.push('\n');
            }
        }

        // Add initial content
        content.push_str(&self.initial_content);

        // Write to temp file
        fs::write(&tmp_path, content)?;

        // Determine editor command
        let editor = self.editor_cmd.unwrap_or_else(|| {
            env::var("EDITOR")
                .or_else(|_| env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    if cfg!(windows) {
                        "notepad".into()
                    } else if cfg!(target_os = "macos") {
                        // Check if we're in a terminal that supports vim
                        which::which("vim")
                            .map(|_| "vim".to_string())
                            .unwrap_or_else(|_| "nano".into())
                    } else {
                        "vim".into()
                    }
                })
        });

        // Open editor
        let status = if cfg!(windows) && editor == "notepad" {
            // Special handling for notepad (doesn't block by default)
            Command::new("cmd")
                .args(&[
                    "/C",
                    "start",
                    "/WAIT",
                    "notepad",
                    tmp_path.to_str().unwrap(),
                ])
                .status()?
        } else {
            Command::new(&editor).arg(&tmp_path).status()?
        };

        if !status.success() {
            bail!("Editor '{}' exited with error", editor);
        }

        // Read back the edited content
        let edited = fs::read_to_string(&tmp_path)?;

        // Process the content
        let result = if self.strip_comments {
            edited
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            edited
        };

        // Validate if needed
        if let Some(validate) = &self.validate {
            validate(&result)?;
        }

        Ok(result)
    }
}
