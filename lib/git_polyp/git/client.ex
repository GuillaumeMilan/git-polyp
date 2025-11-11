defmodule GitPolyp.Git.Client do
  @moduledoc """
  Low-level Git command execution wrapper.

  All Git operations go through this module to ensure consistent
  error handling and result formatting.
  """

  @doc """
  Finds the merge base (common ancestor) between two branches.

  ## Examples
      iex> GitPolyp.Git.Client.merge_base("main", "feature")
      {:ok, "abc123..."}
  """
  def merge_base(base_branch, target_branch) do
    case git_cmd(["merge-base", base_branch, target_branch]) do
      {:ok, output} -> {:ok, String.trim(output)}
      error -> error
    end
  end

  @doc """
  Lists commits in reverse order from base to target (exclusive..inclusive).

  ## Examples
      iex> GitPolyp.Git.Client.rev_list("abc123", "feature")
      {:ok, ["def456", "ghi789"]}
  """
  def rev_list(base_commit, target_branch) do
    case git_cmd(["rev-list", "--reverse", "#{base_commit}..#{target_branch}"]) do
      {:ok, output} ->
        commits =
          output
          |> String.trim()
          |> String.split("\n", trim: true)

        {:ok, commits}

      error ->
        error
    end
  end

  @doc """
  Finds all local branches pointing at a specific commit.

  ## Examples
      iex> GitPolyp.Git.Client.branches_at("abc123")
      {:ok, ["feature-1", "feature-2"]}
  """
  def branches_at(commit) do
    case git_cmd(["branch", "--points-at", commit, "--format=%(refname:short)"]) do
      {:ok, output} ->
        branches =
          output
          |> String.trim()
          |> String.split("\n", trim: true)

        {:ok, branches}

      error ->
        error
    end
  end

  @doc """
  Gets the commit message for a given commit SHA.

  ## Examples
      iex> GitPolyp.Git.Client.commit_message("abc123")
      {:ok, "Add new feature"}
  """
  def commit_message(commit) do
    case git_cmd(["log", "-1", "--format=%B", commit]) do
      {:ok, output} -> {:ok, String.trim(output)}
      error -> error
    end
  end

  @doc """
  Executes a rebase operation onto a new base.

  ## Examples
      iex> GitPolyp.Git.Client.rebase_onto("main", "abc123", "feature")
      {:ok, "Successfully rebased"}
  """
  def rebase_onto(base_branch, merge_base, target_branch) do
    git_cmd(["rebase", "--onto", base_branch, merge_base, target_branch])
  end

  @doc """
  Aborts the current rebase operation.
  """
  def rebase_abort do
    git_cmd(["rebase", "--abort"])
  end

  @doc """
  Continues the current rebase operation.
  """
  def rebase_continue do
    git_cmd(["rebase", "--continue"])
  end

  @doc """
  Checks if a rebase is currently in progress.

  Returns true if `.git/rebase-merge` or `.git/rebase-apply` exists.
  """
  def rebase_in_progress? do
    git_dir = git_dir()

    File.dir?(Path.join(git_dir, "rebase-merge")) or
      File.dir?(Path.join(git_dir, "rebase-apply"))
  end

  @doc """
  Gets the current branch name.

  ## Examples
      iex> GitPolyp.Git.Client.current_branch()
      {:ok, "main"}
  """
  def current_branch do
    case git_cmd(["branch", "--show-current"]) do
      {:ok, output} -> {:ok, String.trim(output)}
      error -> error
    end
  end

  @doc """
  Checks out a branch.

  ## Examples
      iex> GitPolyp.Git.Client.checkout("feature")
      {:ok, "Switched to branch 'feature'"}
  """
  def checkout(branch) do
    git_cmd(["checkout", branch])
  end

  @doc """
  Updates a branch reference to point to a new commit.

  ## Examples
      iex> GitPolyp.Git.Client.update_ref("feature", "abc123")
      {:ok, ""}
  """
  def update_ref(branch, commit) do
    git_cmd(["branch", "--force", branch, commit])
  end

  @doc """
  Checks if a branch exists.

  ## Examples
      iex> GitPolyp.Git.Client.branch_exists?("main")
      true
  """
  def branch_exists?(branch) do
    case git_cmd(["rev-parse", "--verify", branch]) do
      {:ok, _} -> true
      {:error, _} -> false
    end
  end

  @doc """
  Checks if we're inside a git repository.
  """
  def in_git_repo? do
    case git_cmd(["rev-parse", "--git-dir"]) do
      {:ok, _} -> true
      {:error, _} -> false
    end
  end

  @doc """
  Gets the path to the .git directory.
  """
  def git_dir do
    case git_cmd(["rev-parse", "--git-dir"]) do
      {:ok, path} -> String.trim(path)
      {:error, _} -> ".git"
    end
  end

  @doc """
  Lists all commits from a branch or HEAD.
  Used after rebase to identify new commit SHAs.

  If branch_or_ref is nil, uses HEAD.
  """
  def new_commits(branch_or_ref, count) do
    ref = branch_or_ref || "HEAD"

    case git_cmd(["log", "--format=%H %s", "-#{count}", ref]) do
      {:ok, output} ->
        commits =
          output
          |> String.trim()
          |> String.split("\n", trim: true)
          |> Enum.map(fn line ->
            [sha | message_parts] = String.split(line, " ", parts: 2)
            message = Enum.join(message_parts, " ")
            {sha, message}
          end)

        {:ok, commits}

      error ->
        error
    end
  end

  @doc """
  Pushes a branch to remote with --force-with-lease for safety.

  ## Examples
      iex> GitPolyp.Git.Client.push_force_with_lease("feature")
      {:ok, ""}
  """
  def push_force_with_lease(branch, remote \\ "origin") do
    git_cmd(["push", "--force-with-lease", remote, branch])
  end

  # Private helper to execute git commands
  defp git_cmd(args) do
    case System.cmd("git", args, stderr_to_stdout: true) do
      {output, 0} ->
        {:ok, output}

      {output, exit_code} ->
        {:error, %{exit_code: exit_code, message: String.trim(output)}}
    end
  end
end
