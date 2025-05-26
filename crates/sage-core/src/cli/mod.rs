use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

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
}

impl SpinnerStep {
    pub fn finish_success(self, message: &str, detail: Option<&str>) {
        self.pb.finish_and_clear();
        let final_msg = if let Some(detail) = detail {
            format!(
                "●   {} {} {}",
                message,
                style("✔").green(),
                style(detail).dim()
            )
        } else {
            format!("●   {} {}", message, style("✔").green())
        };
        println!("{}", final_msg);
    }

    pub fn finish_success_with_emoji(
        self,
        message: &str,
        detail: Option<&str>,
        custom_emoji: &str,
    ) {
        self.pb.finish_and_clear();
        let final_msg = if let Some(detail) = detail {
            format!("{}   {} {}", custom_emoji, message, style(detail).dim())
        } else {
            format!("{}   {}", custom_emoji, message)
        };
        println!("{}", final_msg);
    }

    pub fn finish_error(self, message: &str, error: &str) {
        self.pb.finish_and_clear();
        let final_msg = format!(
            "●   {} {} {}",
            message,
            style("✗").red(),
            style(error).red().dim()
        );
        println!("{}", final_msg);
    }

    pub fn update_message(&self, message: &str) {
        self.pb.set_message(format!("{}", style(message).dim()));
    }
}

pub struct CliOutput {
    term: Term,
    start_time: Instant,
}

impl CliOutput {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
            start_time: Instant::now(),
        }
    }

    pub fn header(&self, subcommand: &str) {
        let header = format!(
            "🌿  {} — {}",
            style("sage").bold().cyan(),
            style(subcommand).dim()
        );
        println!("{}\n", header);
    }

    // Create a spinner for long-running operations
    pub fn spinner(&self, message: &str) -> SpinnerStep {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.yellow} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("{}", style(message).dim()));
        pb.enable_steady_tick(Duration::from_millis(100));

        SpinnerStep { pb }
    }

    pub fn step_start(&self, message: &str) {
        print!("{}   {}", style("●").yellow(), style(message).dim());
        let _ = self.term.flush();
    }

    pub fn step_success(&self, message: &str, detail: Option<&str>) {
        print!("\r●   {}", message);

        print!(" {}", style("✔").green());

        if let Some(detail) = detail {
            print!(" {}", style(detail).dim());
        }

        println!();
    }

    pub fn step_success_with_emoji(&self, message: &str, detail: Option<&str>, custom_emoji: &str) {
        print!("\r{}   {}", custom_emoji, message);

        if let Some(detail) = detail {
            print!(" {}", style(detail).dim());
        }
        println!();
    }

    pub fn warning(&self, message: &str) {
        println!("{}   {}\n", style("⚠️").yellow(), style(message).yellow());
    }

    pub fn step_update(&self, message: &str) {
        print!("\r{}   {}", style("●").yellow(), style(message).dim());
        let _ = self.term.flush();
    }

    pub fn step_error(&self, message: &str, error: &str) {
        println!(
            "\r●   {} {} {}",
            message,
            style("✗").red(),
            style(error).red().dim()
        );
    }

    pub fn summary(&self) {
        let elapsed = self.start_time.elapsed();
        let duration_str = format_duration(elapsed);
        println!(
            "\n{}   Done in {}",
            style("🎉").bold(),
            style(duration_str).green().bold()
        );
    }

    // More advanced: boxed output
    pub fn boxed_summary(&self, title: &str, items: &[(&str, &str)]) {
        let width = 60;
        println!("\n┌{}┐", "─".repeat(width - 2));
        println!(
            "│ {} │",
            style(format!("{:^width$}", title, width = width - 4)).bold()
        );
        println!("├{}┤", "─".repeat(width - 2));

        for (label, value) in items {
            println!("│ {:<20} {} │", style(label).dim(), style(value).green());
        }

        println!("└{}┘", "─".repeat(width - 2));
    }
}
