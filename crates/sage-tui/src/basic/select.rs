use anyhow::Result;
use inquire::Select;

pub fn select(prompt: String, options: Vec<String>) -> Result<String> {
    Ok(Select::new(&prompt, options).prompt()?.to_string())
}
