use crate::Theme;
use crossterm::{
    cursor, execute,
    style::{Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub struct ProgressHandle {
    message: Arc<Mutex<String>>,
    use_animation: bool,
    needs_clear: Arc<AtomicBool>,
    active: Arc<AtomicBool>,
    theme: Theme,
    last_line_blank: Arc<AtomicBool>,
    tick_handle: Option<std::thread::JoinHandle<()>>,
}

impl ProgressHandle {
    pub(crate) fn new(
        message: String,
        use_animation: bool,
        needs_clear: Arc<AtomicBool>,
        last_line_blank: Arc<AtomicBool>,
        theme: Theme,
    ) -> Self {
        let message_arc = Arc::new(Mutex::new(message));
        let active = Arc::new(AtomicBool::new(true));

        let mut handle = Self {
            message: message_arc.clone(),
            use_animation,
            needs_clear: needs_clear.clone(),
            active: active.clone(),
            theme: theme.clone(),
            last_line_blank: last_line_blank.clone(),
            tick_handle: None,
        };

        if use_animation {
            let theme_for_thread = theme.clone();
            let needs_clear_for_thread = needs_clear.clone();
            let active_for_thread = active.clone();
            let message_for_thread = message_arc.clone();

            let join = thread::spawn(move || {
                let mut spinner = Spinner::wave();
                while active_for_thread.load(Ordering::Relaxed) {
                    let msg = message_for_thread
                        .lock()
                        .ok()
                        .map(|m| m.clone())
                        .unwrap_or_default();

                    execute!(
                        io::stdout(),
                        cursor::MoveToColumn(0),
                        terminal::Clear(ClearType::CurrentLine),
                        Print("  "),
                        Print(&msg),
                        Print(" "),
                        SetForegroundColor(theme_for_thread.muted),
                        Print(spinner.next()),
                        ResetColor,
                    )
                    .ok();

                    io::stdout().flush().ok();
                    needs_clear_for_thread.store(true, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(120));
                }
            });

            handle.tick_handle = Some(join);
            handle.render();
        } else {
            handle.render();
        }

        handle
    }

    pub fn tick(&mut self) {
        if !self.use_animation {
            self.render();
        }
    }

    pub fn done(mut self) {
        self.active.store(false, Ordering::Relaxed);

        if let Some(j) = self.tick_handle.take() {
            let _ = j.join();
        }

        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )
        .ok();

        self.needs_clear.store(false, Ordering::Relaxed);
    }

    pub fn done_with_message(mut self, message: &str) {
        self.active.store(false, Ordering::Relaxed);

        if let Some(j) = self.tick_handle.take() {
            let _ = j.join();
        }

        let msg = self
            .message
            .lock()
            .ok()
            .map(|m| m.clone())
            .unwrap_or_default();
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            SetForegroundColor(self.theme.muted),
            Print("  "),
            Print(&msg),
            Print(": "),
            Print(message),
            ResetColor,
            Print("\n"),
        )
        .ok();

        self.last_line_blank.store(false, Ordering::Relaxed);
        self.needs_clear.store(false, Ordering::Relaxed);
    }

    pub fn finish(mut self, suffix: &str) {
        self.active.store(false, Ordering::Relaxed);

        if let Some(j) = self.tick_handle.take() {
            let _ = j.join();
        }

        let msg = self
            .message
            .lock()
            .ok()
            .map(|m| m.clone())
            .unwrap_or_default();
        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print(format!("  {} {}\n", msg, suffix)),
        )
        .ok();

        self.last_line_blank.store(false, Ordering::Relaxed);
        self.needs_clear.store(false, Ordering::Relaxed);
    }

    pub fn fail(mut self, error: &str) {
        self.active.store(false, Ordering::Relaxed);

        if let Some(j) = self.tick_handle.take() {
            let _ = j.join();
        }

        if error.is_empty() {
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
            )
            .ok();
        } else {
            let msg = self
                .message
                .lock()
                .ok()
                .map(|m| m.clone())
                .unwrap_or_default();
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                SetForegroundColor(self.theme.muted),
                Print("  "),
                Print(&msg),
                Print(": "),
                SetForegroundColor(self.theme.error),
                Print(error),
                ResetColor,
                Print("\n"),
            )
            .ok();
            self.last_line_blank.store(false, Ordering::Relaxed);
        }

        self.needs_clear.store(false, Ordering::Relaxed);
    }

    pub fn set_message(&mut self, message: String) {
        if let Ok(mut m) = self.message.lock() {
            *m = message;
        }
        if !self.use_animation {
            self.render();
        }
    }

    fn render(&mut self) {
        let msg = self
            .message
            .lock()
            .ok()
            .map(|m| m.clone())
            .unwrap_or_default();
        if !self.use_animation {
            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                Print(format!("  {}", msg)),
            )
            .ok();
            io::stdout().flush().ok();
            self.needs_clear.store(true, Ordering::Relaxed);
            return;
        }

        execute!(
            io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print("  "),
            Print(&msg),
            Print(" "),
        )
        .ok();

        io::stdout().flush().ok();
        self.needs_clear.store(true, Ordering::Relaxed);
    }
}

impl Drop for ProgressHandle {
    fn drop(&mut self) {
        if self.active.load(Ordering::Relaxed) {
            self.active.store(false, Ordering::Relaxed);
            if let Some(j) = self.tick_handle.take() {
                let _ = j.join();
            }

            execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                Print("  "),
                Print(
                    &self
                        .message
                        .lock()
                        .ok()
                        .map(|m| m.clone())
                        .unwrap_or_default()
                ),
                Print(" "),
                SetForegroundColor(self.theme.warning),
                Print("!"),
                ResetColor,
                Print(" interrupted\n"),
            )
            .ok();
            self.last_line_blank.store(false, Ordering::Relaxed);
            self.needs_clear.store(false, Ordering::Relaxed);
        }
    }
}

pub struct ProgressBar {
    message: String,
    total: u64,
    current: u64,
    needs_clear: Arc<AtomicBool>,
    start_time: Instant,
    active: bool,
}

impl ProgressBar {
    pub(crate) fn new(message: String, total: u64, needs_clear: Arc<AtomicBool>) -> Self {
        let bar = Self {
            message,
            total,
            current: 0,
            needs_clear,
            start_time: Instant::now(),
            active: true,
        };

        bar.render();
        bar
    }

    pub fn set(&mut self, current: u64) {
        self.current = current.min(self.total);
        self.render();
    }

    pub fn inc(&mut self, amount: u64) {
        self.set(self.current + amount);
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
        self.render();
    }

    pub fn finish(mut self) {
        self.active = false;
        self.current = self.total;
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

struct Spinner {
    frames: Vec<&'static str>,
    current: usize,
}

impl Spinner {
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
