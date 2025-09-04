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

impl Theme {
    pub fn monochrome() -> Self {
        Self {
            primary: Color::White,
            success: Color::White,
            error: Color::White,
            warning: Color::White,
            info: Color::White,
            muted: Color::White,
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            error: Color::Magenta,
            warning: Color::Yellow,
            info: Color::Blue,
            muted: Color::Grey,
        }
    }

    pub fn solarized() -> Self {
        Self {
            primary: Color::Rgb {
                r: 42,
                g: 161,
                b: 152,
            },
            success: Color::Rgb {
                r: 133,
                g: 153,
                b: 0,
            },
            error: Color::Rgb {
                r: 211,
                g: 54,
                b: 130,
            },
            warning: Color::Rgb {
                r: 181,
                g: 137,
                b: 0,
            },
            info: Color::Rgb {
                r: 38,
                g: 139,
                b: 210,
            },
            muted: Color::Rgb {
                r: 88,
                g: 110,
                b: 117,
            },
        }
    }
}
