#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Skip during normal test runs
    fn benchmark_status_methods() {
        println!("\n=== Git Status Performance Benchmark ===");

        // Benchmark full status
        let full_start = Instant::now();
        match status() {
            Ok(status) => {
                let full_duration = full_start.elapsed();
                println!("\n--- Full Status Method ---");
                println!("Time taken: {:?}", full_duration);
                println!("Status summary: {}", status.summary());
                println!(
                    "Branch: {}, Ahead: {}, Behind: {}",
                    status.current_branch, status.ahead_count, status.behind_count
                );
                println!("Staged files: {}", status.staged_files_count());
                println!("Unstaged files: {}", status.unstaged_files_count());
                println!("Untracked files: {}", status.untracked.len());
                println!("Has stash: {}", status.has_stash);
            }
            Err(e) => println!("Full status error: {}", e),
        }

        println!("\n=== Benchmark Complete ===");
    }

    #[test]
    fn test_is_detached_head() {
        // Test detached HEAD state
        let detached_status = GitStatus {
            current_branch: "HEAD".to_string(),
            upstream_branch: None,
            ahead_count: 0,
            behind_count: 0,
            has_stash: false,
            staged_added: vec![],
            staged_modified: vec![],
            staged_deleted: vec![],
            staged_renamed: vec![],
            staged_copied: vec![],
            unstaged_modified: vec![],
            unstaged_deleted: vec![],
            unstaged_added: vec![],
            untracked: vec![],
            ignored: vec![],
            staged_modified_unstaged_modified: vec![],
            staged_added_unstaged_modified: vec![],
            staged_added_unstaged_deleted: vec![],
            staged_deleted_unstaged_modified: vec![],
            staged_renamed_unstaged_modified: vec![],
            staged_copied_unstaged_modified: vec![],
        };

        assert!(detached_status.is_detached_head());

        // Test normal branch state
        let normal_status = GitStatus {
            current_branch: "main".to_string(),
            upstream_branch: Some("origin/main".to_string()),
            ahead_count: 0,
            behind_count: 0,
            has_stash: false,
            staged_added: vec![],
            staged_modified: vec![],
            staged_deleted: vec![],
            staged_renamed: vec![],
            staged_copied: vec![],
            unstaged_modified: vec![],
            unstaged_deleted: vec![],
            unstaged_added: vec![],
            untracked: vec![],
            ignored: vec![],
            staged_modified_unstaged_modified: vec![],
            staged_added_unstaged_modified: vec![],
            staged_added_unstaged_deleted: vec![],
            staged_deleted_unstaged_modified: vec![],
            staged_renamed_unstaged_modified: vec![],
            staged_copied_unstaged_modified: vec![],
        };

        assert!(!normal_status.is_detached_head());

        // Test with feature branch
        let feature_status = GitStatus {
            current_branch: "feature/awesome-feature".to_string(),
            upstream_branch: Some("origin/feature/awesome-feature".to_string()),
            ahead_count: 2,
            behind_count: 1,
            has_stash: true,
            staged_added: vec![],
            staged_modified: vec![],
            staged_deleted: vec![],
            staged_renamed: vec![],
            staged_copied: vec![],
            unstaged_modified: vec![],
            unstaged_deleted: vec![],
            unstaged_added: vec![],
            untracked: vec![],
            ignored: vec![],
            staged_modified_unstaged_modified: vec![],
            staged_added_unstaged_modified: vec![],
            staged_added_unstaged_deleted: vec![],
            staged_deleted_unstaged_modified: vec![],
            staged_renamed_unstaged_modified: vec![],
            staged_copied_unstaged_modified: vec![],
        };

        assert!(!feature_status.is_detached_head());
    }
}
