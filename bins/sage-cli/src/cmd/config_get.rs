use anyhow::Result;

pub fn config_get(key: &str) -> Result<()> {
    let manager = sage_config::ConfigManager::new()?;
    let cfg = manager.load()?;

    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["editor"] => println!("{}", cfg.editor),
        ["auto_update"] => println!("{}", cfg.auto_update),
        ["plugin_dirs"] => println!("{:?}", cfg.plugin_dirs),
        ["tui"] => println!("{:?}", cfg.tui),
        ["tui", "font_size"] => println!("{}", cfg.tui.font_size),
        ["tui", "color_theme"] => println!("{}", cfg.tui.color_theme),
        ["tui", "line_numbers"] => println!("{}", cfg.tui.line_numbers),
        ["ai"] => println!("{:?}", cfg.ai),
        ["ai", "model"] => println!("{}", cfg.ai.model),
        ["ai", "api_url"] => println!("{}", cfg.ai.api_url),
        ["ai", "api_key"] => println!("{}", cfg.ai.api_key),
        ["ai", "max_tokens"] => println!("{}", cfg.ai.max_tokens),
        ["ai", "timeout"] => println!("{}", cfg.ai.max_tokens),
        ["pull_requests"] => println!("{:?}", cfg.pull_requests),
        ["pull_requests", "enabled"] => println!("{}", cfg.pull_requests.enabled),
        ["pull_requests", "default_base"] => println!("{}", cfg.pull_requests.default_base),
        ["pull_requests", "provider"] => println!("{}", cfg.pull_requests.provider),
        ["pull_requests", "access_token"] => println!("{}", cfg.pull_requests.access_token),
        ["save"] => println!("{:?}", cfg.save),
        ["save", "ask_on_mixed_staging"] => println!("{}", cfg.save.ask_on_mixed_staging),
        ["extras", extra_key] => {
            if let Some(val) = cfg.extras.get(*extra_key) {
                println!("{}", val);
            } else {
                eprintln!("Key not found: extras.{}", extra_key);
            }
        }
        ["ai", "extras", extra_key] => {
            if let Some(val) = cfg.ai.extras.get(*extra_key) {
                println!("{}", val);
            } else {
                eprintln!("Key not found: ai.extras.{}", extra_key);
            }
        }
        ["tui", "extras", extra_key] => {
            if let Some(val) = cfg.tui.extras.get(*extra_key) {
                println!("{}", val);
            } else {
                eprintln!("Key not found: tui.extras.{}", extra_key);
            }
        }
        ["pull_requests", "extras", extra_key] => {
            if let Some(val) = cfg.pull_requests.extras.get(*extra_key) {
                println!("{}", val);
            } else {
                eprintln!("Key not found: pull_requests.extras.{}", extra_key);
            }
        }
        _ => {
            eprintln!("✖ Key ‘does.not.exist’ not found.");
            eprintln!("Unknown config key: {}", key);
        }
    }
    Ok(())
}
