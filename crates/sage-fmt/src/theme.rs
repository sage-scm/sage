use crossterm::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Blue,
            muted: Color::DarkGrey,
        }
    }
}
