use anyhow::{Error, Result};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor, Stylize},
    terminal::{self, ClearType},
};
use std::{
    io::{self, Write},
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};

mod components;
mod editor;
mod error;
mod input;
mod multi_select;
mod progress;
mod select;
mod theme;
mod tree;

pub use components::{FileChange, FileStatus, MessageType, SummaryItem};
pub use editor::TextEditor;
pub use error::{ErrorDisplay, format_error_compact};
pub use input::TextInput;
pub use multi_select::MultiSelect;
pub use progress::{ProgressBar, ProgressHandle};
pub use select::Select;
pub use theme::Theme;
pub use tree::{NodeMetadata, TreeNode};

pub struct Tui {
    theme: Theme,
    use_color: bool,
    is_ci: bool,
    needs_clear: Arc<AtomicBool>,
    term_width: u16,
}

impl Tui {
    pub fn new() -> Self {
        let term_width = terminal::size().map(|(w, _)| w).unwrap_or(80);

        Self {
            theme: Theme::default(),
            use_color: supports_color(),
            is_ci: is_ci_environment(),
            needs_clear: Arc::new(AtomicBool::new(false)),
            term_width,
        }
    }

    pub fn with_theme(theme: Theme) -> Self {
        let mut tui = Self::new();
        tui.theme = theme;
        tui
    }

    pub fn header(&self, command: &str) -> Result<()> {
        self.clear_if_needed()?;

        if !self.is_ci {
            println!("sage {}\n", self.style(command, self.theme.muted));
        } else {
            println!("sage {}", command);
        }

        Ok(())
    }

    pub fn summary(&self, items: &[SummaryItem]) -> Result<()> {
        self.clear_if_needed()?;

        print!("  ");

        let parts: Vec<String> = items
            .iter()
            .map(|item| match item {
                SummaryItem::Count(label, count) => {
                    if *count == 1 {
                        format!(
                            "{} {}",
                            self.style(&count.to_string(), Color::White),
                            label.trim_end_matches('s')
                        )
                    } else {
                        format!("{} {}", self.style(&count.to_string(), Color::White), label)
                    }
                }
                SummaryItem::Changes(add, del) => {
                    format!(
                        "{}  {}",
                        self.style(&format!("+{}", add), self.theme.success),
                        self.style(&format!("-{}", del), self.theme.error)
                    )
                }
                SummaryItem::Text(text) => {
                    if text.starts_with('↓') {
                        self.style(text, self.theme.warning).to_string()
                    } else if text.starts_with('↑') {
                        self.style(text, self.theme.success).to_string()
                    } else {
                        text.clone()
                    }
                }
            })
            .collect();

        println!("{}\n", parts.join("  "));
        Ok(())
    }

    pub fn file_list(&self, files: &[FileChange]) -> Result<()> {
        self.clear_if_needed()?;

        if files.is_empty() {
            return Ok(());
        }

        let max_path_len = files
            .iter()
            .map(|f| f.path.len())
            .max()
            .unwrap_or(0)
            .min(50);

        for file in files {
            let path = if file.path.len() > max_path_len {
                format!("{}...", &file.path[..max_path_len - 3])
            } else {
                file.path.clone()
            };

            print!("    {:<width$}", path, width = max_path_len + 2);

            match file.status {
                FileStatus::Modified => {
                    if file.additions > 0 && file.deletions > 0 {
                        print!(
                            "{:>5} {:>4}",
                            self.style(&format!("+{}", file.additions), self.theme.success),
                            self.style(&format!("-{}", file.deletions), self.theme.error)
                        );
                    } else if file.additions > 0 {
                        print!(
                            "{:>9}",
                            self.style(&format!("+{}", file.additions), self.theme.success)
                        );
                    } else if file.deletions > 0 {
                        print!(
                            "     {:>4}",
                            self.style(&format!("-{}", file.deletions), self.theme.error)
                        );
                    }
                }
                FileStatus::Added => {
                    print!(
                        "{:>9}",
                        self.style(&format!("+{}", file.additions), self.theme.success)
                    );
                }
                FileStatus::Deleted => {
                    print!(
                        "     {:>4}",
                        self.style(&format!("-{}", file.deletions), self.theme.error)
                    );
                }
                FileStatus::Renamed => {
                    print!("{:>9}", self.style("renamed", self.theme.info));
                }
            }

            if let Some(desc) = &file.description {
                print!("  {}", self.style(desc, self.theme.muted));
            }

            println!();
        }

        println!();
        Ok(())
    }

    pub fn progress(&self, message: &str) -> ProgressHandle {
        let _ = self.clear_if_needed();
        ProgressHandle::new(
            message.to_string(),
            self.use_color && !self.is_ci,
            self.needs_clear.clone(),
            self.theme.clone(),
        )
    }

    pub fn progress_bar(&self, message: &str, total: u64) -> ProgressBar {
        let _ = self.clear_if_needed();
        ProgressBar::new(
            message.to_string(),
            total,
            self.use_color && !self.is_ci,
            self.needs_clear.clone(),
        )
    }

    pub fn tree(&self, root: TreeNode) -> Result<()> {
        self.clear_if_needed()?;
        self.print_tree_node(&root, "", true)?;
        println!();
        Ok(())
    }

    pub fn commit_message(&self, title: &str, body: Option<&str>) -> Result<()> {
        self.clear_if_needed()?;

        println!("\n  {}", title);

        if let Some(body) = body {
            if !body.is_empty() {
                println!("\n  {}", self.style(body, self.theme.muted));
            }
        }

        println!();
        Ok(())
    }

    pub fn prompt(&self, message: &str, options: &[char]) -> Result<char> {
        self.clear_if_needed()?;

        print!("  {} (", message);

        for (i, opt) in options.iter().enumerate() {
            if i == 0 {
                execute!(
                    io::stdout(),
                    SetForegroundColor(self.theme.primary),
                    SetAttribute(Attribute::Bold),
                    Print(opt.to_string()),
                    SetAttribute(Attribute::Reset),
                    ResetColor,
                )?;
            } else {
                print!("{}", opt);
            }

            if i < options.len() - 1 {
                print!("/");
            }
        }

        print!(")? ");
        io::stdout().flush()?;

        self.needs_clear.store(true, Ordering::Relaxed);

        let result = self.read_char(options)?;

        self.clear_current_line()?;
        self.needs_clear.store(false, Ordering::Relaxed);

        Ok(result)
    }

    pub fn message(&self, msg_type: MessageType, text: &str) -> Result<()> {
        self.clear_if_needed()?;

        let (symbol, color) = match msg_type {
            MessageType::Success => ("✓", self.theme.success),
            MessageType::Error => ("✗", self.theme.error),
            MessageType::Warning => ("!", self.theme.warning),
            MessageType::Info => ("·", self.theme.info),
        };

        println!("  {} {}", self.style(symbol, color), text);
        Ok(())
    }

    pub fn hint(&self, text: &str) -> Result<()> {
        self.clear_if_needed()?;
        println!("    {}", self.style(text, self.theme.muted));
        Ok(())
    }

    pub fn info_section(&self, title: Option<&str>, items: &[(String, String)]) -> Result<()> {
        self.clear_if_needed()?;

        if let Some(title) = title {
            println!("  {}", self.style(title, self.theme.muted));
        }

        if !items.is_empty() {
            let max_key_len = items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

            for (key, value) in items {
                print!(
                    "  {:<width$}    ",
                    self.style(key, self.theme.muted),
                    width = max_key_len
                );

                if value.contains("→") {
                    let parts: Vec<&str> = value.split("→").collect();
                    if parts.len() == 2 {
                        print!(
                            "{} {} {}",
                            self.style(parts[0].trim(), self.theme.primary),
                            self.style("→", self.theme.muted),
                            parts[1].trim()
                        );

                        if let Some(remaining) = value.split("→").nth(1) {
                            if remaining.contains('↑') || remaining.contains('↓') {
                                print!("  ");
                                self.print_colored_metrics(remaining)?;
                            }
                        }
                        println!();
                    } else {
                        println!("{}", value);
                    }
                } else if value.contains('↑') || value.contains('↓') {
                    self.print_colored_metrics(value)?;
                    println!();
                } else if value.contains("ago") {
                    println!("{}", self.style(value, self.theme.muted));
                } else {
                    println!("{}", value);
                }
            }
        }

        println!();
        Ok(())
    }

    pub fn print(&self, text: &str, color: Option<Color>) -> Result<()> {
        self.clear_if_needed()?;

        if let Some(color) = color {
            print!("{}", self.style(text, color));
        } else {
            print!("{}", text);
        }

        io::stdout().flush()?;
        Ok(())
    }

    pub fn println(&self, text: &str, color: Option<Color>) -> Result<()> {
        self.clear_if_needed()?;

        if let Some(color) = color {
            println!("{}", self.style(text, color));
        } else {
            println!("{}", text);
        }

        Ok(())
    }

    pub fn edit_text(&self, title: &str) -> TextEditor {
        let _ = self.clear_if_needed();
        let _ = self.message(MessageType::Info, title);
        TextEditor::new()
    }

    pub fn select<T>(&self, prompt: &str, items: Vec<T>) -> Select<T> {
        let _ = self.clear_if_needed();
        Select::new(prompt, items).with_theme(self.theme.clone())
    }

    pub fn multi_select<T>(&self, prompt: &str, items: Vec<T>) -> MultiSelect<T> {
        let _ = self.clear_if_needed();
        MultiSelect::new(prompt, items)
            .with_theme(self.theme.clone())
            .with_color(self.use_color)
    }

    pub fn input(&self, prompt: &str) -> TextInput {
        let _ = self.clear_if_needed();
        TextInput::new(prompt).with_theme(self.theme.clone())
    }

    pub fn error(&self, error: &Error) -> Result<()> {
        let display = ErrorDisplay::new(self.theme.clone());
        display.show(error)?;
        Ok(())
    }

    pub fn error_with_context(&self, error: &Error, context: &str) -> Result<()> {
        let display = ErrorDisplay::new(self.theme.clone());
        display.show_with_context(error, context)?;
        Ok(())
    }

    fn print_tree_node(&self, node: &TreeNode, prefix: &str, is_last: bool) -> Result<()> {
        if prefix.is_empty() {
            print!("  ");
        } else {
            print!("  {}", prefix);
            print!("{}", if is_last { "└─ " } else { "├─ " });
        }

        print!("{}", node.name);

        if !node.metadata.is_empty() {
            let metadata_str = node
                .metadata
                .iter()
                .map(|m| match m {
                    NodeMetadata::Current => {
                        self.style("● current", self.theme.primary).to_string()
                    }
                    NodeMetadata::Ahead(n) => self
                        .style(&format!("↑{}", n), self.theme.success)
                        .to_string(),
                    NodeMetadata::Behind(n) => self
                        .style(&format!("↓{}", n), self.theme.warning)
                        .to_string(),
                    NodeMetadata::Draft => self.style("draft", self.theme.muted).to_string(),
                    NodeMetadata::Text(s) => self.style(s, self.theme.muted).to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ");

            let name_len = if prefix.is_empty() {
                2
            } else {
                prefix.len() + 5
            } + node.name.len();
            let dots_needed = (50_usize.saturating_sub(name_len))
                .saturating_sub(visible_len(&metadata_str))
                .min(30);

            if dots_needed > 2 {
                print!(" {}", "·".repeat(dots_needed));
            }

            print!(" {}", metadata_str);
        }

        println!();

        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            let child_prefix = if prefix.is_empty() {
                String::new()
            } else {
                format!("{}{}  ", prefix, if is_last { " " } else { "│" })
            };

            self.print_tree_node(child, &child_prefix, is_last_child)?;
        }

        Ok(())
    }

    fn style(&self, text: &str, color: Color) -> String {
        if self.use_color && !self.is_ci {
            format!("{}", text.with(color))
        } else {
            text.to_string()
        }
    }

    fn print_colored_metrics(&self, text: &str) -> Result<()> {
        for ch in text.chars() {
            match ch {
                '↑' => {
                    execute!(
                        io::stdout(),
                        SetForegroundColor(self.theme.success),
                        SetAttribute(Attribute::Bold),
                        Print("↑"),
                        SetAttribute(Attribute::Reset),
                        ResetColor,
                    )?;
                }
                '↓' => {
                    execute!(
                        io::stdout(),
                        SetForegroundColor(self.theme.warning),
                        SetAttribute(Attribute::Bold),
                        Print("↓"),
                        SetAttribute(Attribute::Reset),
                        ResetColor,
                    )?;
                }
                '0'..='9' => {
                    execute!(
                        io::stdout(),
                        SetAttribute(Attribute::Bold),
                        Print(ch.to_string()),
                        SetAttribute(Attribute::Reset),
                    )?;
                }
                _ => print!("{}", ch),
            }
        }
        Ok(())
    }

    fn clear_if_needed(&self) -> Result<()> {
        if self.needs_clear.load(Ordering::Relaxed) {
            self.clear_current_line()?;
            self.needs_clear.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    fn clear_current_line(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine)
        )
        .map_err(|e| anyhow::anyhow!("Failed to clear line: {}", e))?;
        Ok(())
    }

    fn read_char(&self, valid_options: &[char]) -> Result<char> {
        terminal::enable_raw_mode()?;

        let result = loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char(c) => {
                        let c = c.to_ascii_lowercase();
                        if valid_options.contains(&c) {
                            break Ok(c);
                        }
                    }
                    KeyCode::Esc => break Ok('\x1b'),
                    KeyCode::Enter => {
                        if !valid_options.is_empty() {
                            break Ok(valid_options[0]);
                        }
                    }
                    _ => {}
                }
            }
        };

        terminal::disable_raw_mode()?;
        result
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}

fn supports_color() -> bool {
    atty::is(atty::Stream::Stdout)
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("CIRCLECI").is_ok()
}

fn visible_len(s: &str) -> usize {
    let stripped = strip_ansi_escapes::strip(s.as_bytes());
    String::from_utf8_lossy(&stripped).len()
}
