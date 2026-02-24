use clap::Parser;
mod client;
mod commands;
mod io;

#[derive(Parser, Debug)]
enum Commands {
    RebaseStack {
        #[arg(short, long)]
        base: Option<String>,

        upstream: String,
        // branch that needs to be rebased, if not provided, it will be the current branch.
        branch: Option<String>,
    },
    Unstack {
        from: String,
    },
}

trait ResultExt<T, E> {
    fn unwrap_or_exit(self, error_message: &str) -> T;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_or_exit(self, error_message: &str) -> T {
        match self {
            Ok(result) => result,
            Err(_) => {
                eprintln!("{}", error_message);
                std::process::exit(1);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct GitPolyp {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let args: GitPolyp = GitPolyp::parse();
    match args.command {
        commands::Commands::RebaseStack {
            base,
            upstream,
            branch,
            abort,
            undo,
            _continue,
            verbose,
        } => {
            commands::rebase_stack::run(_continue, abort, undo, base, upstream, branch, verbose);
        }
        commands::Commands::Unstack { from } => {
            println!("Unstack command called with from: {}", from);
            // Here you would implement the logic to perform the unstack operation
        }
    }
}
