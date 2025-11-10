defmodule GitPolyp.GitTestHelper do
  @moduledoc """
  Test helpers for creating and managing temporary Git repositories for testing.
  """

  @doc """
  Creates a temporary Git repository for testing.

  Returns {:ok, repo_path} or {:error, reason}
  """
  def create_test_repo(name \\ "test_repo") do
    # Create unique temp directory
    timestamp = :os.system_time(:millisecond)
    repo_path = Path.join([System.tmp_dir!(), "git_polyp_test_#{name}_#{timestamp}"])

    File.mkdir_p!(repo_path)

    # Initialize git repo
    {_, 0} = System.cmd("git", ["init", "-b", "main"], cd: repo_path, stderr_to_stdout: true)
    {_, 0} = System.cmd("git", ["config", "user.name", "Test User"], cd: repo_path)
    {_, 0} = System.cmd("git", ["config", "user.email", "test@example.com"], cd: repo_path)

    {:ok, repo_path}
  end

  @doc """
  Creates a commit in the given repository with the specified message and content.

  Returns {:ok, commit_sha} or {:error, reason}
  """
  def create_commit(repo_path, message, file_content \\ nil) do
    # Generate unique filename based on message
    filename = "file_#{:erlang.phash2(message)}.txt"
    file_path = Path.join(repo_path, filename)

    # Write content to file
    content = file_content || "Content for: #{message}\n"
    File.write!(file_path, content)

    # Stage and commit
    {_, 0} = System.cmd("git", ["add", filename], cd: repo_path, stderr_to_stdout: true)
    {_, 0} = System.cmd("git", ["commit", "-m", message], cd: repo_path, stderr_to_stdout: true)

    # Get the commit SHA
    {sha, 0} = System.cmd("git", ["rev-parse", "HEAD"], cd: repo_path, stderr_to_stdout: true)
    {:ok, String.trim(sha)}
  end

  @doc """
  Creates a branch at the current HEAD or at a specific commit.

  Returns :ok or {:error, reason}
  """
  def create_branch(repo_path, branch_name, commit_sha \\ nil) do
    args = if commit_sha do
      ["branch", branch_name, commit_sha]
    else
      ["branch", branch_name]
    end

    case System.cmd("git", args, cd: repo_path, stderr_to_stdout: true) do
      {_, 0} -> :ok
      {error, _} -> {:error, error}
    end
  end

  @doc """
  Checks out a branch in the repository.
  """
  def checkout_branch(repo_path, branch_name) do
    case System.cmd("git", ["checkout", branch_name], cd: repo_path, stderr_to_stdout: true) do
      {_, 0} -> :ok
      {error, _} -> {:error, error}
    end
  end

  @doc """
  Sets up a linear stack of branches for testing.

  ## Example
      setup_linear_stack(repo_path, %{
        base: "main",
        branches: ["feature-1", "feature-2", "feature-3"],
        commits_per_branch: 1
      })

  Returns {:ok, %{branches: [...], commits: %{branch => [shas]}}}
  """
  def setup_linear_stack(repo_path, opts \\ %{}) do
    base = opts[:base] || "main"
    branches = opts[:branches] || ["feature-1", "feature-2", "feature-3"]
    commits_per_branch = opts[:commits_per_branch] || 1

    # Start on base branch (already on main/master after init)
    commits_map = %{}

    # Create initial commit on base
    {:ok, _} = create_commit(repo_path, "Initial commit on #{base}")

    # Create each branch with commits
    commits_map = Enum.reduce(branches, commits_map, fn branch, acc ->
      # Create commits for this branch
      commit_shas = Enum.map(1..commits_per_branch, fn n ->
        message = "Commit #{n} on #{branch}"
        {:ok, sha} = create_commit(repo_path, message)
        sha
      end)

      # Create branch at current HEAD
      :ok = create_branch(repo_path, branch)

      Map.put(acc, branch, commit_shas)
    end)

    # Return to base branch
    checkout_branch(repo_path, base)

    {:ok, %{branches: branches, commits: commits_map, base: base}}
  end

  @doc """
  Sets up a stack with a specific structure for testing.

  Creates:
  - Base branch (main) with initial commit
  - Feature-1 with 1 commit on top of main
  - Feature-2 with 1 commit on top of feature-1
  - Feature-3 with 1 commit on top of feature-2

  All branches point to their respective commits, and main stays at the initial commit.
  """
  def setup_standard_stack(repo_path) do
    # Create initial commit on main
    {:ok, base_sha} = create_commit(repo_path, "Initial commit")

    # Create feature-1
    {:ok, commit1_sha} = create_commit(repo_path, "Feature 1 commit")
    :ok = create_branch(repo_path, "feature-1")

    # Create feature-2
    {:ok, commit2_sha} = create_commit(repo_path, "Feature 2 commit")
    :ok = create_branch(repo_path, "feature-2")

    # Create feature-3
    {:ok, commit3_sha} = create_commit(repo_path, "Feature 3 commit")
    :ok = create_branch(repo_path, "feature-3")

    # Return main to the initial commit
    checkout_branch(repo_path, "main")
    {_, 0} = System.cmd("git", ["reset", "--hard", base_sha], cd: repo_path)

    {:ok, %{
      branches: ["feature-1", "feature-2", "feature-3"],
      commits: %{
        "feature-1" => [commit1_sha],
        "feature-2" => [commit2_sha],
        "feature-3" => [commit3_sha]
      }
    }}
  end

  @doc """
  Gets the commit SHA for a commit with a specific message.
  """
  def get_commit_sha(repo_path, message) do
    {output, 0} = System.cmd("git", ["log", "--all", "--format=%H %s"], cd: repo_path, stderr_to_stdout: true)

    output
    |> String.split("\n", trim: true)
    |> Enum.find_value(fn line ->
      case String.split(line, " ", parts: 2) do
        [sha, ^message] -> sha
        _ -> nil
      end
    end)
  end

  @doc """
  Gets the current commit SHA of a branch.
  """
  def get_branch_sha(repo_path, branch_name) do
    case System.cmd("git", ["rev-parse", branch_name], cd: repo_path, stderr_to_stdout: true) do
      {sha, 0} -> {:ok, String.trim(sha)}
      {error, _} -> {:error, error}
    end
  end

  @doc """
  Cleans up a test repository.
  """
  def cleanup_test_repo(repo_path) do
    File.rm_rf!(repo_path)
    :ok
  end

  @doc """
  Creates a merge conflict scenario for testing.

  Returns {:ok, %{base: branch, feature: branch, conflict_file: path}}
  """
  def setup_conflict_scenario(repo_path) do
    # Create initial commit
    conflict_file = "conflict.txt"
    file_path = Path.join(repo_path, conflict_file)
    File.write!(file_path, "Original content\n")
    {_, 0} = System.cmd("git", ["add", conflict_file], cd: repo_path)
    {_, 0} = System.cmd("git", ["commit", "-m", "Initial commit"], cd: repo_path)

    # Create base branch and modify file
    :ok = create_branch(repo_path, "base")
    :ok = checkout_branch(repo_path, "base")
    File.write!(file_path, "Base branch content\n")
    {_, 0} = System.cmd("git", ["add", conflict_file], cd: repo_path)
    {_, 0} = System.cmd("git", ["commit", "-m", "Base change"], cd: repo_path)

    # Create feature branch from main and modify same file
    :ok = checkout_branch(repo_path, "main")
    :ok = create_branch(repo_path, "feature")
    :ok = checkout_branch(repo_path, "feature")
    File.write!(file_path, "Feature branch content\n")
    {_, 0} = System.cmd("git", ["add", conflict_file], cd: repo_path)
    {_, 0} = System.cmd("git", ["commit", "-m", "Feature change"], cd: repo_path)

    :ok = checkout_branch(repo_path, "main")

    {:ok, %{base: "base", feature: "feature", conflict_file: conflict_file}}
  end

  @doc """
  Runs a git command in the test repo and returns the output.
  Useful for debugging or custom test scenarios.
  """
  def git_cmd(repo_path, args) do
    System.cmd("git", args, cd: repo_path, stderr_to_stdout: true)
  end
end
