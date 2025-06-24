use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod cmd;

/// 🌿 Sage -- Burning away git complexities
#[derive(Parser)]
#[command(author, version, about = "🌿 Sage — A Git workflow tool for managing branches and commits", long_about = None)]
pub struct Cli {
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
    List(ListArgs),

    /// Config management
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
    #[cfg(feature = "stack")]
    Stack {
        #[command(subcommand)]
        op: StackCmd,
    },

    // ──────────────────────────────── AI extras ──────────────────────────────
    #[cfg(feature = "ai")]
    Tips,

    // ──────────────────────────────── TUI mode ───────────────────────────────
    #[cfg(feature = "tui")]
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
pub struct ListArgs {
    /// Show above and below stats
    #[arg(short, long)]
    stats: bool,
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    #[arg(long)]
    continue_: bool,
    #[arg(long)]
    abort: bool,
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
    List,
    /// Get the value of a specific configuration key
    Get { key: String },
    /// Set a configuration value
    Set {
        key: String,
        value: String,
        #[arg(long)]
        local: bool,
    },
    /// Unset a configuration value
    Unset { key: String },
    /// Open the configuration file in your default editor
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

#[cfg(feature = "stack")]
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
    let cli = Cli::parse();
    match cli.command {
        // Synchronous commands
        Command::Work(args) => cmd::work(&args),
        Command::Sync(args) => cmd::sync(&args),
        Command::List(args) => cmd::list(&args),
        Command::Config { op } => match op {
            ConfigCmd::List => cmd::config_list(),
            ConfigCmd::Get { key } => cmd::config_get(&key),
            ConfigCmd::Set { key, value, local } => cmd::config_set(&key, &value, local),
            ConfigCmd::Unset { key } => cmd::config_unset(&key),
            ConfigCmd::Edit => cmd::config_edit(),
        },
        Command::Log => cmd::log(),

        #[cfg(feature = "stack")]
        Command::Stack { op } => match op {
            StackCmd::Init { name } => cmd::stack_init(&name),
            StackCmd::Next => cmd::stack_navigate::down(),
            StackCmd::Prev => cmd::stack_navigate::up(),
            _ => todo!(),
        },

        #[cfg(feature = "tui")]
        Command::Ui => cmd::ui(),

        // Asynchronous commands
        Command::Save(args) => cmd::save(&args).await,

        // Placeholder commands
        Command::Share(args) => todo!("share {:?}", args),
        Command::Dash { watch } => todo!("dash watch={watch}"),
        Command::Clean { remote, dry_run } => todo!("clean r={remote} d={dry_run}"),
        Command::Undo { id } => todo!("undo {:?}", id),
        Command::History => todo!("history"),
        Command::Resolve => todo!("resolve"),
        Command::Stats { since } => todo!("stats {:?}", since),
        Command::Doctor { fix } => todo!("doctor fix={fix}"),
        Command::Completion { shell } => todo!("completion {shell}"),
        Command::Plugin { cmd } => todo!("plugin {:?}", cmd),
        #[cfg(feature = "ai")]
        Command::Tips => todo!("tips"),
    }
}
