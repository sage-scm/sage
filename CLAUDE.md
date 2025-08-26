# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sage is a Git workflow tool written in Rust that simplifies complex Git operations. It's built as a Cargo workspace with multiple crates for modularity.

## Essential Commands

### Building and Development
- **Build**: `cargo build` (debug mode) or `cargo build --release`
- **Install locally from source**: `./install-local.sh [--release]`
  - `--release`: Build optimized binary (optional)
- **Install from GitHub releases**: `curl -fsSL https://raw.githubusercontent.com/sage-scm/sage/main/install.sh | sh`
- **Development watch mode**: `./watch.sh` - Auto-rebuilds on Rust file changes
- **Run tests**: `cargo test --workspace`
- **Run specific test**: `cargo test <test_name>`
- **Run without installing**: `cargo run --bin sage-cli -- <command>`

### Built-in Features
All features are now enabled by default:
- Advanced stacked-diff operations
- AI-powered commit messages (supports local Ollama)
- Terminal UI dashboard

## Architecture

### Workspace Structure
```
sage/
├── bins/
│   ├── sage-cli/     # Main CLI application (entry: src/main.rs)
│   └── sage-server/  # Optional sync server (not fully implemented)
├── crates/
│   ├── sage-core/    # Core business logic and workflows
│   ├── sage-git/     # Git operations (wraps git2)
│   ├── sage-config/  # Configuration management
│   ├── sage-tui/     # Terminal UI output formatting and interactive components
│   ├── sage-graph/   # Branch graph for stack operations
│   ├── sage-plugin/  # Plugin SDK
│   └── sage-utils/   # Shared utilities
```

### Key Architectural Patterns

1. **Workflow-Based Design**: Core operations are organized as workflows in `crates/sage-core/src/workflows/`. Each workflow handles a complete user action (save, sync, work, etc.). The CLI is installed as `sg` to avoid conflicts with SageMath.

2. **Layered Architecture**:
   - CLI layer: Command parsing with Clap, user interaction
   - Core layer: Business logic, workflow orchestration
   - Git layer: Abstraction over git2 operations
   - Config/Utils: Cross-cutting concerns

3. **Error Handling**: Consistent use of `anyhow::Result` throughout the codebase.

4. **Terminal UI**: The `sage-tui` crate provides a unified terminal output system with:
   - Consistent colored output with theme support
   - Progress indicators and progress bars
   - Interactive prompts with keyboard input handling
   - Tree visualization for branch stacks
   - File change displays with aligned columns
   - Auto-detection of CI environments and color support

5. **Configuration System**: Layered config supporting editor preferences, auto-update, plugins, save behavior, TUI customization, and AI settings. Config is managed through `sage-config` crate.

6. **Async Operations**: Most code is synchronous except AI and GitHub API calls which use Tokio.

## Key Components

### Command Structure
Commands are defined in `bins/sage-cli/src/commands/` with each command having its own module. The main entry point uses Clap to parse commands and dispatch to the appropriate handler.

### Workflows
Located in `crates/sage-core/src/workflows/`, these contain the main business logic:
- `save.rs`: Staging and committing changes
- `sync.rs`: Restacking and pushing branches
- `work.rs`: Creating/switching branches
- `share.rs`: PR creation/updates
- Stack-related workflows

### Git Operations
The `sage-git` crate provides a safe wrapper around git2 operations, handling common patterns like:
- Branch management
- Commit operations
- Remote interactions
- Repository state queries

### Terminal UI Components
The `sage-tui` crate provides:
- `Tui` struct: Main interface for terminal output with methods for headers, summaries, file lists, trees, prompts, and messages
- `ProgressHandle` and `ProgressBar`: Animated progress indicators with automatic cleanup
- `Theme`: Customizable color themes (default, monochrome, high-contrast, solarized)
- `TreeNode`: Tree structure visualization for branch stacks
- Components for file changes, status messages, and interactive prompts

## Development Notes

- The project uses "dog-fooding" - developers use Sage to manage Sage itself
- Many features are still marked as `todo!()` in the implementation
- The codebase follows standard Rust conventions with clear module organization
- Tests are co-located with source files using `#[cfg(test)]` modules

## Coding Guidelines

We have strict coding standards to maintain a beautiful, high-quality codebase:

1. **All code must have a purpose and be thought out** - No unnecessary abstractions or over-engineering
2. **No code comments unless absolutely necessary** - Code should be self-documenting. Only add comments for genuinely complex algorithms that need explanation
3. **Code must be beautiful** - We want developers to look at our codebase and think "wow". Clean, elegant solutions only
4. **Keep it stupid-simple** - No advanced Rust tactics or clever tricks. Code should be readable by Rust developers of all skill levels
