use crate::io::Decorate;

pub mod error {
    use super::Decorate;

    pub const NOT_IN_GIT_REPO: &str =
        "Not a git repository. Please run this command inside a git repository.";
    pub const FAILED_TO_ACCESS_GIT_POLYP_DIR: &str = "Failed to access to git-polyp private directory. Make sure you have the right to access the .git directory.";

    pub fn invalid_arguments() -> String {
        format!(
            "{}
            See `{}` for more information.\n",
            "Invalid arguments provided.".deco_as_error(),
            "git-polyp rebase-stack --help".deco_as_command()
        )
    }

    pub fn failed_to_verify_upstream() -> &'static str {
        "Failed to verify the upstream. Make sure the provided upstream is correct and exists.\n"
    }

    pub fn failed_to_find_merge_base() -> &'static str {
        "Failed to find the merge base between the upstream and the branch.\n"
    }

    pub fn failed_to_verify_base() -> String {
        format!(
            "{}\n\
            Please make sure the provided {} option is correct and exists.\n",
            "Failed to verify the --base option content.".deco_as_error(),
            "--base".deco_as_command()
        )
    }

    pub fn base_not_descendant_of_upstream() -> String {
        format!(
            "{}\n\
            Please provide a {} base option that is a descendant of the upstream.\n",
            "The provided --base option is not a descendant of the upstream.".deco_as_error(),
            "--base".deco_as_command()
        )
    }

    pub fn failed_to_build_stack() -> String {
        format!(
            "{}\n\
            Please make sure the provided upstream and branch are correct and exist, and that the --base option, if provided, is correct and exists.\n\
            If the error persists, please check the state of your repository and try to fix it before running this command again.\n",
            "Failed to build the stack of commits to rebase.".deco_as_error()
        )
    }

    pub fn failed_to_clean_stack() -> String {
        format!(
            "{}\n\
            Please run `{}` to clean the stack.\n\
            If the error persists, please try to remove the {} file manually.\n",
            "Failed to clean the stack state!".deco_as_error(),
            "git-polyp rebase-stack --abort".deco_as_command(),
            ".git/polyp/stack.json".deco_as_path()
        )
    }

    pub fn failed_to_clean_stack_after_rebase() -> String {
        format!(
            "{}\n\
            Please run `{}` to clean the stack.\n\
            If the error persists, please try to remove the {} file manually.\n",
            "Failed to clean the stack!".deco_as_error(),
            "git-polyp rebase-stack --abort".deco_as_command(),
            ".git/polyp/stack.json".deco_as_path()
        )
    }

    pub fn no_rebase_in_progress() -> String {
        format!(
            "{}\n\
            Please make sure you have an ongoing rebase operation before running this command.\n",
            "No ongoing rebase operation found!".deco_as_error()
        )
    }

    pub fn failed_to_undo_rebase() -> String {
        format!(
            "{}\n\
            Please try rerunnning`{}`.\n\
            If the error persists, please try to restore the stack file with the backup file created during the rebase process, or remove the stack file manually if you don't have a backup.\n",
            "Failed to undo the ongoing rebase as it was before running any command!"
                .deco_as_error(),
            "git-polyp rebase-stack --undo".deco_as_command(),
        )
    }

    pub fn failed_to_reset_stack_as_before() -> String {
        format!(
            "{}\n\
            Please run `{}` to reset the stack to its previous state.\n\
            If the error persists, please try to restore the stack file with the backup file created during the rebase process, or remove the stack file manually if you don't have a backup.\n",
            "Failed to reset the stack as it was before!".deco_as_error(),
            "git-polyp rebase-stack --undo".deco_as_command(),
        )
    }

    pub fn failed_to_find_branch() -> String {
        "Failed to find the current branch name. Make sure you are in a git repository.\n"
            .deco_as_error()
            .to_string()
    }

    pub fn failed_to_push_branches(push_command: &str) -> String {
        format!(
            "{}\n\
            Please push the branches manually with the following command:\n\
            {}\n",
            "Failed to push the new branches to the remote repository.".deco_as_error(),
            push_command
        )
    }

    pub fn failed_to_switch_to_branch(branch: &str) -> String {
        let message = format!(
            "Failed to switch to the branch `{}`",
            branch.deco_as_command()
        )
        .deco_as_error();
        format!("{}\n", message)
    }
}

pub mod info {
    use super::Decorate;

    pub fn undoing_rebase() -> &'static str {
        "Reseting your repository as it was before starting any rebase operation.\n"
    }

    pub fn rebase_undone() -> &'static str {
        "Rebase undone. Your repository is now in the state it was before starting any rebase operation.\n"
    }

    pub fn rebase_in_progress() -> std::string::String {
        format!(
            "A rebase is already in progress.\n\
        Continue it with `{}`.\n\
        Abort it without doing any modification to the repository with `{}`.\n\
        Abort by reseting the stack of commit to its version before any operation with `{}`.\n",
            "git-polyp rebase-stack --continue".deco_as_command(),
            "git-polyp rebase-stack --abort".deco_as_command(),
            "git-polyp rebase-stack --undo".deco_as_command()
        )
    }

    pub fn ask_rebase_confirmation() -> String {
        "Do you want to rebase this stack?".to_string()
    }

    pub fn ask_push_confirmation(push_command: String) -> String {
        format!(
            "Rebase successful. Do you want to push the new branches to '{}' ?\n\n\
            You can also push them later with the following command:\n\
            {}\n\n\
            Push now ?",
            "origin".deco_as_command(),
            push_command
        )
    }

    pub fn aborting_rebase() -> &'static str {
        "Aborting rebase."
    }

    pub fn failed_initialize_rebase() -> &'static str {
        "Failed intiialize the rebase. Cleaning...\n"
    }

    pub fn failed_to_perform_rebase() -> &'static str {
        "Failed to perform the rebase. Cleaning...\n"
    }

    pub fn failed_to_set_new_stack() -> &'static str {
        "Failed to set the new stack. Cleaning...\n"
    }

    pub fn resolve_conflicts_and_continue() -> String {
        format!(
            "{}\n\
            Please resolve the conflicts, stage the changes, and run\n\
                > {}\n\
            to continue the rebase operation.\n
            Or abort using:\n\
                > {}\n",
            "Cherry-pick conflict detected during the rebase operation!".deco_as_error(),
            "`git-polyp rebase-stack --continue` ".deco_as_command(),
            "`git-polyp rebase-stack --abort`".deco_as_command()
        )
    }
}
