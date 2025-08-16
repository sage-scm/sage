use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

/// Global CLI configuration
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub json: bool,
    pub no_color: bool,
}

impl GlobalConfig {
    pub fn new(json: bool, no_color: bool) -> Self {
        Self { json, no_color }
    }
}

fn format_duration(duration: Duration) -> String {
    let total_ms = duration.as_millis();
    let total_secs = duration.as_secs();

    if total_secs >= 60 {
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        if seconds == 0 {
            format!("{}m", minutes)
        } else {
            format!("{}m {}s", minutes, seconds)
        }
    } else if total_secs >= 1 {
        let secs = duration.as_secs_f64();
        format!("{:.3}s", secs)
    } else {
        format!("{}ms", total_ms)
    }
}

pub struct SpinnerStep {
    pb: ProgressBar,
    config: GlobalConfig,
}

impl SpinnerStep {
    fn style_text<'a>(&self, text: &'a str) -> console::StyledObject<&'a str> {
        if self.config.no_color {
            style(text).force_styling(false)
        } else {
            style(text)
        }
    }

    pub fn finish_success(self, message: &str, detail: Option<&str>) {
        if self.config.json {
            return;
        }
        
        self.pb.finish_and_clear();
        if let Ok(term) = Term::stdout().clear_line() {
            let _ = term;
        }
        let final_msg = if let Some(detail) = detail {
            format!(
                "●   {} {} {}",
                message,
                self.style_text("✔").green(),
                self.style_text(detail).dim()
            )
        } else {
            format!("●   {} {}", message, self.style_text("✔").green())
        };
        println!("{}", final_msg);
    }

    pub fn finish_success_with_emoji(
        self,
        message: &str,
        detail: Option<&str>,
        custom_emoji: &str,
    ) {
        if self.config.json {
            return;
        }
        
        self.pb.finish_and_clear();
        if let Ok(term) = Term::stdout().clear_line() {
            let _ = term;
        }
        let final_msg = if let Some(detail) = detail {
            format!("{}   {} {}", custom_emoji, message, self.style_text(detail).dim())
        } else {
            format!("{}   {}", custom_emoji, message)
        };
        println!("{}", final_msg);
    }

    pub fn finish_error(self, message: &str, error: &str) {
        if self.config.json {
            return;
        }
        
        self.pb.finish_and_clear();
        if let Ok(term) = Term::stdout().clear_line() {
            let _ = term;
        }
        let final_msg = format!(
            "●   {} {} {}",
            message,
            self.style_text("✗").red(),
            self.style_text(error).red().dim()
        );
        println!("{}", final_msg);
    }

    pub fn update_message(&self, message: &str) {
        if self.config.json {
            return;
        }
        self.pb.set_message(format!("{}", self.style_text(message).dim()));
    }
}

pub struct CliOutput {
    term: Term,
    start_time: Instant,
    config: GlobalConfig,
}

impl CliOutput {
    pub fn new(config: GlobalConfig) -> Self {
        Self {
            term: Term::stdout(),
            start_time: Instant::now(),
            config,
        }
    }

    fn style_text<'a>(&self, text: &'a str) -> console::StyledObject<&'a str> {
        if self.config.no_color {
            style(text).force_styling(false)
        } else {
            style(text)
        }
    }

    pub fn header(&self, subcommand: &str) {
        if self.config.json {
            return;
        }
        let header = format!(
            "🌿  {} — {}",
            self.style_text("sage").bold().cyan(),
            self.style_text(subcommand).dim()
        );
        println!("{}\n", header);
    }

    pub fn spinner(&self, message: &str) -> SpinnerStep {
        if self.config.json {
            return SpinnerStep { 
                pb: ProgressBar::hidden(),
                config: self.config.clone(),
            };
        }
        
        let pb = ProgressBar::new_spinner();
        let template = if self.config.no_color {
            "{spinner} {msg}"
        } else {
            "{spinner:.yellow} {msg}"
        };
        
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template(template)
                .unwrap(),
        );
        pb.set_message(format!("{}", self.style_text(message).dim()));
        pb.enable_steady_tick(Duration::from_millis(100));

        SpinnerStep { 
            pb,
            config: self.config.clone(),
        }
    }

    pub fn step_start(&self, message: &str) {
        if self.config.json {
            return;
        }
        print!("{}   {}", self.style_text("●").yellow(), self.style_text(message).dim());
        let _ = self.term.flush();
    }

    pub fn step_success(&self, message: &str, detail: Option<&str>) {
        if self.config.json {
            return;
        }
        
        self.term.clear_line().unwrap_or(());

        print!("\r●   {}", message);

        print!(" {}", self.style_text("✔").green());

        if let Some(detail) = detail {
            print!(" {}", self.style_text(detail).dim());
        }

        println!();
    }

    pub fn step_success_with_emoji(&self, message: &str, detail: Option<&str>, custom_emoji: &str) {
        if self.config.json {
            return;
        }
        
        self.term.clear_line().unwrap_or(());

        print!("\r{}  {}", custom_emoji, message);

        if let Some(detail) = detail {
            print!(" {}", self.style_text(detail).dim());
        }
        println!();
    }

    pub fn warning(&self, message: &str) {
        if self.config.json {
            return;
        }
        println!("{}   {}\n", self.style_text("⚠️").yellow(), self.style_text(message).yellow());
    }

    pub fn step_update(&self, message: &str) {
        if self.config.json {
            return;
        }
        self.term.clear_line().unwrap_or(());
        print!("\r{}   {}", self.style_text("●").yellow(), self.style_text(message).dim());
        let _ = self.term.flush();
    }

    pub fn step_error(&self, message: &str, error: &str) {
        if self.config.json {
            return;
        }
        self.term.clear_line().unwrap_or(());
        println!(
            "\r●   {} {} {}",
            message,
            self.style_text("✗").red(),
            self.style_text(error).red().dim()
        );
    }

    pub fn summary(&self) {
        if self.config.json {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let duration_str = format_duration(elapsed);
        println!(
            "\n{}   Done in {}",
            self.style_text("🎉").bold(),
            self.style_text(&duration_str).green().bold()
        );
    }

    pub fn boxed_summary(&self, title: &str, items: &[(&str, &str)]) {
        if self.config.json {
            return;
        }
        
        let width = 60;
        println!("\n┌{}┐", "─".repeat(width - 2));
        println!(
            "│ {} │",
            self.style_text(&format!("{:^width$}", title, width = width - 4)).bold()
        );
        println!("├{}┤", "─".repeat(width - 2));

        for (label, value) in items {
            println!("│ {:<20} {} │", self.style_text(label).dim(), self.style_text(value).green());
        }

        println!("└{}┘", "─".repeat(width - 2));
    }

    /// Output data in JSON format
    pub fn json_output<T: serde::Serialize>(&self, data: &T) -> anyhow::Result<()> {
        if self.config.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        Ok(())
    }

    /// Check if JSON mode is enabled
    pub fn is_json_mode(&self) -> bool {
        self.config.json
    }
}