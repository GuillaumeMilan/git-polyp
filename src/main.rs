use clap::Parser;
mod client;
mod io;
mod stack;

#[derive(Parser, Debug)]
enum Commands {
    RebaseStack {
        #[arg(short, long)]
        onto: Option<String>,

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
    command: Commands,
}

fn main() {
    let args: GitPolyp = GitPolyp::parse();
    match args.command {
        Commands::RebaseStack {
            onto,
            upstream,
            branch,
        } => {
            rebase_stack(onto, upstream, branch);
        }
        Commands::Unstack { from } => {
            println!("Unstack command called with from: {}", from);
            // Here you would implement the logic to perform the unstack operation
        }
    }
}

fn rebase_stack(onto: Option<String>, upstream: String, branch: Option<String>) {
    let is_in_git_repo = client::is_in_repo()
        .unwrap_or_exit("Not a git repository. Please run this command inside a git repository.");
    if !is_in_git_repo {
        eprintln!("Not a git repository. Please run this command inside a git repository.");
        std::process::exit(1);
    }
    check_if_in_progress();
    let branch = ensure_branch(branch);
    let merge_base = client::merge_base(&upstream, &branch)
        .unwrap_or_exit("Failed to find the merge base between the upstream and the branch.");

    let rebase_stack = match &onto {
        Some(onto) => stack::Stack::new(onto, &branch),
        None => stack::Stack::new(&merge_base, &branch),
    }
    .unwrap_or_exit("Failed to build the stack of commits to rebase.");

    println!("Stack\n-----\n{}", rebase_stack.format());
    if !io::YNQuestion::new("Do you want to rebase this stack?".to_string())
        .ask()
        .unwrap_or_exit("Failed to read user input.")
    {
        println!("Aborting rebase.");
        std::process::exit(0);
    }
    check_if_in_progress();

    match rebase_stack.persist() {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Failed intiialize the rebase. Cleaning...");
            stack::Stack::clean().unwrap_or_exit(
                "Failed to clean the stack.
                Please run `git-polyp rebase-stack --abort` to clean the stack.
                If the error persists, please try to remove the .git/polyp/stack file manually.
                ",
            );
        }
    }

    println!(
        "Rebasing {} onto {}",
        branch,
        onto.unwrap_or_else(|| "default upstream".to_string())
    );
}

fn check_if_in_progress() {
    let in_progress = stack::Stack::exists().unwrap_or_exit("Failed to access to git-polyp private directory. Make sure you have the right to access the .git directory.");
    if in_progress {
        eprintln!("A rebase is already in progress.
        Continue it with `git-polyp rebase-stack --continue`.
        Abort it without doing any modification to the repository with `git-polyp rebase-stack --abort`.
        Abort by reseting the stack of commit to its version before any operation with `git-polyp rebase-stack --reset`.
        ");
        std::process::exit(1);
    };
}

fn ensure_branch(branch: Option<String>) -> String {
    const FAILED_TO_FIND_BRANCH: &str =
        "Failed to find the current branch name. Make sure you are in a git repository.";
    match branch {
        Some(branch) => branch,
        None => client::rev_parse().unwrap_or_exit(FAILED_TO_FIND_BRANCH),
    }
}
