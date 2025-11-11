defmodule GitPolyp.Assertions do
  @moduledoc """
  Custom assertions for GitPolyp tests.
  """

  import ExUnit.Assertions

  @doc """
  Asserts that a branch points to a specific commit SHA.
  """
  def assert_branch_points_to(repo_path, branch_name, expected_sha) do
    {actual_sha, 0} = System.cmd("git", ["rev-parse", branch_name], cd: repo_path, stderr_to_stdout: true)
    actual_sha = String.trim(actual_sha)

    assert actual_sha == expected_sha,
      "Expected branch '#{branch_name}' to point to #{expected_sha}, but it points to #{actual_sha}"
  end

  @doc """
  Asserts that commits are in a specific order (parent -> child).
  """
  def assert_stack_order(repo_path, commit_shas) when is_list(commit_shas) do
    # Verify each commit (except the first) has the previous commit as parent
    commit_shas
    |> Enum.chunk_every(2, 1, :discard)
    |> Enum.each(fn [parent_sha, child_sha] ->
      {parent_of_child, 0} = System.cmd("git", ["rev-parse", "#{child_sha}^"], cd: repo_path, stderr_to_stdout: true)
      parent_of_child = String.trim(parent_of_child)

      assert parent_of_child == parent_sha,
        "Expected commit #{child_sha} to have parent #{parent_sha}, but it has parent #{parent_of_child}"
    end)
  end

  @doc """
  Asserts that a file exists at the given path.
  """
  def assert_file_exists(file_path) do
    assert File.exists?(file_path),
      "Expected file to exist at #{file_path}"
  end

  @doc """
  Asserts that a file does not exist at the given path.
  """
  def refute_file_exists(file_path) do
    refute File.exists?(file_path),
      "Expected file to not exist at #{file_path}, but it does"
  end

  @doc """
  Asserts that a branch exists in the repository.
  """
  def assert_branch_exists(repo_path, branch_name) do
    case System.cmd("git", ["rev-parse", "--verify", branch_name], cd: repo_path, stderr_to_stdout: true) do
      {_, 0} -> :ok
      {_, _} -> flunk("Expected branch '#{branch_name}' to exist, but it does not")
    end
  end

  @doc """
  Asserts that a rebase is in progress.
  """
  def assert_rebase_in_progress(repo_path) do
    git_dir = Path.join(repo_path, ".git")
    rebase_merge_dir = Path.join(git_dir, "rebase-merge")
    rebase_apply_dir = Path.join(git_dir, "rebase-apply")

    rebase_in_progress = File.dir?(rebase_merge_dir) or File.dir?(rebase_apply_dir)

    assert rebase_in_progress,
      "Expected a rebase to be in progress, but none was found"
  end

  @doc """
  Asserts that no rebase is in progress.
  """
  def refute_rebase_in_progress(repo_path) do
    git_dir = Path.join(repo_path, ".git")
    rebase_merge_dir = Path.join(git_dir, "rebase-merge")
    rebase_apply_dir = Path.join(git_dir, "rebase-apply")

    rebase_in_progress = File.dir?(rebase_merge_dir) or File.dir?(rebase_apply_dir)

    refute rebase_in_progress,
      "Expected no rebase to be in progress, but one was found"
  end

  @doc """
  Asserts that the current branch is the expected one.
  """
  def assert_current_branch(repo_path, expected_branch) do
    {actual_branch, 0} = System.cmd("git", ["branch", "--show-current"], cd: repo_path, stderr_to_stdout: true)
    actual_branch = String.trim(actual_branch)

    assert actual_branch == expected_branch,
      "Expected current branch to be '#{expected_branch}', but it is '#{actual_branch}'"
  end

  @doc """
  Strips ANSI escape codes from a string.
  """
  def strip_ansi(string) do
    String.replace(string, ~r/\e\[[0-9;]*m/, "")
  end
end
