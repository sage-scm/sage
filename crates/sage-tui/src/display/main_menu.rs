use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};
use std::io;

#[derive(Default, Debug)]
pub enum MainMenuView {
    #[default]
    StackView,
    Preview,
}

#[derive(Debug)]
pub struct MainMenu {
    pub current_view: MainMenuView,
    pub current_stack: Option<String>,
    pub in_stack: bool,
    pub exit: bool,
    pub selected_tab: usize,
    pub tabs: Vec<&'static str>,
    pub stack_cursor: usize,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            current_view: MainMenuView::StackView,
            current_stack: None,
            in_stack: false,
            exit: false,
            selected_tab: 0,
            tabs: vec!["Diff", "PR", "Commits", "Log", "Tree"],
            stack_cursor: 0,
        }
    }
}

impl MainMenu {
    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        while !self.exit {
            terminal.draw(|f| self.draw(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => self.exit = true,
                        KeyCode::Left => {
                            if self.selected_tab > 0 {
                                self.selected_tab -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if self.selected_tab < self.tabs.len() - 1 {
                                self.selected_tab += 1;
                            }
                        }
                        KeyCode::Up => {
                            if self.stack_cursor > 0 {
                                self.stack_cursor -= 1;
                            }
                        }
                        KeyCode::Down => {
                            self.stack_cursor += 1;
                        }
                        KeyCode::Char('d') => self.current_view = MainMenuView::Preview,
                        KeyCode::Char('s') => self.current_view = MainMenuView::StackView,
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let outer_block = Block::default()
            .title("🌿 SAGE")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let inner_area = outer_block.inner(frame.size());
        frame.render_widget(outer_block, frame.size());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(10),
                Constraint::Length(1),
            ])
            .split(inner_area);

        frame.render_widget(&HeaderView, chunks[0]);
        frame.render_widget(&InfoView, chunks[1]);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[2]);

        match self.current_view {
            MainMenuView::StackView => {
                frame.render_widget(
                    &StackView {
                        cursor: self.stack_cursor,
                    },
                    content_chunks[0],
                );
                frame.render_widget(
                    &PreviewView {
                        selected_tab: self.selected_tab,
                        tabs: &self.tabs,
                    },
                    content_chunks[1],
                );
            }
            MainMenuView::Preview => {
                let full_width = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .split(chunks[2])[0];
                frame.render_widget(
                    &PreviewView {
                        selected_tab: self.selected_tab,
                        tabs: &self.tabs,
                    },
                    full_width,
                );
            }
        }

        frame.render_widget(&PluginStatusView, chunks[3]);
    }
}

struct HeaderView;

impl Widget for &HeaderView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Paragraph::new(vec![
            Line::from("repo: commonwealth/banking-core                    branch: feature/refactor-auth-stack"),
            Line::from("stack: auth-rework                            default: main"),
        ])
        .wrap(Wrap { trim: true })
        .render(area, buf);
    }
}

struct InfoView;

impl Widget for &InfoView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Paragraph::new("↑↓: Move  ⏎: View Diff  ←/→: Tab  s: Stack View  d: Preview  q: Quit")
            .style(Style::default().fg(Color::Gray))
            .render(area, buf);
    }
}

struct StackView {
    cursor: usize,
}

impl Widget for &StackView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let items = ["● main (default)                                   [merged ✅]",
            "├─ ◉ refactor-auth                                  [PR #82] [open ⏳]",
            "│  ├─ ◉ login-method-swap                           [PR #83] [draft 📝]",
            "│  │  ├─ ◉ remove-passwords                         [no PR] [untracked 🚫]",
            "│  │  └─ ◯ otp-migration                             [no PR] [dirty ✏️]",
            "│  └─ ◯ cleanup-unused                              [no PR] [committed ✅]",
            "└─ ◯ docs-update                                     [PR #80] [merged ✅]"];

        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let content = if i == self.cursor {
                    Line::from(Span::styled(
                        *line,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(*line)
                };
                ListItem::new(content)
            })
            .collect();

        List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("STACK"))
            .render(area, buf);
    }
}

struct PreviewView<'a> {
    selected_tab: usize,
    tabs: &'a [&'a str],
}

impl<'a> Widget for &PreviewView<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(3)])
            .split(area);

        let tab_title = if !self.tabs.is_empty() {
            format!("View: {}", self.tabs[self.selected_tab])
        } else {
            "View".to_string()
        };

        Paragraph::new("commit: 192ac32 - remove legacy password flow\n\n- Removed auth/password_utils.js\n- Updated loginService.ts to use verifyOTP instead")
            .block(Block::default().borders(Borders::ALL).title(tab_title))
            .render(chunks[1], buf);

        Paragraph::new("PREVIEW (press d/p/c/l/t)").render(chunks[0], buf);
    }
}

struct PluginStatusView;

impl Widget for &PluginStatusView {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Paragraph::new(
            "🌱 Plugins: [🔌 coverage] [🎨 lint] [📦 size]   Last fetch: 2m ago   |   ⟳ Pulling...",
        )
        .style(Style::default().fg(Color::DarkGray))
        .render(area, buf);
    }
}
