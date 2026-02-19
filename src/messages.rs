use crate::io::Decorate;

pub mod error {
    use super::Decorate;

    pub const NOT_IN_GIT_REPO: &str =
        "Not a git repository. Please run this command inside a git repository.";
    pub const FAILED_TO_ACCESS_GIT_POLYP_DIR: &str = "Failed to access to git-polyp private directory. Make sure you have the right to access the .git directory.";

    pub fn failed_to_verify_upstream() -> &'static str {
        "Failed to verify the upstream. Make sure the provided upstream is correct and exists."
    }

    pub fn failed_to_find_merge_base() -> &'static str {
        "Failed to find the merge base between the upstream and the branch."
    }

    pub fn failed_to_verify_base() -> String {
        format!(
            "{}
            Please make sure the provided {} option is correct and exists.",
            "Failed to verify the --base option content.".deco_as_error(),
            "--base".deco_as_command()
        )
    }

    pub fn base_not_descendant_of_upstream() -> String {
        format!(
            "{}
            Please provide a {} base option that is a descendant of the upstream.",
            "The provided --base option is not a descendant of the upstream.".deco_as_error(),
            "--base".deco_as_command()
        )
    }

    pub fn failed_to_build_stack() -> String {
        format!(
            "{}
            Please make sure the provided upstream and branch are correct and exist, and that the --base option, if provided, is correct and exists.
            If the error persists, please check the state of your repository and try to fix it before running this command again.",
            "Failed to build the stack of commits to rebase.".deco_as_error()
        )
    }

    pub fn failed_to_clean_stack() -> String {
        format!(
            "{}
            Please run `{}` to clean the stack.
            If the error persists, please try to remove the {} file manually.",
            "Failed to clean the stack state!".deco_as_error(),
            "git-polyp rebase-stack --abort".deco_as_command(),
            ".git/polyp/stack.json".deco_as_path()
        )
    }

    pub fn failed_to_clean_stack_after_rebase() -> String {
        format!(
            "{}
            Please run `{}` to clean the stack.
            If the error persists, please try to remove the {} file manually.",
            "Failed to clean the stack!".deco_as_error(),
            "git-polyp rebase-stack --abort".deco_as_command(),
            ".git/polyp/stack.json".deco_as_path()
        )
    }

    pub fn failed_to_reset_stack_as_before() -> String {
        format!(
            "{}
            Please run `{}` to reset the stack to its previous state.
            If the error persists, please try to restore the stack file with the backup file created during the rebase process, or remove the stack file manually if you don't have a backup.",
            "Failed to reset the stack as it was before!".deco_as_error(),
            "git-polyp rebase-stack --reset".deco_as_command(),
        )
    }

    pub fn failed_to_find_branch() -> String {
        "Failed to find the current branch name. Make sure you are in a git repository."
            .deco_as_error()
            .to_string()
    }

    pub fn failed_to_push_branches(push_command: &str) -> String {
        format!(
            "{}
            Please push the branches manually with the following command:
            {}",
            "Failed to push the new branches to the remote repository.".deco_as_error(),
            push_command
        )
    }
}

pub mod info {
    use super::Decorate;

    pub fn rebase_in_progress() -> &'static str {
        "A rebase is already in progress.
        Continue it with `git-polyp rebase-stack --continue`.
        Abort it without doing any modification to the repository with `git-polyp rebase-stack --abort`.
        Abort by reseting the stack of commit to its version before any operation with `git-polyp rebase-stack --reset`."
    }

    pub fn ask_rebase_confirmation() -> String {
        "Do you want to rebase this stack?".to_string()
    }

    pub fn ask_push_confirmation(push_command: String) -> String {
        format!(
            "Rebase successful. Do you want to push the new branches to '{}' ?
            You can also push them later with the following command:
            {}",
            "origin".deco_as_command(),
            push_command
        )
    }

    pub fn aborting_rebase() -> &'static str {
        "Aborting rebase."
    }

    pub fn failed_initialize_rebase() -> &'static str {
        "Failed intiialize the rebase. Cleaning..."
    }

    pub fn stack_persisted() -> &'static str {
        "Stack persisted. Starting rebase..."
    }

    pub fn failed_to_perform_rebase() -> &'static str {
        "Failed to perform the rebase. Cleaning..."
    }

    pub fn failed_to_set_new_stack() -> &'static str {
        "Failed to set the new stack. Cleaning..."
    }
}
