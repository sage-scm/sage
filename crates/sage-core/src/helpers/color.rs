use anyhow::{Result, anyhow};
use colored::ColoredString;
use colored::Colorize;

pub fn hex(text: &str, hex: &str) -> ColoredString {
    let rgb = hex_to_rgb(hex).unwrap();
    text.truecolor(rgb.0, rgb.1, rgb.2)
}

pub fn sage(text: &str) -> ColoredString {
    hex(text, "#8EA58C")
}

pub fn gray(text: &str) -> ColoredString {
    hex(text, "#6B737C")
}

pub fn blue(text: &str) -> ColoredString {
    hex(text, "#59B4FF")
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8)> {
    let mut hex = hex.trim_start_matches('#').to_lowercase();

    if hex.len() != 6 && hex.len() != 3 {
        Err(anyhow!("Invalid hex color: {}", hex))?;
    }

    for c in hex.chars() {
        if (c < '0' || c > '9') && (c < 'a' || c > 'f') {
            Err(anyhow!("Invalid hex color: {}", hex))?;
        }
    }

    if hex.len() == 3 {
        hex = format!(
            "{}{}{}{}{}{}",
            &hex.chars().nth(0).unwrap(),
            &hex.chars().nth(0).unwrap(),
            &hex.chars().nth(1).unwrap(),
            &hex.chars().nth(1).unwrap(),
            &hex.chars().nth(2).unwrap(),
            &hex.chars().nth(2).unwrap()
        );
    }

    let r_val = u8::from_str_radix(&hex[0..2], 16)?;
    let g_val = u8::from_str_radix(&hex[2..4], 16)?;
    let b_val = u8::from_str_radix(&hex[4..6], 16)?;

    Ok((r_val, g_val, b_val))
}

pub trait ColorizeExt {
    fn sage(&self) -> ColoredString;
    fn gray(&self) -> ColoredString;
    fn blue(&self) -> ColoredString;
    fn url(&self) -> ColoredString;
}

impl ColorizeExt for str {
    fn sage(&self) -> ColoredString {
        sage(self)
    }
    fn gray(&self) -> ColoredString {
        gray(self)
    }
    fn blue(&self) -> ColoredString {
        blue(self)
    }
    fn url(&self) -> ColoredString {
        // Style the URL as blue and underlined using the custom trait's blue method
        <str as ColorizeExt>::blue(self).underline()
    }
}
