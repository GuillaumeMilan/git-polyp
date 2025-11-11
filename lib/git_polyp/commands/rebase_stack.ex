defmodule GitPolyp.Commands.RebaseStack do
  @moduledoc """
  Implements the rebase-stack command.

  Rebases a linear stack of local branches onto a new base branch.
  """

  alias GitPolyp.Git.{Client, StackBuilder, BranchUpdater}
  alias GitPolyp.State.{Metadata, Manager}
  alias GitPolyp.UI.{Formatter, Prompt}

  @doc """
  Runs the rebase-stack command.

  Accepts arguments:
  - [base_branch, target_branch] - Start a new rebase
  - ["--continue"] - Continue after conflict resolution
  - ["--abort"] - Abort the rebase
  """
  def run(args) do
    # Parse options with OptionParser
    {opts, positional, invalid} =
      OptionParser.parse(args,
        strict: [
          continue: :boolean,
          abort: :boolean,
          help: :boolean
        ],
        aliases: [
          h: :help
        ]
      )

    cond do
      opts[:help] ->
        print_help()
        :ok

      not Enum.empty?(invalid) ->
        invalid_opts = Enum.map(invalid, fn {opt, _} -> opt end) |> Enum.join(", ")
        {:error, "Unknown option: #{invalid_opts}\nRun 'git-polyp rebase-stack --help' for usage information."}

      opts[:continue] ->
        continue_rebase()

      opts[:abort] ->
        abort_rebase()

      length(positional) == 2 ->
        [base_branch, target_branch] = positional
        start_rebase(base_branch, target_branch)

      true ->
        print_help()
        {:error, "Invalid arguments. Expected: base-branch target-branch"}
    end
  end

  defp start_rebase(base_branch, target_branch) do
    with :ok <- validate_environment(),
         :ok <- validate_no_rebase_in_progress(),
         {:ok, original_branch} <- Client.current_branch(),
         :ok <- validate_branches(base_branch, target_branch),
         {:ok, stack, merge_base} <- StackBuilder.build_stack(base_branch, target_branch),
         :ok <- display_and_confirm_stack(stack, base_branch, target_branch),
         :ok <- save_metadata(base_branch, merge_base, target_branch, stack, original_branch),
         :ok <- checkout_target(target_branch),
         :ok <- execute_rebase(base_branch, merge_base, target_branch, stack) do
      :ok
    else
      {:error, reason} -> {:error, reason}
    end
  end

  defp validate_environment do
    if Client.in_git_repo?() do
      :ok
    else
      {:error, "Not in a git repository"}
    end
  end

  defp validate_no_rebase_in_progress do
    cond do
      Manager.exists?() ->
        {:error,
         "A rebase-stack operation is already in progress.\n" <>
           "Use --continue to resume or --abort to cancel."}

      Client.rebase_in_progress?() ->
        {:error,
         "A git rebase is in progress.\n" <>
           "Please finish it with 'git rebase --continue' or 'git rebase --abort' first."}

      true ->
        :ok
    end
  end

  defp validate_branches(base_branch, target_branch) do
    cond do
      not Client.branch_exists?(base_branch) ->
        {:error, "Base branch '#{base_branch}' does not exist"}

      not Client.branch_exists?(target_branch) ->
        {:error, "Target branch '#{target_branch}' does not exist"}

      true ->
        :ok
    end
  end

  defp display_and_confirm_stack(stack, base_branch, target_branch) do
    IO.puts(Formatter.header("Stack to rebase:"))
    IO.puts("")
    IO.puts("  Base:   #{Formatter.branch(base_branch)}")
    IO.puts("  Target: #{Formatter.branch(target_branch)}")
    IO.puts("")
    IO.puts(StackBuilder.format_stack(stack))
    IO.puts("")

    if Prompt.confirm("Proceed with rebase?") do
      :ok
    else
      {:error, "Rebase cancelled by user"}
    end
  end

  defp save_metadata(base_branch, merge_base, target_branch, stack, original_branch) do
    metadata = Metadata.new(base_branch, merge_base, target_branch, stack, original_branch)

    case Manager.save(metadata) do
      :ok -> :ok
      {:error, reason} -> {:error, "Failed to save metadata: #{reason}"}
    end
  end

  defp checkout_target(target_branch) do
    case Client.checkout(target_branch) do
      {:ok, _} ->
        :ok

      {:error, %{message: message}} ->
        {:error, "Failed to checkout #{target_branch}: #{message}"}
    end
  end

  defp execute_rebase(base_branch, merge_base, target_branch, stack) do
    IO.puts(Formatter.info("Starting rebase..."))

    case Client.rebase_onto(base_branch, merge_base, target_branch) do
      {:ok, _output} ->
        # Rebase succeeded, update branches
        finalize_rebase(stack, base_branch)

      {:error, %{exit_code: exit_code}} when exit_code != 0 ->
        # Rebase conflict
        handle_rebase_conflict()
    end
  end

  defp handle_rebase_conflict do
    IO.puts("")
    IO.puts(Formatter.warning("Rebase conflict detected!"))
    IO.puts("")
    IO.puts("Please resolve the conflicts and then run:")
    IO.puts(Formatter.command("git-polyp rebase-stack --continue"))
    IO.puts("")
    IO.puts("Or abort the rebase:")
    IO.puts(Formatter.command("git-polyp rebase-stack --abort"))
    IO.puts("")

    # Metadata is already saved, so we can just exit
    :ok
  end

  defp finalize_rebase(stack, base_branch) do
    IO.puts(Formatter.success("Rebase completed successfully!"))
    IO.puts("")

    # Get the rebased commits BEFORE switching branches
    # At this point, HEAD still points to the rebased target branch
    stack_length = length(stack)

    case Client.new_commits(nil, stack_length) do
      {:ok, new_commits} ->
        # Now checkout base branch to avoid "branch is checked out" errors
        IO.puts(Formatter.info("Switching to #{base_branch} to update branch pointers..."))

        case Client.checkout(base_branch) do
          {:ok, _} ->
            update_stack_branches(stack, new_commits)

          {:error, %{message: message}} ->
            IO.puts(Formatter.warning("Could not checkout #{base_branch}: #{message}"))
            IO.puts("Attempting to update branches anyway...")
            update_stack_branches(stack, new_commits)
        end

      {:error, reason} ->
        {:error, "Failed to get new commits: #{inspect(reason)}"}
    end
  end

  defp update_stack_branches(stack, new_commits) do
    case BranchUpdater.update_branches_with_commits(stack, new_commits) do
      {:ok, updates} ->
        IO.puts(Formatter.header("Updated branches:"))
        IO.puts(BranchUpdater.format_updates(updates))
        IO.puts("")
        print_push_instructions(updates)
        Manager.delete()
        :ok

      {:warning, {:unmatched_commits, unmatched, updates}} ->
        IO.puts(Formatter.header("Updated branches:"))
        IO.puts(BranchUpdater.format_updates(updates))
        IO.puts("")
        IO.puts(Formatter.warning("Some commits could not be matched:"))

        Enum.each(unmatched, fn {_, branch, commit, _} ->
          IO.puts("  #{branch} (#{String.slice(commit, 0..7)})")
        end)

        IO.puts("")
        print_push_instructions(updates)
        Manager.delete()
        :ok

      {:error, {:update_failed, errors}} ->
        IO.puts(Formatter.error("Failed to update some branches:"))
        IO.puts("")

        Enum.each(errors, fn {:error, %{branch: branch, reason: reason}} ->
          error_msg = format_update_error(branch, reason)
          IO.puts("  #{error_msg}")
        end)

        IO.puts("")
        Manager.delete()
        {:error, "Branch update failed"}
    end
  end

  defp format_update_error(branch, %{message: message}) when is_binary(message) do
    cond do
      String.contains?(message, "worktree") ->
        "#{branch}: Cannot update - branch is checked out in a worktree"

      String.contains?(message, "checked out") ->
        "#{branch}: Cannot update - branch is currently checked out"

      true ->
        "#{branch}: #{message}"
    end
  end

  defp format_update_error(branch, reason) do
    "#{branch}: #{inspect(reason)}"
  end

  defp print_push_instructions(updates) do
    IO.puts(Formatter.header("Next steps:"))
    IO.puts("To push the rebased branches to remote, run:")
    IO.puts("")

    Enum.each(updates, fn update ->
      IO.puts(Formatter.command("git push --force-with-lease origin #{update.branch}"))
    end)

    IO.puts("")

    # Only prompt for auto-push in production (not during tests)
    if Mix.env() != :test do
      if Prompt.confirm("Would you like to push all these branches now?") do
        push_branches(updates)
      else
        IO.puts("You can push the branches manually later using the commands above.")
      end
    end
  end

  defp push_branches(updates) do
    IO.puts("")
    IO.puts(Formatter.info("Pushing branches..."))
    IO.puts("")

    results =
      Enum.map(updates, fn update ->
        IO.puts("Pushing #{Formatter.branch(update.branch)}...")

        case Client.push_force_with_lease(update.branch) do
          {:ok, _} ->
            IO.puts(Formatter.success("  ✓ #{update.branch} pushed successfully"))
            {:ok, update.branch}

          {:error, %{message: message}} ->
            IO.puts(Formatter.error("  ✗ Failed to push #{update.branch}: #{message}"))
            {:error, update.branch, message}
        end
      end)

    IO.puts("")

    failures = Enum.filter(results, fn result -> match?({:error, _, _}, result) end)

    if Enum.empty?(failures) do
      IO.puts(Formatter.success("All branches pushed successfully!"))
    else
      IO.puts(Formatter.warning("Some branches failed to push:"))

      Enum.each(failures, fn {:error, branch, _message} ->
        IO.puts("  - #{branch}")
      end)

      IO.puts("")
      IO.puts("You can retry pushing these branches manually.")
    end
  end

  # Continue command implementation
  defp continue_rebase do
    with {:ok, metadata} <- load_metadata(),
         :ok <- validate_rebase_complete(),
         :ok <- finalize_continue(metadata) do
      :ok
    else
      {:error, reason} -> {:error, reason}
    end
  end

  defp load_metadata do
    case Manager.load() do
      {:ok, metadata} ->
        {:ok, metadata}

      {:error, :not_found} ->
        {:error,
         "No rebase-stack operation in progress.\n" <>
           "Start a new one with: git-polyp rebase-stack <base> <target>"}

      {:error, reason} ->
        {:error, "Failed to load rebase metadata: #{reason}"}
    end
  end

  defp validate_rebase_complete do
    if Client.rebase_in_progress?() do
      {:error,
       "Git rebase still has conflicts.\n" <>
         "Please resolve conflicts and stage changes with 'git add',\n" <>
         "then run 'git rebase --continue' before running this command again."}
    else
      :ok
    end
  end

  defp finalize_continue(metadata) do
    IO.puts(Formatter.info("Continuing rebase-stack operation..."))
    IO.puts(Formatter.success("Rebase completed successfully!"))
    IO.puts("")

    # Get commits from the target branch before switching
    stack_length = length(metadata.stack)

    case Client.new_commits(metadata.target_branch, stack_length) do
      {:ok, new_commits} ->
        # Checkout base branch to avoid "branch is checked out" errors
        IO.puts(Formatter.info("Switching to #{metadata.base_branch} to update branch pointers..."))

        case Client.checkout(metadata.base_branch) do
          {:ok, _} ->
            update_stack_branches_continue(metadata, new_commits)

          {:error, %{message: message}} ->
            IO.puts(Formatter.warning("Could not checkout #{metadata.base_branch}: #{message}"))
            IO.puts("Attempting to update branches anyway...")
            update_stack_branches_continue(metadata, new_commits)
        end

      {:error, reason} ->
        {:error, "Failed to get new commits from #{metadata.target_branch}: #{inspect(reason)}"}
    end
  end

  defp update_stack_branches_continue(metadata, new_commits) do
    case BranchUpdater.update_branches_with_commits(metadata.stack, new_commits) do
      {:ok, updates} ->
        IO.puts(Formatter.header("Updated branches:"))
        IO.puts(BranchUpdater.format_updates(updates))
        IO.puts("")
        print_push_instructions(updates)
        Manager.delete()
        :ok

      {:warning, {:unmatched_commits, unmatched, updates}} ->
        IO.puts(Formatter.header("Updated branches:"))
        IO.puts(BranchUpdater.format_updates(updates))
        IO.puts("")
        IO.puts(Formatter.warning("Some commits could not be matched:"))

        Enum.each(unmatched, fn {_, branch, commit, _} ->
          IO.puts("  #{branch} (#{String.slice(commit, 0..7)})")
        end)

        IO.puts("")
        print_push_instructions(updates)
        Manager.delete()
        :ok

      {:error, {:update_failed, errors}} ->
        IO.puts(Formatter.error("Failed to update some branches:"))
        IO.puts("")

        Enum.each(errors, fn {:error, %{branch: branch, reason: reason}} ->
          error_msg = format_update_error(branch, reason)
          IO.puts("  #{error_msg}")
        end)

        IO.puts("")
        Manager.delete()
        {:error, "Branch update failed"}
    end
  end

  # Abort command implementation
  defp abort_rebase do
    with {:ok, _metadata} <- load_metadata(),
         :ok <- abort_git_rebase(),
         :ok <- cleanup_metadata() do
      IO.puts(Formatter.success("Rebase-stack operation aborted"))
      IO.puts("Repository has been restored to its previous state.")
      :ok
    else
      {:error, reason} -> {:error, reason}
    end
  end

  defp abort_git_rebase do
    if Client.rebase_in_progress?() do
      IO.puts(Formatter.info("Aborting git rebase..."))

      case Client.rebase_abort() do
        {:ok, _} ->
          :ok

        {:error, %{message: message}} ->
          # Even if git rebase --abort fails, we should still clean up metadata
          IO.puts(Formatter.warning("Failed to abort git rebase: #{message}"))
          IO.puts("You may need to manually run: git rebase --abort")
          :ok
      end
    else
      # No active git rebase, just clean up metadata
      :ok
    end
  end

  defp cleanup_metadata do
    case Manager.delete() do
      :ok ->
        :ok

      {:error, reason} ->
        IO.puts(Formatter.warning("Failed to delete metadata: #{inspect(reason)}"))
        :ok
    end
  end

  defp print_help do
    IO.puts("""
    #{Formatter.header("git-polyp rebase-stack")} - Rebase a linear stack of branches

    #{Formatter.header("USAGE:")}
      git-polyp rebase-stack <base-branch> <target-branch>
      git-polyp rebase-stack --continue
      git-polyp rebase-stack --abort

    #{Formatter.header("ARGUMENTS:")}
      <base-branch>    The branch to rebase onto (e.g., main)
      <target-branch>  The top branch of the stack to rebase

    #{Formatter.header("OPTIONS:")}
      --continue       Continue rebase after resolving conflicts
      --abort          Abort the rebase operation
      -h, --help       Show this help message

    #{Formatter.header("DESCRIPTION:")}
      Rebases a linear stack of local branches onto a new base branch.

      The command:
      1. Identifies all commits and branches between base and target
      2. Shows the stack and asks for confirmation
      3. Rebases the entire stack onto the new base
      4. Updates all branch pointers to the new commits

    #{Formatter.header("EXAMPLES:")}
      # Rebase feature-1, feature-2, feature-3 onto main
      git-polyp rebase-stack main feature-3

      # Continue after resolving conflicts
      git-polyp rebase-stack --continue

      # Abort the rebase
      git-polyp rebase-stack --abort
    """)
  end
end
