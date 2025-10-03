use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::{
    io::IsTerminal,
    sync::{Arc, atomic::AtomicBool},
};

mod symbols;
mod theme;
use symbols::{Symbols, ascii_mode as symbols_ascii_mode};
pub use theme::Theme;

pub struct Console {
    theme: Theme,
    use_color: bool,
    is_ci: bool,
    needs_clear: Arc<AtomicBool>,
    last_line_blank: Arc<AtomicBool>,
    symbols: Symbols,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Success,
    Error,
    Warning,
    Info,
}

#[allow(dead_code)]
impl Console {
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            use_color: supports_color(),
            is_ci: is_ci_environment(),
            needs_clear: Arc::new(AtomicBool::new(false)),
            last_line_blank: Arc::new(AtomicBool::new(false)),
            symbols: Symbols::new(symbols_ascii_mode()),
        }
    }

    pub fn header(&self, command: &str) -> Result<()> {
        println!("sage {}", self.style(command, self.theme.muted));
        Ok(())
    }

    pub fn message(&self, msg_type: MessageType, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }

        let (symbol, color) = match msg_type {
            MessageType::Success => (self.symbols.check, self.theme.success),
            MessageType::Error => (self.symbols.cross, self.theme.error),
            MessageType::Warning => (self.symbols.warn, self.theme.warning),
            MessageType::Info => (self.symbols.dot, self.theme.info),
        };

        println!("  {} {}", self.style(symbol, color), text);
        Ok(())
    }

    fn style(&self, text: &str, color: Color) -> String {
        if self.use_color && !self.is_ci {
            format!("{}", text.with(color))
        } else {
            text.to_string()
        }
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn supports_color() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    if let Ok(force_color) = std::env::var("FORCE_COLOR") {
        return !force_color.is_empty()
            && force_color != "0"
            && force_color.to_lowercase() != "false";
    }

    if !std::io::stdout().is_terminal() {
        return false;
    }

    match std::env::var("TERM") {
        Ok(term) => {
            if term == "dumb" || term.is_empty() {
                return false;
            }
            true
        }
        Err(_) => false,
    }
}

fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok()
    || std::env::var("GITHUB_ACTIONS").is_ok()
    || std::env::var("GITLAB_CI").is_ok()
    || std::env::var("CIRCLECI").is_ok()
    || std::env::var("JENKINS_URL").is_ok()
    || std::env::var("BUILDKITE").is_ok()
    || std::env::var("TRAVIS").is_ok()
    || std::env::var("APPVEYOR").is_ok()
    || std::env::var("AZURE_PIPELINES").is_ok()
    || std::env::var("TEAMCITY_VERSION").is_ok()
    || std::env::var("BAMBOO_BUILD_NUMBER").is_ok()
    // Also check if stdout is not a terminal as a fallback
    || !std::io::stdout().is_terminal()
}
