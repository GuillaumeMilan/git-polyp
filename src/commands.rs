pub mod rebase_stack;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Commands {
    #[command(
            arg_required_else_help = true,
            group = clap::ArgGroup::new("action")
                .required(true)
                .args(&["upstream", "abort", "undo", "_continue"])
        )]
    RebaseStack {
        #[arg(short, long, conflicts_with_all = &["abort", "undo", "_continue"])]
        base: Option<String>,
        #[arg(long, conflicts_with_all = &["base", "undo", "_continue"])]
        abort: bool,
        #[arg(long, conflicts_with_all = &["base", "abort", "_continue"])]
        undo: bool,
        #[arg(long, conflicts_with_all = &["base", "abort", "undo"])]
        _continue: bool,
        #[arg(long)]
        verbose: bool,

        #[arg(conflicts_with_all = &["abort", "undo", "_continue"])]
        upstream: Option<String>,
        // branch that needs to be rebased, if not provided, it will be the current branch.
        branch: Option<String>,
    },
    Unstack {
        from: String,
    },
}
