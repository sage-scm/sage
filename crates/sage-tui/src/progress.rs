use crate::Theme;
use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Handle for an indeterminate progress indicator
pub struct ProgressHandle {
    message: String,
    spinner: Spinner,
    use_animation: bool,
    needs_clear: Arc<AtomicBool>,
    start_time: Instant,
    active: bool,
    theme: Theme,
}

impl ProgressHandle {
    pub(crate) fn new(
        message: String,
        use_animation: bool,
        needs_clear: Arc<AtomicBool>,
        theme: Theme,
    ) -> Self {
        // Don't print a newline - just start on current line
        let mut handle = Self {
            message,
            spinner: Spinner::braille(),
            use_animation,
            needs_clear,
            start_time: Instant::now(),
            active: true,
            theme,
        };

        handle.render();
        handle
    }

    /// Update the spinner animation
    pub fn tick(&mut self) {
        if self.active && self.use_animation {
            self.render();
        }
    }

    /// Complete silently - removes the entire progress line
    pub fn done(mut self) {
        self.active = false;

        // Just clear the current line completely
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )
        .ok();

        self.needs_clear.store(false, Ordering::Relaxed);
    }

    /// Complete with visible confirmation (use sparingly)
    pub fn done_with_message(mut self, message: &str) {
        self.active = false;

        // Replace the current progress line with the message
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            SetForegroundColor(self.theme.muted),
            Print("  "),
            Print(&self.message),
            Print(": "),
            Print(message),
            ResetColor,
            Print("\n"),
        )
        .ok();

        self.needs_clear.store(false, Ordering::Relaxed);
    }

    /// Complete with custom message
    pub fn finish(mut self, suffix: &str) {
        self.active = false;

        // Replace current line with final message
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print(format!("  {} {}\n", self.message, suffix)),
        )
        .ok();

        self.needs_clear.store(false, Ordering::Relaxed);
    }

    /// Fail silently or with error message
    pub fn fail(mut self, error: &str) {
        self.active = false;

        if error.is_empty() {
            // Just clear the progress line
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
            )
            .ok();
        } else {
            // Replace with error message
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                SetForegroundColor(self.theme.muted),
                Print("  "),
                Print(&self.message),
                Print(": "),
                SetForegroundColor(self.theme.error),
                Print(error),
                ResetColor,
                Print("\n"),
            )
            .ok();
        }

        self.needs_clear.store(false, Ordering::Relaxed);
    }

    /// Update the message
    pub fn set_message(&mut self, message: String) {
        self.message = message;
        self.render();
    }

    fn render(&mut self) {
        if !self.use_animation {
            execute!(
                io::stdout(),
                cursor::SavePosition,
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                Print(format!("  {}...", self.message)),
                cursor::RestorePosition,
            )
            .ok();
            io::stdout().flush().ok();
            self.needs_clear.store(true, Ordering::Relaxed);
            return;
        }

        let frame = self.spinner.next();

        execute!(
            io::stdout(),
            cursor::SavePosition,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print("  "),
            Print(&self.message),
            Print("... "),
            SetForegroundColor(self.theme.muted),
            Print(frame),
            ResetColor,
            cursor::RestorePosition,
        )
        .ok();

        io::stdout().flush().ok();
        self.needs_clear.store(true, Ordering::Relaxed);
    }
}

impl Drop for ProgressHandle {
    fn drop(&mut self) {
        if self.active {
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                Print("  "), // Indent
                Print(&self.message),
                Print(" "),
                SetForegroundColor(self.theme.warning),
                Print("!"),
                ResetColor,
                Print(" interrupted\n"),
            )
            .ok();

            self.needs_clear.store(false, Ordering::Relaxed);
        }
    }
}

/// Handle for a determinate progress bar
pub struct ProgressBar {
    message: String,
    total: u64,
    current: u64,
    use_animation: bool,
    needs_clear: Arc<AtomicBool>,
    start_time: Instant,
    active: bool,
}

impl ProgressBar {
    pub(crate) fn new(
        message: String,
        total: u64,
        use_animation: bool,
        needs_clear: Arc<AtomicBool>,
    ) -> Self {
        let mut bar = Self {
            message,
            total,
            current: 0,
            use_animation,
            needs_clear,
            start_time: Instant::now(),
            active: true,
        };

        bar.render();
        bar
    }

    /// Update progress
    pub fn set(&mut self, current: u64) {
        self.current = current.min(self.total);
        self.render();
    }

    /// Increment progress by amount
    pub fn inc(&mut self, amount: u64) {
        self.set(self.current + amount);
    }

    /// Update message
    pub fn set_message(&mut self, message: String) {
        self.message = message;
        self.render();
    }

    /// Complete the progress bar
    pub fn finish(mut self) {
        self.active = false;
        self.current = self.total;

        // Clear the progress line and print done message
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )
        .ok();

        println!("  {} done", self.message);
        self.needs_clear.store(false, Ordering::Relaxed);
    }

    fn render(&self) {
        execute!(
            io::stdout(),
            cursor::SavePosition,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )
        .ok();

        let percent = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u8
        } else {
            0
        };

        // Build progress bar with indent
        let bar_width = 20;
        let filled = (bar_width * percent as usize / 100).min(bar_width);
        let empty = bar_width - filled;

        print!(
            "  {} [{}{}] {}%",
            self.message,
            "█".repeat(filled),
            "░".repeat(empty),
            percent
        );

        // Add speed/ETA if enough time has passed
        let elapsed = self.start_time.elapsed();
        if elapsed > Duration::from_secs(2) && self.current > 0 && self.current < self.total {
            let rate = self.current as f64 / elapsed.as_secs_f64();
            let remaining = (self.total - self.current) as f64 / rate;

            if remaining < 60.0 {
                print!(" · {}s", remaining as u64);
            } else {
                print!(" · {}m", (remaining / 60.0) as u64);
            }
        }

        execute!(io::stdout(), cursor::RestorePosition).ok();
        io::stdout().flush().ok();
        self.needs_clear.store(true, Ordering::Relaxed);
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.active {
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
            )
            .ok();

            println!("  {} interrupted", self.message);
            self.needs_clear.store(false, Ordering::Relaxed);
        }
    }
}

/// Spinner animation frames
struct Spinner {
    frames: Vec<&'static str>,
    current: usize,
}

impl Spinner {
    fn braille() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current: 0,
        }
    }

    fn dots() -> Self {
        Self {
            frames: vec!["   ", "·  ", "·· ", "···", " ··", "  ·", "   "],
            current: 0,
        }
    }

    fn wave() -> Self {
        Self {
            frames: vec!["◡◡◡", "◠◡◡", "◡◠◡", "◡◡◠", "◡◠◡", "◠◡◡", "◡◡◡"],
            current: 0,
        }
    }

    fn next(&mut self) -> &'static str {
        let frame = self.frames[self.current];
        self.current = (self.current + 1) % self.frames.len();
        frame
    }
}
