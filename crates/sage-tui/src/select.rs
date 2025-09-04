use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::{Theme, supports_color};

enum SelectAction {
    Continue,
    Select,
    Cancel,
}

pub struct Select<'a, T> {
    items: Vec<T>,
    display: Box<dyn Fn(&T) -> String + 'a>,
    preview: Option<Box<dyn Fn(&T) -> Vec<String> + 'a>>,
    prompt: String,
    fuzzy_search: bool,
    filter: String,
    filtered_indices: Vec<usize>,
    selected: usize,
    theme: Theme,
    use_color: bool,
    show_numbers: bool,
    max_height: usize,
    scroll_offset: usize,
}

impl<'a, T> Select<'a, T> {
    pub fn new(prompt: impl Into<String>, items: Vec<T>) -> Self {
        let item_count = items.len();
        let filtered_indices = (0..item_count).collect();

        Self {
            items,
            display: Box::new(|_| String::from("(no display function)")),
            preview: None,
            prompt: prompt.into(),
            fuzzy_search: false,
            filter: String::new(),
            filtered_indices,
            selected: 0,
            theme: Theme::default(),
            use_color: supports_color(),
            show_numbers: false,
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

    pub fn with_preview<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Vec<String> + 'a,
    {
        self.preview = Some(Box::new(f));
        self
    }

    pub fn fuzzy_search(mut self, enabled: bool) -> Self {
        self.fuzzy_search = enabled;
        self
    }

    pub fn show_numbers(mut self, show: bool) -> Self {
        self.show_numbers = show;
        self
    }

    pub fn max_height(mut self, height: usize) -> Self {
        self.max_height = height.max(3);
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

    pub fn run(mut self) -> Result<Option<usize>> {
        if self.items.is_empty() {
            return Ok(None);
        }
        terminal::enable_raw_mode()?;

        execute!(io::stdout(), cursor::Hide)?;
        execute!(io::stdout(), cursor::SavePosition)?;

        let result = self.run_loop();

        execute!(io::stdout(), cursor::RestorePosition)?;
        execute!(io::stdout(), terminal::Clear(ClearType::FromCursorDown))?;
        execute!(io::stdout(), cursor::Show)?;
        terminal::disable_raw_mode()?;

        result
    }

    fn run_loop(&mut self) -> Result<Option<usize>> {
        loop {
            self.render()?;

            if let Event::Key(key) = event::read()? {
                match self.handle_key(key) {
                    SelectAction::Select => {
                        if !self.filtered_indices.is_empty() {
                            return Ok(Some(self.filtered_indices[self.selected]));
                        }
                    }
                    SelectAction::Cancel => return Ok(None),
                    SelectAction::Continue => continue,
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> SelectAction {
        match key.code {
            KeyCode::Enter => SelectAction::Select,
            KeyCode::Esc => SelectAction::Cancel,
            KeyCode::Up => {
                self.move_selection(-1);
                SelectAction::Continue
            }
            KeyCode::Down => {
                self.move_selection(1);
                SelectAction::Continue
            }
            KeyCode::PageUp => {
                self.move_selection(-(self.max_height as i32));
                SelectAction::Continue
            }
            KeyCode::PageDown => {
                self.move_selection(self.max_height as i32);
                SelectAction::Continue
            }
            KeyCode::Home => {
                self.selected = 0;
                self.scroll_offset = 0;
                SelectAction::Continue
            }
            KeyCode::End => {
                if !self.filtered_indices.is_empty() {
                    self.selected = self.filtered_indices.len() - 1;
                    self.update_scroll();
                }
                SelectAction::Continue
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                SelectAction::Cancel
            }
            KeyCode::Char(c) if self.fuzzy_search => {
                self.filter.push(c);
                self.update_filter();
                SelectAction::Continue
            }
            KeyCode::Backspace if self.fuzzy_search && !self.filter.is_empty() => {
                self.filter.pop();
                self.update_filter();
                SelectAction::Continue
            }
            KeyCode::Char(c) if self.show_numbers && c.is_ascii_digit() => {
                let num = c.to_digit(10).unwrap() as usize;
                if num > 0 && num <= self.filtered_indices.len() && num <= 9 {
                    self.selected = num - 1;
                    return SelectAction::Select;
                }
                SelectAction::Continue
            }
            _ => SelectAction::Continue,
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_indices.is_empty() {
            return;
        }

        let new_selected = if delta < 0 {
            self.selected.saturating_sub(delta.abs() as usize)
        } else {
            (self.selected + delta as usize).min(self.filtered_indices.len() - 1)
        };

        self.selected = new_selected;
        self.update_scroll();
    }

    fn update_scroll(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.max_height {
            self.scroll_offset = self.selected - self.max_height + 1;
        }
    }

    fn update_filter(&mut self) {
        self.filtered_indices.clear();

        if self.filter.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let filter_lower = self.filter.to_lowercase();

            for (idx, item) in self.items.iter().enumerate() {
                let display = (self.display)(item).to_lowercase();

                if self.fuzzy_match(&display, &filter_lower) {
                    self.filtered_indices.push(idx);
                }
            }
        }

        self.selected = 0;
        self.scroll_offset = 0;
    }

    fn fuzzy_match(&self, text: &str, pattern: &str) -> bool {
        let mut pattern_chars = pattern.chars();
        let mut current_char = pattern_chars.next();

        for text_char in text.chars() {
            if let Some(pc) = current_char {
                if text_char == pc {
                    current_char = pattern_chars.next();
                    if current_char.is_none() {
                        return true;
                    }
                }
            }
        }

        current_char.is_none()
    }

    fn render(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown)
        )?;
        println!("  {}", self.prompt);
        if self.fuzzy_search {
            if self.filter.is_empty() {
                println!("  {}", self.style("Type to filter...", self.theme.muted));
            } else {
                println!(
                    "  Filter: {} ({})",
                    self.style(&self.filter, self.theme.primary),
                    self.filtered_indices.len()
                );
            }
        }
        println!();
        let visible_end = (self.scroll_offset + self.max_height).min(self.filtered_indices.len());
        for (display_idx, actual_idx) in self.filtered_indices[self.scroll_offset..visible_end]
            .iter()
            .enumerate()
        {
            let is_selected = display_idx + self.scroll_offset == self.selected;
            let item = &self.items[*actual_idx];
            let display_text = (self.display)(item);
            print!("  ");

            if is_selected {
                print!("{}", self.style(">", self.theme.primary));
            } else {
                print!(" ");
            }
            print!(" ");
            if self.show_numbers && display_idx < 9 {
                print!(
                    "{} ",
                    self.style(&format!("{}.", display_idx + 1), self.theme.muted)
                );
            } else if self.show_numbers {
                print!("  ");
            }
            if is_selected {
                println!("{}", self.style(&display_text, self.theme.primary));
            } else {
                println!("{}", display_text);
            }
        }
        if self.scroll_offset > 0 {
            println!("  {}", self.style("↑ more above", self.theme.muted));
        }
        if visible_end < self.filtered_indices.len() {
            println!("  {}", self.style("↓ more below", self.theme.muted));
        }
        if let Some(preview_fn) = &self.preview {
            if !self.filtered_indices.is_empty() {
                let selected_item = &self.items[self.filtered_indices[self.selected]];
                let preview_lines = preview_fn(selected_item);

                if !preview_lines.is_empty() {
                    println!();
                    println!("  {}", self.style("Preview:", self.theme.muted));
                    for line in preview_lines.iter().take(5) {
                        println!("  {}", self.style(line, self.theme.muted));
                    }
                }
            }
        }
        println!();
        println!(
            "  {}",
            self.style("↑↓ Navigate • Enter Select • Esc Cancel", self.theme.muted)
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
