use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::Theme;

/// Action to take after handling a key
enum SelectAction {
    Continue,
    Select,
    Cancel,
}

/// Interactive select component with fuzzy search
pub struct Select<'a, T> {
    /// Items to select from
    items: Vec<T>,
    /// Display function for items
    display: Box<dyn Fn(&T) -> String + 'a>,
    /// Optional preview function
    preview: Option<Box<dyn Fn(&T) -> Vec<String> + 'a>>,
    /// Prompt message
    prompt: String,
    /// Enable fuzzy search
    fuzzy_search: bool,
    /// Current filter string
    filter: String,
    /// Filtered items with their original indices
    filtered_indices: Vec<usize>,
    /// Currently selected index in filtered list
    selected: usize,
    /// Theme
    theme: Theme,
    /// Whether colors are enabled
    use_color: bool,
    /// Show item index numbers
    show_numbers: bool,
    /// Max items to show at once
    max_height: usize,
    /// Scroll offset
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

    /// Set custom display function
    pub fn with_display<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> String + 'a,
    {
        self.display = Box::new(f);
        self
    }

    /// Set preview function
    pub fn with_preview<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Vec<String> + 'a,
    {
        self.preview = Some(Box::new(f));
        self
    }

    /// Enable fuzzy search
    pub fn fuzzy_search(mut self, enabled: bool) -> Self {
        self.fuzzy_search = enabled;
        self
    }

    /// Show item numbers for quick selection
    pub fn show_numbers(mut self, show: bool) -> Self {
        self.show_numbers = show;
        self
    }

    /// Set max visible height
    pub fn max_height(mut self, height: usize) -> Self {
        self.max_height = height.max(3); // Minimum 3 items
        self
    }

    /// Set theme
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set whether to use colors
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    /// Run the interactive selection
    pub fn run(mut self) -> Result<Option<usize>> {
        if self.items.is_empty() {
            return Ok(None);
        }

        // Enable raw mode for keyboard input
        terminal::enable_raw_mode()?;

        // Hide cursor during selection
        execute!(io::stdout(), cursor::Hide)?;

        // Save cursor position
        execute!(io::stdout(), cursor::SavePosition)?;

        let result = self.run_loop();

        // Restore terminal state
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
            // Simple fuzzy matching
            let filter_lower = self.filter.to_lowercase();

            for (idx, item) in self.items.iter().enumerate() {
                let display = (self.display)(item).to_lowercase();

                if self.fuzzy_match(&display, &filter_lower) {
                    self.filtered_indices.push(idx);
                }
            }
        }

        // Reset selection
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
        // Clear from cursor down
        execute!(
            io::stdout(),
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        // Print prompt
        println!("  {}", self.prompt);

        // Show filter if active
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

        // Empty line for spacing
        println!();

        // Calculate visible range
        let visible_end = (self.scroll_offset + self.max_height).min(self.filtered_indices.len());

        // Show items
        for (display_idx, actual_idx) in self.filtered_indices[self.scroll_offset..visible_end]
            .iter()
            .enumerate()
        {
            let is_selected = display_idx + self.scroll_offset == self.selected;
            let item = &self.items[*actual_idx];
            let display_text = (self.display)(item);

            // Indent and selection indicator
            print!("  ");

            if is_selected {
                print!("{}", self.style(">", self.theme.primary));
            } else {
                print!(" ");
            }
            print!(" ");

            // Number if enabled
            if self.show_numbers && display_idx < 9 {
                print!(
                    "{} ",
                    self.style(&format!("{}.", display_idx + 1), self.theme.muted)
                );
            } else if self.show_numbers {
                print!("  ");
            }

            // Item text
            if is_selected {
                println!("{}", self.style(&display_text, self.theme.primary));
            } else {
                println!("{}", display_text);
            }
        }

        // Show scroll indicators
        if self.scroll_offset > 0 {
            println!("  {}", self.style("↑ more above", self.theme.muted));
        }
        if visible_end < self.filtered_indices.len() {
            println!("  {}", self.style("↓ more below", self.theme.muted));
        }

        // Show preview if available
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

        // Show help
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

fn supports_color() -> bool {
    atty::is(atty::Stream::Stdout)
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}
