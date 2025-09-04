use anyhow::{Error, Result};
use crossterm::{
    execute,
    style::{Attribute, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use std::io;

use crate::Theme;

pub struct ErrorDisplay {
    theme: Theme,
}

impl ErrorDisplay {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn show(&self, error: &Error) -> Result<()> {
        self.print_main_error(error)?;
        self.print_error_chain(error)?;
        self.print_suggestions(error)?;
        Ok(())
    }

    pub fn show_with_context(&self, error: &Error, context: &str) -> Result<()> {
        println!();
        self.print_context(context)?;
        self.show(error)?;
        Ok(())
    }

    fn print_main_error(&self, error: &Error) -> Result<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(self.theme.error),
            SetAttribute(Attribute::Bold),
            Print("  ✗ Error: "),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;

        println!("{}", error);
        Ok(())
    }

    fn print_error_chain(&self, error: &Error) -> Result<()> {
        let chain: Vec<_> = error.chain().skip(1).collect();

        if !chain.is_empty() {
            println!();
            for (i, cause) in chain.iter().enumerate() {
                let prefix = if i == 0 {
                    "  Caused by:"
                } else {
                    "           "
                };

                execute!(
                    io::stdout(),
                    SetForegroundColor(self.theme.muted),
                    Print(prefix),
                    ResetColor
                )?;

                println!(" {}", cause);
            }
        }

        Ok(())
    }

    fn print_context(&self, context: &str) -> Result<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(self.theme.info),
            Print("  Context: "),
            ResetColor
        )?;
        println!("{}", context);
        Ok(())
    }

    fn print_suggestions(&self, error: &Error) -> Result<()> {
        let suggestions = self.get_suggestions(error);

        if !suggestions.is_empty() {
            println!();
            execute!(
                io::stdout(),
                SetForegroundColor(self.theme.warning),
                Print("  Suggestions:"),
                ResetColor
            )?;
            println!();

            for suggestion in suggestions {
                println!("    • {}", suggestion);
            }
        }

        Ok(())
    }

    fn get_suggestions(&self, error: &Error) -> Vec<&'static str> {
        let error_str = error.to_string().to_lowercase();
        let mut suggestions = Vec::new();

        if error_str.contains("permission denied") {
            suggestions.push("Check file permissions");
            suggestions.push("Try running with appropriate privileges");
        }

        if error_str.contains("not found") {
            if error_str.contains("file") || error_str.contains("directory") {
                suggestions.push("Check if the path exists");
                suggestions.push("Verify the spelling of the file or directory name");
            } else if error_str.contains("command") || error_str.contains("executable") {
                suggestions.push("Check if the program is installed");
                suggestions.push("Verify the command is in your PATH");
            }
        }

        if error_str.contains("connection") || error_str.contains("network") {
            suggestions.push("Check your internet connection");
            suggestions.push("Verify the remote server is accessible");
            suggestions.push("Check firewall settings");
        }

        if error_str.contains("timeout") {
            suggestions.push("Try the operation again");
            suggestions.push("Check if the service is responding");
        }

        if error_str.contains("conflict") {
            suggestions.push("Resolve conflicts manually");
            suggestions.push("Consider using --force if appropriate");
        }

        if error_str.contains("uncommitted") || error_str.contains("dirty") {
            suggestions.push("Commit or stash your changes");
            suggestions.push("Use --force to override (dangerous)");
        }

        if error_str.contains("authentication") || error_str.contains("unauthorized") {
            suggestions.push("Check your credentials");
            suggestions.push("Verify your access tokens are valid");
        }

        if error_str.contains("space") || error_str.contains("disk") {
            suggestions.push("Free up disk space");
            suggestions.push("Check available storage");
        }

        suggestions
    }
}

pub fn format_error_compact(error: &Error) -> String {
    let mut result = error.to_string();

    let causes: Vec<_> = error.chain().skip(1).collect();
    if !causes.is_empty() {
        result.push_str(" (");
        result.push_str(&causes[0].to_string());
        result.push(')');
    }

    result
}
