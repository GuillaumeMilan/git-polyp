use clap::Parser;
mod client;
mod commands;
mod error;
mod io;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct GitPolyp {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() -> std::process::ExitCode {
    use std::process::ExitCode;

    let args: GitPolyp = GitPolyp::parse();
    let result = match args.command {
        commands::Commands::Rebase {
            base,
            upstream,
            branch,
            abort,
            undo,
            _continue,
            verbose,
        } => commands::rebase_stack::run(_continue, abort, undo, base, upstream, branch, verbose),
        commands::Commands::Unstack { from } => {
            println!("Unstack command called with from: {}", from);
            // Here you would implement the logic to perform the unstack operation
            Ok(())
        }
        commands::Commands::Worktree(args) => commands::worktree::run(args, false),
        commands::Commands::Completions { shell } => {
            commands::completions::run(shell);
            Ok(())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error::AppError::UserAborted) => ExitCode::SUCCESS,
        Err(error::AppError::SilentFailure) => ExitCode::FAILURE,
        Err(error::AppError::Message(msg)) => {
            eprintln!("{}", msg);
            ExitCode::FAILURE
        }
    }
}
