mod messages;
mod stack;
use crate::ResultExt;
use crate::client;
use crate::io::{Decorate, YNQuestion};

pub fn run(
    _continue: bool,
    abort: bool,
    undo: bool,
    base: Option<String>,
    upstream: Option<String>,
    branch: Option<String>,
) {
    match (_continue, abort, undo, upstream) {
        (true, false, false, None) => run_continue(),
        (false, true, false, None) => run_abort(),
        (false, false, true, None) => run_undo(),
        (false, false, false, Some(upstream)) => run_normal(base, upstream, branch),
        _ => {
            // We should never enter this case (as clap should prevent it with the
            // conflicts_with_all and arg_required_else_help settings)
            eprintln!("{}", messages::error::invalid_arguments());
            // TODO execute equivalent of rebase-stack --help
            std::process::exit(1);
        }
    }
}

fn run_continue() {
    match stack::Stack::load() {
        Ok(rebase_stack) => {
            unwrap_rebase_result(continue_rebase(), &rebase_stack);
            perform_continue(&rebase_stack);
        }
        Err(_) => {
            eprintln!("{}", messages::error::no_rebase_in_progress());
            std::process::exit(1);
        }
    }
}

fn run_abort() {
    match stack::Stack::load() {
        Ok(_) => {
            let failed_to_clean_stack = messages::error::failed_to_clean_stack();
            stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
        }
        Err(_) => {
            eprintln!("{}", messages::error::no_rebase_in_progress());
            std::process::exit(1);
        }
    }
}

fn run_undo() {
    println!("Not implemented yet!");
    println!("{}", messages::info::undoing_rebase());
    let no_rebase_in_progress = messages::error::no_rebase_in_progress();
    let failed_to_undo_rebase = messages::error::failed_to_undo_rebase();
    let stack = stack::Stack::load().unwrap_or_exit(&no_rebase_in_progress);
    stack.apply().unwrap_or_exit(&failed_to_undo_rebase);
    stack::Stack::clean().unwrap_or_exit(&failed_to_undo_rebase);
    println!("{}", messages::info::rebase_undone());
}

fn run_normal(base: Option<String>, upstream: String, branch: Option<String>) {
    let is_in_git_repo = client::is_in_repo().unwrap_or_exit(messages::error::NOT_IN_GIT_REPO);
    if !is_in_git_repo {
        eprintln!("{}", messages::error::NOT_IN_GIT_REPO);
        std::process::exit(1);
    }
    check_if_in_progress();
    let branch_ref = get_branch_ref(branch);
    let upstream_ref =
        client::rev_parse(&upstream).unwrap_or_exit(messages::error::failed_to_verify_upstream());
    let merge_base_ref = client::merge_base(&upstream_ref, &branch_ref)
        .unwrap_or_exit(messages::error::failed_to_find_merge_base());

    let failed_to_build_stack = messages::error::failed_to_build_stack();

    let rebase_stack = match &base {
        Some(base) => {
            let failed_to_verify_base = messages::error::failed_to_verify_base();
            let base_ref = client::rev_parse(base).unwrap_or_exit(&failed_to_verify_base);
            let base_upstream_base = client::merge_base(base, &upstream)
                .unwrap_or_exit(&failed_to_verify_base);
            if base_upstream_base != merge_base_ref {
                let error_message = messages::error::base_not_descendant_of_upstream();
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

    println!(
        "\n\n{}\n\n",
        rebase_stack.format_with_title("Current stack of commits")
    );
    if false
        == YNQuestion::new(messages::info::ask_rebase_confirmation())
            .ask()
            .unwrap_or(false)
    {
        println!("{}", messages::info::aborting_rebase());
        std::process::exit(0);
    }
    check_if_in_progress();

    let failed_to_clean_stack = messages::error::failed_to_clean_stack();

    match rebase_stack.persist() {
        Ok(_) => (),
        Err(_) => {
            eprintln!("{}", messages::info::failed_initialize_rebase());
            stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
        }
    }
    unwrap_rebase_result(perform_rebase(&rebase_stack), &rebase_stack);
    perform_continue(&rebase_stack);
}

fn perform_continue(rebase_stack: &stack::Stack) {
    let failed_to_reset_stack_as_before = messages::error::failed_to_reset_stack_as_before();

    match set_new_stack(&rebase_stack) {
        Ok(()) => (),
        Err(_) => {
            eprintln!("{}", messages::info::failed_to_set_new_stack());
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

    let push_command =
        format!("> git push origin --force-with-lease {}", branches_str).deco_as_command();
    let push_question = messages::info::ask_push_confirmation(push_command.to_string());
    let failed_to_clean_stack = messages::error::failed_to_clean_stack();

    let end_on_branch = rebase_stack.top_branch();

    match end_on_branch {
        Some(branch) => {
            client::switch(&branch)
                .map_err(|_| {
                    eprintln!("{}", messages::error::failed_to_switch_to_branch(&branch));
                })
                // In case of error just ignore and continue, the user will endup on the top commit of the stack
                .unwrap_or(());
        }
        None => (),
    }

    if false == YNQuestion::new(push_question).ask().unwrap_or(false) {
        stack::Stack::clean().unwrap_or_exit(&failed_to_clean_stack);
        std::process::exit(0);
    }

    let failed_to_push_branches = messages::error::failed_to_push_branches(&push_command);

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

enum RebaseResult {
    Success,
    CherryPickConflict,
}

fn perform_rebase(rebase_stack: &stack::Stack) -> Result<RebaseResult, ()> {
    client::checkout(&rebase_stack.destination_ref).map_err(|_| ())?;
    match client::cherry_pick(rebase_stack.base_ref(), rebase_stack.top_ref()) {
        Ok(()) => Ok(RebaseResult::Success),
        Err(_) => Ok(RebaseResult::CherryPickConflict),
    }
}

fn continue_rebase() -> Result<RebaseResult, ()> {
    match client::cherry_pick_continue() {
        Ok(()) => Ok(RebaseResult::Success),
        Err(_) => Ok(RebaseResult::CherryPickConflict),
    }
}

fn unwrap_rebase_result(
    result_with_rebase_result: Result<RebaseResult, ()>,
    rebase_stack: &stack::Stack,
) -> () {
    match result_with_rebase_result {
        Ok(RebaseResult::Success) => (),
        Ok(RebaseResult::CherryPickConflict) => {
            eprintln!("{}", messages::info::resolve_conflicts_and_continue());
            std::process::exit(1);
        }
        Err(()) => {
            let rebase_error = messages::error::failed_to_clean_stack_after_rebase();
            eprintln!("{}", messages::info::failed_to_perform_rebase());
            rebase_stack.apply().unwrap_or_exit(&rebase_error);
            stack::Stack::clean().unwrap_or_exit(&rebase_error);
            std::process::exit(1);
        }
    }
}

fn set_new_stack(rebase_stack: &stack::Stack) -> Result<(), ()> {
    let new_head_ref = get_branch_ref(None);
    let new_stack = stack::Stack::new(
        &rebase_stack.destination_ref,
        &new_head_ref,
        &rebase_stack.destination_ref,
    )
    .map_err(|_| ())?;
    let new_stack = new_stack
        .apply_branches_from(&rebase_stack)
        .map_err(|_| ())?;
    println!(
        "\n\n{}\n\n",
        rebase_stack.format_with_title("New stack of rebased commits")
    );
    new_stack.apply().map_err(|_| ())?;

    Ok(())
}

fn check_if_in_progress() {
    let in_progress =
        stack::Stack::exists().unwrap_or_exit(messages::error::FAILED_TO_ACCESS_GIT_POLYP_DIR);
    if in_progress {
        eprintln!("{}", messages::info::rebase_in_progress());
        std::process::exit(1);
    };
}

fn get_branch_ref(branch: Option<String>) -> String {
    let failed_to_find_branch = messages::error::failed_to_find_branch();
    let branch = match branch {
        Some(branch) => branch,
        None => client::current_branch().unwrap_or_exit(&failed_to_find_branch),
    };
    client::rev_parse(&branch).unwrap_or_exit(&failed_to_find_branch)
}
