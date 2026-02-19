use clap::Parser;
mod client;
mod io;
mod stack;
use io::Decorate;

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

    let failed_to_build_stack = format!(
        "{}
        Please make sure the provided upstream and branch are correct and exist, and that the --base option, if provided, is correct and exists.
        If the error persists, please check the state of your repository and try to fix it before running this command again.
        ",
        "Failed to build the stack of commits to rebase.".to_string().deco_as_error()
    );

    let rebase_stack = match &base {
        Some(base) => {
            let failed_to_verify_base = format!(
                "{}
                Please make sure the provided {} option is correct and exists.
                ",
                "Failed to verify the --base option content.".to_string().deco_as_error(),
                "--base".to_string().deco_as_command()
            );
            let base_ref = client::rev_parse(base).unwrap_or_exit(&failed_to_verify_base);
            let base_upstream_base = client::merge_base(base, &upstream)
                .unwrap_or_exit(&failed_to_verify_base);
            if base_upstream_base != merge_base_ref {
                let error_message = format!(
                    "{}
                    Please provide a {} base option that is a descendant of the upstream.
                    ",
                    "The provided --base option is not a descendant of the upstream.".to_string().deco_as_error(),
                    "--base".to_string().deco_as_command()
                );
                eprintln!("{}", &error_message);
                std::process::exit(1);
            }
            println!("As base ref {:?} is a descendant of the upstream, the stack will be built from {:?} to {:?}.",
                base_ref, merge_base_ref, branch_ref);

            stack::Stack::new(&base_ref, &branch_ref, &upstream_ref)
        },
        None => stack::Stack::new(&merge_base_ref, &branch_ref, &upstream_ref),
    }
    .unwrap_or_exit(&failed_to_build_stack);

    println!("Stack\n-----\n{}", rebase_stack.format());
    if false
        == io::YNQuestion::new("Do you want to rebase this stack?".to_string())
            .ask()
            .unwrap_or(false)
    {
        println!("Aborting rebase.");
        std::process::exit(0);
    }
    check_if_in_progress();

    let failed_to_clean_stack = format!(
        "{}
        Please run `{}` to clean the stack.
        If the error persists, please try to remove the {} file manually.
        ",
        "Failed to clean the stack state !"
            .to_string()
            .deco_as_error(),
        "git-polyp rebase-stack --abort"
            .to_string()
            .deco_as_command(),
        ".git/polyp/stack.json".to_string().deco_as_path()
    );

    match rebase_stack.persist() {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Failed intiialize the rebase. Cleaning...");
            stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
        }
    }
    println!("Stack persisted. Starting rebase...");
    let rebase_error = format!(
        "{}
        Please run `{}` to clean the stack.
        If the error persists, please try to remove the {} file manually.
        ",
        "Failed to clean the stack !".to_string().deco_as_error(),
        "git-polyp rebase-stack --abort"
            .to_string()
            .deco_as_command(),
        ".git/polyp/stack.json".to_string().deco_as_path()
    );

    match perform_rebase(&rebase_stack, &upstream_ref) {
        Ok(()) => (),
        Err(()) => {
            eprintln!("Failed to perform the rebase. Cleaning...");
            stack::Stack::clean().unwrap_or_exit(&rebase_error);
            std::process::exit(1);
        }
    }

    let failed_to_reset_stack_as_before = format!(
        "{}
        Please run `{}` to reset the stack to its previous state.
        If the error persists, please try to restore the stack file with the backup file created during the rebase process, or remove the stack file manually if you don't have a backup.
        ",
        "Failed to reset the stack as it was before !".to_string().deco_as_error(),
        "git-polyp rebase-stack --reset"
            .to_string()
            .deco_as_command(),
    );

    match set_new_stack(&rebase_stack, &upstream_ref) {
        Ok(()) => (),
        Err(_) => {
            eprintln!("Failed to set the new stack. Cleaning...");
            rebase_stack
                .apply()
                .unwrap_or_exit(&failed_to_reset_stack_as_before);
            std::process::exit(1);
        }
    }

    let branches = rebase_stack.branches();
    let branches_str = branches
        .iter()
        .map(|branch| format!("{}", branch))
        .collect::<Vec<String>>()
        .join(" ");

    let push_command = format!("git push origin {}", branches_str).deco_as_command();
    let push_question = format!(
        "Rebase successful. Do you want to push the new branches to '{}' ?
        You can also push them later with the following command:
        {}",
        "origin".to_string().deco_as_command(),
        push_command
    );

    if false == io::YNQuestion::new(push_question).ask().unwrap_or(false) {
        stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
        std::process::exit(0);
    }

    let failed_to_push_branches = format!(
        "{}
        Please push the branches manually with the following command:
        {}
        ",
        "Failed to push the new branches to the remote repository."
            .to_string()
            .deco_as_error(),
        push_command
    );

    match client::push_branches("origin", branches) {
        Ok(()) => (),
        Err(_) => {
            eprintln!("{}", &failed_to_push_branches);
            stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
            std::process::exit(1);
        }
    }

    stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
}

fn perform_rebase(rebase_stack: &stack::Stack, upstream_ref: &str) -> Result<(), ()> {
    client::checkout(upstream_ref).map_err(|_| ())?;
    client::cherry_pick(rebase_stack.base_ref(), rebase_stack.top_ref()).map_err(|_| ())?;

    Ok(())
}

fn set_new_stack(rebase_stack: &stack::Stack, upstream_ref: &str) -> Result<(), ()> {
    let new_head_ref = get_branch_ref(None);
    let new_stack = stack::Stack::new(upstream_ref, &new_head_ref, upstream_ref).map_err(|_| ())?;
    let new_stack = new_stack
        .apply_branches_from(&rebase_stack)
        .map_err(|_| ())?;
    println!("New stack\n-----\n{}", new_stack.format());
    new_stack.apply().map_err(|_| ())?;

    Ok(())
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
    let failed_to_find_branch = format!(
        "{}",
        "Failed to find the current branch name. Make sure you are in a git repository."
            .to_string()
            .deco_as_error()
    );
    let branch = match branch {
        Some(branch) => branch,
        None => client::current_branch().unwrap_or_exit(&failed_to_find_branch),
    };
    client::rev_parse(&branch).unwrap_or_exit(&failed_to_find_branch)
}
