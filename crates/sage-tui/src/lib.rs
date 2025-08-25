use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Stylize},
    terminal::{self, ClearType},
};
use std::{
    io::{self, Write},
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};

mod components;
mod progress;
mod theme;
mod tree;

pub use components::{FileChange, FileStatus, MessageType, SummaryItem};
pub use progress::{ProgressBar, ProgressHandle};
pub use theme::Theme;
pub use tree::{NodeMetadata, TreeNode};

// Main TUI structure for managing terminal output
pub struct Tui {
    /// Color theme
    theme: Theme,
    /// Whether colors are enabled
    use_color: bool,
    /// Whether we're in CI environment
    is_ci: bool,
    /// Track if current line needs clearing
    needs_clear: Arc<AtomicBool>,
    /// Terminal width for responsive layouts
    term_width: u16,
}

impl Tui {
    /// Create a new TUI instance
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

    /// Create with custom theme
    pub fn with_theme(theme: Theme) -> Self {
        let mut tui = Self::new();
        tui.theme = theme;
        tui
    }

    /// Print command header
    pub fn header(&self, command: &str) -> Result<()> {
        self.clear_if_needed()?;

        if !self.is_ci {
            println!("sage {}\n", self.style(command, self.theme.muted));
        } else {
            println!("sage {}", command);
        }

        Ok(())
    }

    /// Print summary line (e.g., "3 files  +89 -23")
    pub fn summary(&self, items: &[SummaryItem]) -> Result<()> {
        self.clear_if_needed()?;

        let parts: Vec<String> = items
            .iter()
            .map(|item| match item {
                SummaryItem::Count(label, count) => {
                    format!("{} {}", count, label)
                }
                SummaryItem::Changes(add, del) => {
                    format!(
                        "{}  {}",
                        self.style(&format!("+{}", add), self.theme.success),
                        self.style(&format!("-{}", del), self.theme.error)
                    )
                }
                SummaryItem::Text(text) => {
                    // Handle special text like "↓ 3 new commits"
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

    /// Display file changes in aligned columns
    pub fn file_list(&self, files: &[FileChange]) -> Result<()> {
        self.clear_if_needed()?;

        if files.is_empty() {
            return Ok(());
        }

        // Calculate column widths
        let max_path_len = files
            .iter()
            .map(|f| f.path.len())
            .max()
            .unwrap_or(0)
            .min(50); // Cap at 50 chars

        for file in files {
            // Path (truncate if needed)
            let path = if file.path.len() > max_path_len {
                format!("{}...", &file.path[..max_path_len - 3])
            } else {
                file.path.clone()
            };

            print!("  {:<width$}", path, width = max_path_len + 2);

            // Status and changes
            match file.status {
                FileStatus::Modified => {
                    if file.additions > 0 && file.deletions > 0 {
                        print!(
                            "{:>5} {:>4}   ",
                            self.style(&format!("+{}", file.additions), self.theme.success),
                            self.style(&format!("-{}", file.deletions), self.theme.error)
                        );
                    } else if file.additions > 0 {
                        print!(
                            "{:>10}   ",
                            self.style(&format!("+{}", file.additions), self.theme.success)
                        );
                    } else if file.deletions > 0 {
                        print!(
                            "{:>10}   ",
                            self.style(&format!("-{}", file.deletions), self.theme.error)
                        );
                    }
                }
                FileStatus::Added => {
                    print!(
                        "{:>10}   ",
                        self.style(&format!("+{}", file.additions), self.theme.success)
                    );
                }
                FileStatus::Deleted => {
                    print!(
                        "{:>10}   ",
                        self.style(&format!("-{}", file.deletions), self.theme.error)
                    );
                }
                FileStatus::Renamed => {
                    print!("{:>10}   ", self.style("renamed", self.theme.info));
                }
            }

            // Description
            if let Some(desc) = &file.description {
                print!("{}", self.style(desc, self.theme.muted));
            }

            println!();
        }

        println!();
        Ok(())
    }

    /// Start a progress indicator
    pub fn progress(&self, message: &str) -> ProgressHandle {
        let _ = self.clear_if_needed();
        ProgressHandle::new(
            message.to_string(),
            self.use_color && !self.is_ci,
            self.needs_clear.clone(),
        )
    }

    /// Start a progress bar with known total
    pub fn progress_bar(&self, message: &str, total: u64) -> ProgressBar {
        let _ = self.clear_if_needed();
        ProgressBar::new(
            message.to_string(),
            total,
            self.use_color && !self.is_ci,
            self.needs_clear.clone(),
        )
    }

    /// Display a tree structure (for stack visualization)
    pub fn tree(&self, root: TreeNode) -> Result<()> {
        self.clear_if_needed()?;
        self.print_tree_node(&root, "", true)?;
        println!();
        Ok(())
    }

    /// Print commit message with title and optional body
    pub fn commit_message(&self, title: &str, body: Option<&str>) -> Result<()> {
        self.clear_if_needed()?;

        println!("{}", title);

        if let Some(body) = body {
            if !body.is_empty() {
                println!("\n{}", self.style(body, self.theme.muted));
            }
        }

        println!();
        Ok(())
    }

    /// Interactive prompt that clears after input
    pub fn prompt(&self, message: &str, options: &[char]) -> Result<char> {
        self.clear_if_needed()?;

        // Build options string
        let options_str = if options.len() <= 3 {
            options
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("/")
        } else {
            format!("{}/{}/..", options[0], options[1])
        };

        // Print prompt
        print!(
            "{} · {} · ",
            message,
            self.style(&options_str, self.theme.muted)
        );
        io::stdout().flush()?;

        // Mark that we need to clear this line
        self.needs_clear.store(true, Ordering::Relaxed);

        // Read input
        let result = self.read_char(options)?;

        // Clear the prompt line
        self.clear_current_line()?;

        Ok(result)
    }

    /// Display a message with type indicator
    pub fn message(&self, msg_type: MessageType, text: &str) -> Result<()> {
        self.clear_if_needed()?;

        let (symbol, color) = match msg_type {
            MessageType::Success => ("✓", self.theme.success),
            MessageType::Error => ("✗", self.theme.error),
            MessageType::Warning => ("!", self.theme.warning),
            MessageType::Info => ("·", self.theme.info),
        };

        println!("{} {}\n", self.style(symbol, color), text);
        Ok(())
    }

    /// Display a hint (usually after an error)
    pub fn hint(&self, text: &str) -> Result<()> {
        self.clear_if_needed()?;

        // Indent hints and color them muted
        for line in text.lines() {
            println!("  {}", self.style(line, self.theme.muted));
        }

        Ok(())
    }

    /// Display key-value pairs in aligned columns
    pub fn info_section(&self, title: Option<&str>, items: &[(String, String)]) -> Result<()> {
        self.clear_if_needed()?;

        if let Some(title) = title {
            println!("{}", self.style(title, self.theme.muted));
        }

        if !items.is_empty() {
            let max_key_len = items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

            for (key, value) in items {
                // Color special values
                let colored_value = if value.contains('↑') {
                    self.apply_mixed_colors(value)
                } else if value.contains("ago") {
                    self.style(value, self.theme.muted).to_string()
                } else {
                    value.clone()
                };

                println!("  {:<width$}  {}", key, colored_value, width = max_key_len);
            }
        }

        println!();
        Ok(())
    }

    /// Print raw text with optional color
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

    /// Print line with optional color
    pub fn println(&self, text: &str, color: Option<Color>) -> Result<()> {
        self.clear_if_needed()?;

        if let Some(color) = color {
            println!("{}", self.style(text, color));
        } else {
            println!("{}", text);
        }

        Ok(())
    }

    // Private helper methods

    fn print_tree_node(&self, node: &TreeNode, prefix: &str, is_last: bool) -> Result<()> {
        // Print branch connector
        if !prefix.is_empty() {
            print!("{}", prefix);
            print!("{}", if is_last { "└─ " } else { "├─ " });
        }

        // Print node name
        print!("{}", node.name);

        // Print metadata with dots
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

            // Calculate dots
            let name_len = prefix.len() + node.name.len() + 3;
            let dots_needed = (50_usize.saturating_sub(name_len))
                .saturating_sub(visible_len(&metadata_str))
                .min(30);

            if dots_needed > 2 {
                print!(" {}", "·".repeat(dots_needed));
            }

            print!(" {}", metadata_str);
        }

        println!();

        // Print children
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

    fn apply_mixed_colors(&self, text: &str) -> String {
        // Handle text with mixed ↑ and ↓ symbols
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '↑' {
                // Color ↑ and following number green
                let mut num = String::from("↑");
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                result.push_str(&self.style(&num, self.theme.success));
            } else if ch == '↓' {
                // Color ↓ and following number yellow
                let mut num = String::from("↓");
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                result.push_str(&self.style(&num, self.theme.warning));
            } else {
                result.push(ch);
            }
        }

        result
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
                        // Default to first option on Enter
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

// Helper functions

fn supports_color() -> bool {
    // Check if stdout is a tty and supports color
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
    // Strip ANSI codes and count visible characters
    let stripped = strip_ansi_escapes::strip(s);
    String::from_utf8_lossy(&stripped).len()
}
