mod main_menu;

use crate::display::main_menu::MainMenu;
use anyhow::Result;
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, terminal::enable_raw_mode};

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Main,
    Help,
}

#[derive(Default, Debug)]
pub struct SageUI {
    current_screen: Screen,
    exit: bool,
}

impl SageUI {
    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut terminal = ratatui::init();

        while !self.exit {
            match self.current_screen {
                Screen::Main => MainMenu::default().run(&mut terminal)?,
                _ => todo!(),
                // Screen::Help => self.help_screen(&mut terminal)?,
            }
            self.handle_events()?;
        }
        ratatui::restore();
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('h') => self.switch_screen(Screen::Help),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn switch_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }
}
