use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use sage_core::check_for_updates;

mod cmd;

/// Global CLI configuration passed to all commands
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub json: bool,
    pub no_color: bool,
}

impl GlobalConfig {
    pub fn new(json: bool, no_color: bool) -> Self {
        Self { json, no_color }
    }
}

/// 🌿 Sage -- Burning away git complexities
#[derive(Parser)]
#[command(author, version, about = "🌿 Sage — A Git workflow tool for managing branches and commits", long_about = None)]
pub struct Cli {
    /// Output results in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // ────────────────────────────────── Core workflow ─────────────────────────
    /// Smart create / checkout a branch
    #[clap(alias = "w")]
    Work(WorkArgs),

    /// Stage → commit (optionally AI-assisted)
    #[clap(alias = "s")]
    Save(SaveArgs),

    /// Restack + push
    #[clap(alias = "ss")]
    Sync(SyncArgs),

    /// Create or update a PR
    Share(ShareArgs),

    /// Repo dashboard
    Dash {
        /// Live refresh
        #[arg(long)]
        watch: bool,
    },

    /// Show previous commits
    Log,

    /// Prune branches and reflog
    Clean {
        #[arg(long)]
        remote: bool,
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Revert an item
    Undo { id: Option<String> },

    /// Alias for `undo --list`
    History,

    /// Launch mergetool
    Resolve,

    /// Repo statistics
    Stats {
        #[arg(long = "since")]
        since: Option<String>,
    },

    /// Environment / toolchain health‑check
    Doctor {
        #[arg(long)]
        fix: bool,
    },

    /// List local branches
    List {
        #[arg(short, long)]
        relative: bool,
    },

    /// Config management
    #[clap(alias = "c")]
    Config {
        #[command(subcommand)]
        op: ConfigCmd,
    },

    /// Generate shell completions
    Completion { shell: String },

    // ─────────────────────────────── Plugin manager ──────────────────────────
    Plugin {
        #[command(subcommand)]
        cmd: PluginCmd,
    },

    // ───────────────────────────── Stack namespace ───────────────────────────
    Stack {
        #[command(subcommand)]
        op: StackCmd,
    },

    // ──────────────────────────────── AI extras ──────────────────────────────
    Tips,

    // ──────────────────────────────── TUI mode ───────────────────────────────
    Ui,
}

// ───────────────────────────────── Argument structs ──────────────────────────
#[derive(Args, Debug)]
pub struct SaveArgs {
    /// The message to commit with
    #[clap(value_parser)]
    message: Option<String>,
    /// Use AI to generate a commit message
    #[arg(short, long)]
    ai: bool,
    /// Commit all changes
    #[arg(long)]
    all: bool,
    /// Commit only these paths
    #[arg(long, value_delimiter = ',')]
    paths: Option<Vec<String>>,
    /// Amend the previous commit
    #[arg(long)]
    amend: bool,
    /// Push the commit to the remote
    #[arg(short = 'p', long)]
    push: bool,
    /// Create an empty git commit
    #[arg(short, long)]
    empty: bool,
}

#[derive(Args, Debug)]
pub struct WorkArgs {
    /// The branch to work on
    #[clap(value_parser)]
    branch: Option<String>,
    #[clap(value_parser, long)]
    parent: Option<String>,
    /// Fetch from remote before switching (only when switching to existing branches)
    #[clap(long, short, default_value = "false")]
    fetch: bool,
    /// Use the root branch
    #[clap(long, short, default_value = "false")]
    root: bool,
    /// Push to remote after
    #[clap(long, short, default_value = "false")]
    push: bool,
    /// Fuzzy search for branch name
    #[clap(long, short = 'z', default_value = "false")]
    fuzzy: bool,
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Continue an interrupted sync operation
    #[arg(long)]
    continue_: bool,
    /// Abort the current sync operation
    #[arg(long)]
    abort: bool,
    /// Rebase instead of merge when syncing
    #[arg(long, short)]
    rebase: bool,
    /// Force push after successful sync
    #[arg(long, short)]
    force: bool,
    /// Parent branch to sync with (defaults to tracked parent or default branch)
    #[arg(long, short)]
    parent: Option<String>,
    /// Automatically stash and unstash changes if needed
    #[arg(long)]
    autostash: bool,
    /// Only sync the current branch, not the entire stack
    #[arg(long)]
    no_stack: bool,
}

#[derive(Args, Debug)]
pub struct ShareArgs {
    #[arg(long)]
    draft: bool,
    #[arg(long)]
    ready: bool,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// List all available configuration options with their current values
    #[clap(alias = "l")]
    List,
    /// Get the value of a specific configuration key
    #[clap(alias = "g")]
    Get { key: String },
    /// Set a configuration value
    #[clap(alias = "s")]
    Set {
        key: String,
        value: String,
        #[arg(long)]
        local: bool,
    },
    /// Unset a configuration value
    #[clap(alias = "u")]
    Unset { key: String },
    /// Open the configuration file in your default editor
    #[clap(alias = "e")]
    Edit,
}

#[derive(Subcommand, Debug)]
pub enum PluginCmd {
    List,
    Install { source: String },
    Remove { id: String },
    Enable { id: String },
    Disable { id: String },
    Run { hook: String },
}

#[derive(Debug, Subcommand)]
pub enum StackCmd {
    Init {
        name: String,
    },
    Branch {
        name: String,
        #[arg(long)]
        parent: Option<String>,
    },
    Adopt {
        parent: String,
    },
    Log,
    Next,
    Prev,
    Goto {
        branch: String,
    },
    Restack {
        #[arg(long)]
        continue_: bool,
        #[arg(long)]
        abort: bool,
        #[arg(long)]
        onto: Option<String>,
        #[arg(long)]
        autosquash: bool,
    },
    Submit {
        #[arg(long)]
        ready: bool,
    },
    Update,
    Status,
    Clean,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = sage_config::ConfigManager::new()?;
    let cfg = config.load()?;

    if cfg.auto_update {
        let _ = check_for_updates().await;
    }

    let cli = Cli::parse();
    let global_config = GlobalConfig::new(cli.json, cli.no_color);

    match cli.command {
        // Synchronous commands
        Command::Work(args) => cmd::work(&args, &global_config),
        Command::Sync(args) => cmd::sync(&args, &global_config),
        Command::List { relative } => cmd::list(relative, &global_config),
        Command::Config { op } => match op {
            ConfigCmd::List => cmd::config_list(&global_config),
            ConfigCmd::Get { key } => cmd::config_get(&key, &global_config),
            ConfigCmd::Set { key, value, local } => {
                cmd::config_set(&key, &value, local, &global_config)
            }
            ConfigCmd::Unset { key } => cmd::config_unset(&key, &global_config),
            ConfigCmd::Edit => cmd::config_edit(&global_config),
        },
        Command::Log => cmd::log(&global_config),

        Command::Stack { op } => match op {
            StackCmd::Init { name } => cmd::stack_init(&name, &global_config),
            StackCmd::Next => cmd::stack_navigate::down(&global_config),
            StackCmd::Prev => cmd::stack_navigate::up(&global_config),
            StackCmd::Adopt { parent } => cmd::stack_adopt(&parent, &global_config),
            _ => todo!(),
        },

        Command::Ui => cmd::ui(&global_config),

        // Asynchronous commands
        Command::Save(args) => cmd::save(&args, &global_config).await,
        Command::Share(args) => cmd::share(&args, &global_config),

        // Placeholder commands
        Command::Dash { watch } => todo!("dash watch={watch}"),
        Command::Clean { remote, dry_run } => todo!("clean r={remote} d={dry_run}"),
        Command::Undo { id } => cmd::undo(id, &global_config),
        Command::History => cmd::history(&global_config),
        Command::Resolve => todo!("resolve"),
        Command::Stats { since } => todo!("stats {:?}", since),
        Command::Doctor { fix } => todo!("doctor fix={fix}"),
        Command::Completion { shell } => todo!("completion {shell}"),
        Command::Plugin { cmd } => todo!("plugin {:?}", cmd),
        Command::Tips => todo!("tips"),
    }
}
