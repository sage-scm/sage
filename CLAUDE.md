# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sage is a Git workflow tool written in Rust that simplifies complex Git operations. It's built as a Cargo workspace with multiple crates for modularity.

## Essential Commands

### Building and Development
- **Build**: `cargo build` (debug mode) or `cargo build --release`
- **Install locally**: `./install.sh [--stack] [--ai] [--tui] [--all] [--release]`
  - `--stack`: Enable stacked-diff commands
  - `--ai`: Enable AI commit message generation
  - `--tui`: Enable terminal UI mode
  - `--all`: Enable all features
  - `--release`: Build optimized binary
- **Development watch mode**: `./watch.sh` - Auto-rebuilds on Rust file changes
- **Run tests**: `cargo test --all`
- **Run specific test**: `cargo test <test_name>`
- **Run without installing**: `cargo run --bin sage-cli -- <command>`

### Feature Flags
The project uses Cargo features to enable/disable functionality:
- `stack`: Advanced stacked-diff operations
- `ai`: AI-powered commit messages (supports local Ollama)
- `tui`: Terminal UI dashboard

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
│   ├── sage-tui/     # Terminal UI components
│   ├── sage-graph/   # Branch graph for stack features
│   ├── sage-plugin/  # Plugin SDK
│   └── sage-utils/   # Shared utilities
```

### Key Architectural Patterns

1. **Workflow-Based Design**: Core operations are organized as workflows in `crates/sage-core/src/workflows/`. Each workflow handles a complete user action (save, sync, work, etc.).

2. **Layered Architecture**:
   - CLI layer: Command parsing with Clap, user interaction
   - Core layer: Business logic, workflow orchestration
   - Git layer: Abstraction over git2 operations
   - Config/Utils: Cross-cutting concerns

3. **Error Handling**: Consistent use of `anyhow::Result` throughout the codebase.

4. **CLI Output**: The `CliOutput` trait in `sage-core` provides consistent terminal output with colors, spinners, and progress bars.

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
- Stack-related workflows when `stack` feature is enabled

### Git Operations
The `sage-git` crate provides a safe wrapper around git2 operations, handling common patterns like:
- Branch management
- Commit operations
- Remote interactions
- Repository state queries

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
