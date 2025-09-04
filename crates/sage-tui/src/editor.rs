use anyhow::{Result, bail};
use std::env;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct TextEditor {
    initial_content: String,
    template: Option<String>,
    comments: Vec<String>,
    suffix: String,
    validate: Option<Box<dyn Fn(&str) -> Result<()>>>,
    strip_comments: bool,
    editor_cmd: Option<String>,
}

impl TextEditor {
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

    pub fn initial_content(mut self, content: impl Into<String>) -> Self {
        self.initial_content = content.into();
        self
    }

    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    pub fn comments(mut self, comments: Vec<impl Into<String>>) -> Self {
        self.comments = comments.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comments.push(comment.into());
        self
    }

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

    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<()> + 'static,
    {
        self.validate = Some(Box::new(f));
        self
    }

    pub fn strip_comments(mut self, strip: bool) -> Self {
        self.strip_comments = strip;
        self
    }

    pub fn editor(mut self, cmd: impl Into<String>) -> Self {
        self.editor_cmd = Some(cmd.into());
        self
    }

    pub fn run(self) -> Result<String> {
        let tmp = NamedTempFile::with_suffix(&self.suffix)?;
        let tmp_path = tmp.path().to_owned();
        let mut content = String::new();
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
        if let Some(template) = &self.template {
            content.push_str(template);
            if !template.ends_with('\n') && !self.initial_content.is_empty() {
                content.push('\n');
            }
        }
        content.push_str(&self.initial_content);
        fs::write(&tmp_path, content)?;
        let editor = self.editor_cmd.unwrap_or_else(|| {
            env::var("EDITOR")
                .or_else(|_| env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    if cfg!(windows) {
                        "notepad".into()
                    } else if cfg!(target_os = "macos") {
                        which::which("vi")
                            .map(|_| "vi".to_string())
                            .unwrap_or_else(|_| "nano".into())
                    } else {
                        "vim".into()
                    }
                })
        });
        let status = if cfg!(windows) && editor == "notepad" {
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
        let edited = fs::read_to_string(&tmp_path)?;
        let result = if self.strip_comments {
            edited
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            edited
        };
        if let Some(validate) = &self.validate {
            validate(&result)?;
        }

        Ok(result)
    }
}
