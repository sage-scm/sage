use anyhow::{Result, bail};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::Theme;

pub struct TextInput {
    prompt: String,
    value: String,
    cursor_pos: usize,
    placeholder: Option<String>,
    default: Option<String>,
    validate: Option<Box<dyn Fn(&str) -> Result<()>>>,
    theme: Theme,
    password: bool,
    error_message: Option<String>,
}

impl TextInput {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            value: String::new(),
            cursor_pos: 0,
            placeholder: None,
            default: None,
            validate: None,
            theme: Theme::default(),
            password: false,
            error_message: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn default(mut self, text: impl Into<String>) -> Self {
        let default_text = text.into();
        self.value = default_text.clone();
        self.cursor_pos = self.value.len();
        self.default = Some(default_text);
        self
    }

    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<()> + 'static,
    {
        self.validate = Some(Box::new(f));
        self
    }

    pub fn password(mut self, is_password: bool) -> Self {
        self.password = is_password;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn run(mut self) -> Result<String> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), cursor::SavePosition)?;

        let result = self.run_loop();

        execute!(
            io::stdout(),
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown)
        )?;
        terminal::disable_raw_mode()?;

        result
    }

    fn run_loop(&mut self) -> Result<String> {
        loop {
            self.render()?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        if let Some(validate) = &self.validate {
                            match validate(&self.value) {
                                Ok(_) => return Ok(self.value.clone()),
                                Err(e) => {
                                    self.error_message = Some(e.to_string());
                                }
                            }
                        } else {
                            return Ok(self.value.clone());
                        }
                    }
                    KeyCode::Esc => bail!("Input cancelled"),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        bail!("Input cancelled")
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.value.clear();
                        self.cursor_pos = 0;
                        self.error_message = None;
                    }
                    KeyCode::Char(c) => {
                        self.value.insert(self.cursor_pos, c);
                        self.cursor_pos += 1;
                        self.error_message = None;
                    }
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.value.remove(self.cursor_pos);
                            self.error_message = None;
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_pos < self.value.len() {
                            self.value.remove(self.cursor_pos);
                            self.error_message = None;
                        }
                    }
                    KeyCode::Left => {
                        self.cursor_pos = self.cursor_pos.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        self.cursor_pos = (self.cursor_pos + 1).min(self.value.len());
                    }
                    KeyCode::Home => {
                        self.cursor_pos = 0;
                    }
                    KeyCode::End => {
                        self.cursor_pos = self.value.len();
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        println!("  {}", self.prompt);
        print!("  ");

        if self.value.is_empty() && self.placeholder.is_some() {
            execute!(
                io::stdout(),
                SetForegroundColor(self.theme.muted),
                Print(self.placeholder.as_ref().unwrap()),
                ResetColor
            )?;
        } else if self.password {
            print!("{}", "*".repeat(self.value.len()));
        } else {
            print!("{}", self.value);
        }

        if let Some(error) = &self.error_message {
            println!();
            execute!(
                io::stdout(),
                SetForegroundColor(self.theme.error),
                Print(format!("  ✗ {}", error)),
                ResetColor
            )?;
        }

        let display_cursor_pos = if self.password {
            self.cursor_pos
        } else {
            self.cursor_pos
        };

        execute!(
            io::stdout(),
            cursor::MoveToColumn(2 + display_cursor_pos as u16),
            cursor::MoveUp(if self.error_message.is_some() { 1 } else { 0 }),
            cursor::Show
        )?;

        io::stdout().flush()?;
        Ok(())
    }
}
