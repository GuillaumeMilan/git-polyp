mod messages;
mod stack;
use crate::client;
use crate::error::AppError;
use crate::io::{Decorate, YNQuestion};

pub fn run(
    _continue: bool,
    abort: bool,
    undo: bool,
    base: Option<String>,
    upstream: Option<String>,
    branch: Option<String>,
    verbose: bool,
) -> Result<(), AppError> {
    match (_continue, abort, undo, upstream) {
        (true, false, false, None) => run_continue(verbose),
        (false, true, false, None) => run_abort(verbose),
        (false, false, true, None) => run_undo(verbose),
        (false, false, false, Some(upstream)) => run_normal(base, upstream, branch, verbose),
        _ => {
            // We should never enter this case (as clap should prevent it with the
            // conflicts_with_all and arg_required_else_help settings)
            Err(AppError::Message(
                messages::error::invalid_arguments().to_string(),
            ))
            // TODO execute equivalent of rebase --help
        }
    }
}

fn run_continue(verbose: bool) -> Result<(), AppError> {
    match stack::Stack::load(verbose) {
        Ok(rebase_stack) => {
            unwrap_rebase_result(continue_rebase(verbose), &rebase_stack, verbose)?;
            perform_continue(&rebase_stack, verbose)
        }
        Err(_) => Err(AppError::Message(
            messages::error::no_rebase_in_progress().to_string(),
        )),
    }
}

fn run_abort(verbose: bool) -> Result<(), AppError> {
    match stack::Stack::load(verbose) {
        Ok(_) => {
            let failed_to_clean_stack = messages::error::failed_to_clean_stack();
            stack::Stack::clean(verbose)
                .map_err(|_| AppError::Message(failed_to_clean_stack.to_string()))?;
            Ok(())
        }
        Err(_) => Err(AppError::Message(
            messages::error::no_rebase_in_progress().to_string(),
        )),
    }
}

fn run_undo(verbose: bool) -> Result<(), AppError> {
    println!("Not implemented yet!");
    println!("{}", messages::info::undoing_rebase());
    let no_rebase_in_progress = messages::error::no_rebase_in_progress();
    let failed_to_undo_rebase = messages::error::failed_to_undo_rebase();
    let stack = stack::Stack::load(verbose)
        .map_err(|_| AppError::Message(no_rebase_in_progress.to_string()))?;
    stack
        .apply(verbose)
        .map_err(|_| AppError::Message(failed_to_undo_rebase.to_string()))?;
    stack::Stack::clean(verbose)
        .map_err(|_| AppError::Message(failed_to_undo_rebase.to_string()))?;
    println!("{}", messages::info::rebase_undone());
    Ok(())
}

fn run_normal(
    base: Option<String>,
    upstream: String,
    branch: Option<String>,
    verbose: bool,
) -> Result<(), AppError> {
    let is_in_git_repo = client::is_in_repo(verbose)
        .map_err(|_| AppError::Message(messages::error::NOT_IN_GIT_REPO.to_string()))?;
    if !is_in_git_repo {
        return Err(AppError::Message(
            messages::error::NOT_IN_GIT_REPO.to_string(),
        ));
    }
    check_if_in_progress(verbose)?;
    let branch_ref = get_branch_ref(branch, verbose)?;
    let upstream_ref = client::rev_parse(&upstream, verbose)
        .map_err(|_| AppError::Message(messages::error::failed_to_verify_upstream().to_string()))?;
    let merge_base_ref = client::merge_base(&upstream_ref, &branch_ref, verbose)
        .map_err(|_| AppError::Message(messages::error::failed_to_find_merge_base().to_string()))?;

    let failed_to_build_stack = messages::error::failed_to_build_stack();

    let rebase_stack = match &base {
        Some(base) => {
            let failed_to_verify_base = messages::error::failed_to_verify_base();
            let base_ref = client::rev_parse(base, verbose)
                .map_err(|_| AppError::Message(failed_to_verify_base.to_string()))?;
            let base_upstream_base = client::merge_base(base, &upstream, verbose)
                .map_err(|_| AppError::Message(failed_to_verify_base.to_string()))?;
            if base_upstream_base != merge_base_ref {
                return Err(AppError::Message(
                    messages::error::base_not_descendant_of_upstream().to_string(),
                ));
            }
            println!("As base ref {:?} is a descendant of the upstream, the stack will be built from {:?} to {:?}.",
                base_ref, merge_base_ref, branch_ref);

            stack::Stack::new(&base_ref, &branch_ref, &upstream_ref, verbose)
        }
        None => stack::Stack::new(&merge_base_ref, &branch_ref, &upstream_ref, verbose),
    }
    .map_err(|_| AppError::Message(failed_to_build_stack.to_string()))?;

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
        return Err(AppError::UserAborted);
    }
    check_if_in_progress(verbose)?;

    let failed_to_clean_stack = messages::error::failed_to_clean_stack();

    match rebase_stack.persist(verbose) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("{}", messages::info::failed_initialize_rebase());
            stack::Stack::clean(verbose)
                .map_err(|_| AppError::Message(failed_to_clean_stack.to_string()))?;
        }
    }
    unwrap_rebase_result(
        perform_rebase(&rebase_stack, verbose),
        &rebase_stack,
        verbose,
    )?;
    perform_continue(&rebase_stack, verbose)
}

fn perform_continue(rebase_stack: &stack::Stack, verbose: bool) -> Result<(), AppError> {
    let failed_to_reset_stack_as_before = messages::error::failed_to_reset_stack_as_before();

    match set_new_stack(&rebase_stack, verbose) {
        Ok(()) => (),
        Err(_) => {
            eprintln!("{}", messages::info::failed_to_set_new_stack());
            rebase_stack
                .apply(verbose)
                .map_err(|_| AppError::Message(failed_to_reset_stack_as_before.to_string()))?;
            return Err(AppError::Message(
                messages::info::failed_to_set_new_stack().to_string(),
            ));
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
            client::switch(&branch, verbose)
                .map_err(|_| {
                    eprintln!("{}", messages::error::failed_to_switch_to_branch(&branch));
                })
                // In case of error just ignore and continue, the user will endup on the top commit of the stack
                .unwrap_or(());
        }
        None => (),
    }

    if false == YNQuestion::new(push_question).ask().unwrap_or(false) {
        stack::Stack::clean(verbose)
            .map_err(|_| AppError::Message(failed_to_clean_stack.to_string()))?;
        return Err(AppError::UserAborted);
    }

    let failed_to_push_branches = messages::error::failed_to_push_branches(&push_command);

    match client::push_branches("origin", branches, verbose) {
        Ok(()) => (),
        Err(_) => {
            stack::Stack::clean(verbose)
                .map_err(|_| AppError::Message(failed_to_clean_stack.to_string()))?;
            return Err(AppError::Message(failed_to_push_branches.to_string()));
        }
    }

    stack::Stack::clean(verbose)
        .map_err(|_| AppError::Message(failed_to_clean_stack.to_string()))?;
    Ok(())
}

enum RebaseResult {
    Success,
    CherryPickConflict,
}

fn perform_rebase(rebase_stack: &stack::Stack, verbose: bool) -> Result<RebaseResult, ()> {
    client::switch_d(&rebase_stack.destination_ref, verbose).map_err(|_| ())?;
    match client::cherry_pick(rebase_stack.base_ref(), rebase_stack.top_ref(), verbose) {
        Ok(()) => Ok(RebaseResult::Success),
        Err(_) => Ok(RebaseResult::CherryPickConflict),
    }
}

fn continue_rebase(verbose: bool) -> Result<RebaseResult, ()> {
    match client::cherry_pick_continue(verbose) {
        Ok(()) => Ok(RebaseResult::Success),
        Err(_) => Ok(RebaseResult::CherryPickConflict),
    }
}

fn unwrap_rebase_result(
    result_with_rebase_result: Result<RebaseResult, ()>,
    rebase_stack: &stack::Stack,
    verbose: bool,
) -> Result<(), AppError> {
    match result_with_rebase_result {
        Ok(RebaseResult::Success) => Ok(()),
        Ok(RebaseResult::CherryPickConflict) => Err(AppError::Message(
            messages::info::resolve_conflicts_and_continue().to_string(),
        )),
        Err(()) => {
            let rebase_error = messages::error::failed_to_clean_stack_after_rebase();
            eprintln!("{}", messages::info::failed_to_perform_rebase());
            rebase_stack
                .apply(verbose)
                .map_err(|_| AppError::Message(rebase_error.to_string()))?;
            stack::Stack::clean(verbose)
                .map_err(|_| AppError::Message(rebase_error.to_string()))?;
            Err(AppError::Message(
                messages::info::failed_to_perform_rebase().to_string(),
            ))
        }
    }
}

fn set_new_stack(rebase_stack: &stack::Stack, verbose: bool) -> Result<(), ()> {
    let new_head_ref = get_branch_ref(None, verbose).map_err(|_| ())?;
    let new_stack = stack::Stack::new(
        &rebase_stack.destination_ref,
        &new_head_ref,
        &rebase_stack.destination_ref,
        verbose,
    )
    .map_err(|_| ())?;
    let new_stack = new_stack
        .apply_branches_from(&rebase_stack, verbose)
        .map_err(|_| ())?;
    println!(
        "\n\n{}\n\n",
        new_stack.format_with_title("New stack of rebased commits")
    );
    new_stack.apply(verbose).map_err(|_| ())?;

    Ok(())
}

fn check_if_in_progress(verbose: bool) -> Result<(), AppError> {
    let in_progress = stack::Stack::exists(verbose).map_err(|_| {
        AppError::Message(messages::error::FAILED_TO_ACCESS_GIT_POLYP_DIR.to_string())
    })?;
    if in_progress {
        return Err(AppError::Message(
            messages::info::rebase_in_progress().to_string(),
        ));
    }
    Ok(())
}

fn get_branch_ref(branch: Option<String>, verbose: bool) -> Result<String, AppError> {
    let failed_to_find_branch = messages::error::failed_to_find_branch();
    let branch = match branch {
        Some(branch) => branch,
        None => client::current_branch(verbose)
            .map_err(|_| AppError::Message(failed_to_find_branch.to_string()))?,
    };
    client::rev_parse(&branch, verbose)
        .map_err(|_| AppError::Message(failed_to_find_branch.to_string()))
}
