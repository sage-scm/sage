use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::Theme;

pub struct MultiSelect<'a, T> {
    items: Vec<T>,
    selected: Vec<bool>,
    cursor: usize,
    prompt: String,
    display: Box<dyn Fn(&T) -> String + 'a>,
    theme: Theme,
    use_color: bool,
    max_height: usize,
    scroll_offset: usize,
}

impl<'a, T> MultiSelect<'a, T> {
    pub fn new(prompt: impl Into<String>, items: Vec<T>) -> Self {
        let count = items.len();
        Self {
            items,
            selected: vec![false; count],
            cursor: 0,
            prompt: prompt.into(),
            display: Box::new(|_| String::from("(item)")),
            theme: Theme::default(),
            use_color: supports_color(),
            max_height: 10,
            scroll_offset: 0,
        }
    }

    pub fn with_display<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> String + 'a,
    {
        self.display = Box::new(f);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn preselected(mut self, indices: &[usize]) -> Self {
        for &idx in indices {
            if idx < self.selected.len() {
                self.selected[idx] = true;
            }
        }
        self
    }

    pub fn max_height(mut self, height: usize) -> Self {
        self.max_height = height.max(3);
        self
    }

    pub fn run(mut self) -> Result<Vec<usize>> {
        if self.items.is_empty() {
            return Ok(Vec::new());
        }

        terminal::enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide, cursor::SavePosition)?;

        let result = self.run_loop();

        execute!(
            io::stdout(),
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown),
            cursor::Show
        )?;
        terminal::disable_raw_mode()?;

        result
    }

    fn run_loop(&mut self) -> Result<Vec<usize>> {
        loop {
            self.render()?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        return Ok(self
                            .selected
                            .iter()
                            .enumerate()
                            .filter_map(|(i, &sel)| if sel { Some(i) } else { None })
                            .collect());
                    }
                    KeyCode::Esc => return Ok(Vec::new()),
                    KeyCode::Up => self.move_cursor(-1),
                    KeyCode::Down => self.move_cursor(1),
                    KeyCode::Char(' ') => {
                        self.selected[self.cursor] = !self.selected[self.cursor];
                        self.move_cursor(1);
                    }
                    KeyCode::Char('a') => {
                        let all_selected = self.selected.iter().all(|&s| s);
                        self.selected.iter_mut().for_each(|s| *s = !all_selected);
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Vec::new());
                    }
                    _ => {}
                }
            }
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        let new_cursor = if delta < 0 {
            self.cursor.saturating_sub(delta.abs() as usize)
        } else {
            (self.cursor + delta as usize).min(self.items.len() - 1)
        };

        self.cursor = new_cursor;
        self.update_scroll();
    }

    fn update_scroll(&mut self) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + self.max_height {
            self.scroll_offset = self.cursor - self.max_height + 1;
        }
    }

    fn render(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        println!("  {}", self.prompt);

        let selected_count = self.selected.iter().filter(|&&s| s).count();
        println!(
            "  {} selected",
            self.style(&selected_count.to_string(), self.theme.muted)
        );
        println!();

        let visible_end = (self.scroll_offset + self.max_height).min(self.items.len());

        for i in self.scroll_offset..visible_end {
            let is_cursor = i == self.cursor;
            let is_selected = self.selected[i];

            print!("  ");

            if is_cursor {
                print!("{}", self.style(">", self.theme.primary));
            } else {
                print!(" ");
            }

            if is_selected {
                print!(" {}", self.style("[✓]", self.theme.success));
            } else {
                print!(" [ ]");
            }

            let display_text = (self.display)(&self.items[i]);
            if is_cursor {
                println!(" {}", self.style(&display_text, self.theme.primary));
            } else {
                println!(" {}", display_text);
            }
        }

        if self.scroll_offset > 0 || visible_end < self.items.len() {
            println!();
            if self.scroll_offset > 0 {
                println!("  {}", self.style("↑ more above", self.theme.muted));
            }
            if visible_end < self.items.len() {
                println!("  {}", self.style("↓ more below", self.theme.muted));
            }
        }

        println!();
        println!(
            "  {}",
            self.style(
                "Space Select • a Toggle All • Enter Confirm • Esc Cancel",
                self.theme.muted
            )
        );

        io::stdout().flush()?;
        Ok(())
    }

    fn style(&self, text: &str, color: Color) -> String {
        if self.use_color {
            format!("{}", text.with(color))
        } else {
            text.to_string()
        }
    }
}

fn supports_color() -> bool {
    atty::is(atty::Stream::Stdout)
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}
