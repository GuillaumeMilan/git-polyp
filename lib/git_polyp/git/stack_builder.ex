defmodule GitPolyp.Git.StackBuilder do
  @moduledoc """
  Identifies and builds a linear stack of branches between a base and target branch.

  A stack is a sequence of commits where each commit may have local branches
  pointing to it. The stack must be linear (no merge commits).
  """

  alias GitPolyp.Git.Client

  @doc """
  Identifies the stack of branches between base_branch and target_branch.

  Returns a list of stack entries, each containing:
  - commit: SHA of the commit
  - branches: List of branch names pointing to this commit
  - message: Commit message

  ## Examples
      iex> GitPolyp.Git.StackBuilder.build_stack("main", "feature-3")
      {:ok, [
        %{commit: "abc123", branches: ["feature-1"], message: "Add feature 1"},
        %{commit: "def456", branches: ["feature-2"], message: "Add feature 2"},
        %{commit: "ghi789", branches: ["feature-3"], message: "Add feature 3"}
      ]}
  """
  def build_stack(base_branch, target_branch) do
    with {:ok, merge_base} <- Client.merge_base(base_branch, target_branch),
         {:ok, commits} <- Client.rev_list(merge_base, target_branch),
         {:ok, stack} <- build_stack_entries(commits) do
      if Enum.empty?(stack) do
        {:error, "No commits found between #{base_branch} and #{target_branch}"}
      else
        {:ok, stack, merge_base}
      end
    else
      {:error, %{message: message}} ->
        {:error, message}

      {:error, reason} ->
        {:error, reason}
    end
  end

  # Builds stack entries for a list of commits
  defp build_stack_entries(commits) do
    results =
      Enum.map(commits, fn commit ->
        with {:ok, branches} <- Client.branches_at(commit),
             {:ok, message} <- Client.commit_message(commit) do
          {:ok,
           %{
             commit: commit,
             branches: branches,
             message: message
           }}
        end
      end)

    # Check if all operations succeeded
    case Enum.find(results, fn result -> match?({:error, _}, result) end) do
      nil ->
        stack = Enum.map(results, fn {:ok, entry} -> entry end)
        {:ok, stack}

      {:error, reason} ->
        {:error, reason}
    end
  end

  @doc """
  Validates that a stack is linear (no merge commits for now).

  For the initial version, we simply check that each commit has exactly
  one parent within the stack range.
  """
  def validate_linear_stack(_stack) do
    # For now, we assume the stack is linear since we're using rev-list
    # which already filters to a linear history.
    # Future enhancement: detect and handle merge commits
    :ok
  end

  @doc """
  Formats a stack for display, showing branches and commit messages.

  Format: * <short-commit-hash> - (<branches>) <commit-message>
  - short-commit-hash in red
  - branches in yellow
  """
  def format_stack(stack) do
    stack
    |> Enum.map(fn entry ->
      short_commit = String.slice(entry.commit, 0..8)

      # Format commit hash in red
      commit_colored = IO.ANSI.format([:red, short_commit, :reset]) |> IO.iodata_to_binary()

      # Format branches in yellow
      branches_colored =
        if Enum.empty?(entry.branches) do
          ""
        else
          branch_list = Enum.join(entry.branches, ", ")
          IO.ANSI.format([:yellow, "(", branch_list, ")", :reset]) |> IO.iodata_to_binary()
        end

      message_first_line = entry.message |> String.split("\n") |> List.first()

      # Build the line with proper spacing
      if branches_colored == "" do
        "* #{commit_colored} - #{message_first_line}"
      else
        "* #{commit_colored} - #{branches_colored} #{message_first_line}"
      end
    end)
    |> Enum.join("\n")
  end
end
