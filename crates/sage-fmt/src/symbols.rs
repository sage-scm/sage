#[allow(dead_code)]
pub struct Symbols {
    pub check: &'static str,
    pub cross: &'static str,
    pub warn: &'static str,
    pub dot: &'static str,
    pub arrow: &'static str,
    pub bullet: &'static str,
    pub lightning: &'static str,
    pub up: &'static str,
    pub down: &'static str,
    pub stage_filled: &'static str,
    pub stage_empty: &'static str,
}

impl Symbols {
    pub fn new(ascii: bool) -> Self {
        if ascii {
            Self {
                check: "OK",
                cross: "X",
                warn: "!",
                dot: ".",
                arrow: "->",
                bullet: "-",
                lightning: "!",
                up: "^",
                down: "v",
                stage_filled: "*",
                stage_empty: "o",
            }
        } else {
            Self {
                check: "✓",
                cross: "✗",
                warn: "!",
                dot: "·",
                arrow: "→",
                bullet: "•",
                lightning: "⚡",
                up: "↑",
                down: "↓",
                stage_filled: "●",
                stage_empty: "○",
            }
        }
    }
}

pub fn ascii_mode() -> bool {
    // Explicit override to force ASCII safe output
    if std::env::var("SAGE_FMT_ASCII").is_ok() {
        return true;
    }
    // Fall back to very conservative detection
    matches!(std::env::var("TERM"), Ok(term) if term == "dumb")
}
