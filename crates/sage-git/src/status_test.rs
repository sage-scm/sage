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
} 