use clap::Parser;
mod client;
mod io;
mod stack;

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
    command: Commands,
}

fn main() {
    let args: GitPolyp = GitPolyp::parse();
    match args.command {
        Commands::RebaseStack {
            base,
            upstream,
            branch,
        } => {
            rebase_stack(base, upstream, branch);
        }
        Commands::Unstack { from } => {
            println!("Unstack command called with from: {}", from);
            // Here you would implement the logic to perform the unstack operation
        }
    }
}

fn rebase_stack(base: Option<String>, upstream: String, branch: Option<String>) {
    let is_in_git_repo = client::is_in_repo()
        .unwrap_or_exit("Not a git repository. Please run this command inside a git repository.");
    if !is_in_git_repo {
        eprintln!("Not a git repository. Please run this command inside a git repository.");
        std::process::exit(1);
    }
    check_if_in_progress();
    let branch_ref = get_branch_ref(branch);
    let upstream_ref = client::rev_parse(&upstream).unwrap_or_exit(
        "Failed to verify the upstream. Make sure the provided upstream is correct and exists.",
    );
    let merge_base_ref = client::merge_base(&upstream_ref, &branch_ref)
        .unwrap_or_exit("Failed to find the merge base between the upstream and the branch.");

    let rebase_stack = match &base {
        Some(base) => {
            const FAILED_TO_VERIFY_BASE: &str = "Failed to verify the --base option content. Make sure the provided commit exists.";
            let base_ref = client::rev_parse(base).unwrap_or_exit(FAILED_TO_VERIFY_BASE);
            let base_upstream_base = client::merge_base(base, &upstream)
                .unwrap_or_exit(FAILED_TO_VERIFY_BASE);
            if base_upstream_base != merge_base_ref {
                eprintln!("The provided --base option is not a descendant of the upstream. Please provide an --base option that is a descendant of the upstream.");
                std::process::exit(1);
            }
            println!("As base ref {:?} is a descendant of the upstream, the stack will be built from {:?} to {:?}.",
                base_ref, merge_base_ref, branch_ref);

            stack::Stack::new(&base_ref, &branch_ref, &upstream_ref)
        },
        None => stack::Stack::new(&merge_base_ref, &branch_ref, &upstream_ref),
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
    println!("Stack persisted. Starting rebase... Not implemented yet");
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

fn get_branch_ref(branch: Option<String>) -> String {
    const FAILED_TO_FIND_BRANCH: &str =
        "Failed to find the current branch name. Make sure you are in a git repository.";
    let branch = match branch {
        Some(branch) => branch,
        None => client::current_branch().unwrap_or_exit(FAILED_TO_FIND_BRANCH),
    };
    client::rev_parse(&branch).unwrap_or_exit(FAILED_TO_FIND_BRANCH)
}
