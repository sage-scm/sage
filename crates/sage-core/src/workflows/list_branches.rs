use anyhow::Result;
use sage_git::status::branch_status;

pub fn list_branches(stats: bool) -> Result<()> {
    let branches = sage_git::branch::list_branches()?;
    let current_branch = sage_git::branch::get_current()?;

    for branch in branches {
        let prefix = if branch == current_branch {
            "●  "
        } else {
            "   "
        };

        let branch_status = if stats {
            let status = branch_status(&branch)?;
            if status.ahead_count > 0 || status.behind_count > 0 {
                format!(" ↑{} ↓{}", status.ahead_count, status.behind_count)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        println!("{} {}{}", prefix, branch, branch_status);
    }

    Ok(())
}
