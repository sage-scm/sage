use anyhow::Result;
use inquire::Confirm;

pub fn check(question: String) -> Result<bool> {
    Ok(Confirm::new(&question).prompt()?)
}
