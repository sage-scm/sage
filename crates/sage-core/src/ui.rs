use std::io::{self, Write};

pub struct Ui {
    pub json_mode: bool,
    pub no_color: bool,
    pub width: usize,
}

impl Ui {
    pub fn new(json_mode: bool, no_color: bool) -> Self {
        let width = term_width().unwrap_or(100).clamp(80, 140);
        Self {
            json_mode,
            no_color,
            width,
        }
    }

    pub fn header(&self, kvs: &[(&str, &str)]) {
        if self.json_mode {
            return;
        }
        let left = kvs
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{left}");
        println!("{}", "─".repeat(self.width));
    }

    pub fn section<T: AsRef<str>>(&self, title: T) {
        if self.json_mode {
            return;
        }
        println!("{}", title.as_ref());
    }

    pub fn line<T: AsRef<str>>(&self, text: T) {
        if self.json_mode {
            return;
        }
        println!("{}", text.as_ref());
    }

    pub fn bullet<T: AsRef<str>>(&self, text: T) {
        if self.json_mode {
            return;
        }
        println!("  • {}", text.as_ref());
    }

    pub fn table(&self, headers: &[&str], rows: &[Vec<String>]) {
        if self.json_mode {
            return;
        }
        let widths = compute_widths(self.width, headers, rows);
        // headers
        for (i, h) in headers.iter().enumerate() {
            print!("{:width$}", clip(h, widths[i]), width = widths[i]);
            if i < headers.len() - 1 {
                print!("  ");
            }
        }
        println!();
        // underline
        for (i, h) in headers.iter().enumerate() {
            print!("{:-<width$}", "", width = widths[i].min(h.len()).max(1));
            if i < headers.len() - 1 {
                print!("  ");
            }
        }
        println!();
        // rows
        for r in rows {
            for (i, cell) in r.iter().enumerate() {
                print!("{:width$}", clip(cell, widths[i]), width = widths[i]);
                if i < headers.len() - 1 {
                    print!("  ");
                }
            }
            println!();
        }
        println!();
    }

    /// Confirm with [y/N/v]. `on_verbose` prints the underlying script.
    /// Returns Ok(true) to proceed, Ok(false) to abort.
    pub fn confirm<F>(&self, prompt: &str, on_verbose: F) -> io::Result<bool>
    where
        F: Fn(),
    {
        if self.json_mode {
            // In JSON mode, never proceed unless an explicit --yes flag was given at the caller.
            return Ok(false);
        }
        print!("{prompt} [y/N/v] ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let ans = buf.trim().to_lowercase();
        match ans.as_str() {
            "y" | "yes" => Ok(true),
            "v" => {
                on_verbose();
                self.confirm(prompt, on_verbose)
            }
            _ => Ok(false),
        }
    }

    pub fn undo_line(&self, undo_id: &str) {
        if self.json_mode {
            return;
        }
        println!();
        println!("Undo");
        println!("  sage undo {undo_id}");
    }
}

fn term_width() -> Option<usize> {
    #[cfg(any(unix, windows))]
    {
        use terminal_size::{Width, terminal_size};
        if let Some((Width(w), _)) = terminal_size() {
            return Some(w as usize);
        }
        None
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn compute_widths(total: usize, headers: &[&str], rows: &[Vec<String>]) -> Vec<usize> {
    let cols = headers.len();
    let mut widths = vec![0usize; cols];
    for (i, h) in headers.iter().enumerate() {
        widths[i] = widths[i].max(h.len());
    }
    for r in rows {
        for (i, cell) in r.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }
    // pad between columns (2 spaces each)
    let mut sum: usize = widths.iter().sum::<usize>() + (cols.saturating_sub(1)) * 2;
    if sum > total {
        // shrink proportionally
        for w in &mut widths {
            let new_w = (*w as f32 * (total as f32 / sum as f32)).floor() as usize;
            *w = new_w.max(8);
        }
    }
    // ensure final width constraint
    sum = widths.iter().sum::<usize>() + (cols.saturating_sub(1)) * 2;
    while sum > total {
        let i = widths
            .iter()
            .enumerate()
            .max_by_key(|(_, w)| **w)
            .map(|(i, _)| i)
            .unwrap();
        widths[i] = widths[i].saturating_sub(1);
        sum -= 1;
    }
    widths
}

fn clip(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i + 1 >= width {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}
