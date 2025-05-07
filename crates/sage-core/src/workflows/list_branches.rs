use anyhow::Result;
use sage_git::status;

pub fn list_branches(stats: bool) -> Result<()> {
    let branches = sage_git::branch::list_branches()?;

    let current_branch = sage_git::branch::get_current()?;

    for branch in branches {
        // We need to switch to the branch to get the status.
        sage_git::branch::switch(&branch, false)?;
        let branch_status = status::status()?;

        let prefix = if branch == current_branch {
            "●  "
        } else {
            "   "
        };

        let branch_status = if stats {
            &format!(
                "↑{} ↓{}",
                branch_status.ahead_count, branch_status.behind_count
            )
        } else {
            ""
        };

        println!("{} {} {}", prefix, branch, branch_status);
    }

    sage_git::branch::switch(&current_branch, false)?;

    Ok(())
}
