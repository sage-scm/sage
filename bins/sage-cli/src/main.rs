use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod cmd;

/// 🌿 Sage -- Burning away git complexities
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // ────────────────────────────────── Core workflow ─────────────────────────
    /// Smart create / checkout a branch
    Work { branch: String },

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
        cmd: StackCmd,
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
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
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
#[derive(Subcommand)]
pub enum StackCmd {
    Init,
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

<<<<<<< HEAD
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        // Synchronous commands
        Command::Work { branch } => cmd::work(&branch),

        // Asynchronous commands
        Command::Save(args) => cmd::save(&args).await,

        // Placeholder commands
=======
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Work { branch } => cmd::work(&branch),
        Command::Save(args) => todo!("save {:?}", args),
>>>>>>> origin/main
        Command::Sync(args) => todo!("sync {:?}", args),
        Command::Share(args) => todo!("share {:?}", args),
        Command::Dash { watch } => todo!("dash watch={watch}"),
        Command::Clean { remote, dry_run } => todo!("clean r={remote} d={dry_run}"),
        Command::Undo { id } => todo!("undo {:?}", id),
        Command::History => todo!("history"),
        Command::Resolve => todo!("resolve"),
        Command::Stats { since } => todo!("stats {:?}", since),
        Command::Doctor { fix } => todo!("doctor fix={fix}"),
        Command::Config { op } => todo!("config {:?}", op),
        Command::Completion { shell } => todo!("completion {shell}"),
        Command::Plugin { cmd } => todo!("plugin {:?}", cmd),
        #[cfg(feature = "stack")]
        Command::Stack { cmd } => todo!("stack {:?}", cmd),
        #[cfg(feature = "ai")]
        Command::Tips => todo!("tips"),
        #[cfg(feature = "tui")]
        Command::Ui => todo!("ui"),
    }
}
