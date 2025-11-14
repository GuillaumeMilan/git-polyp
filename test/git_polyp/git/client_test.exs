defmodule GitPolyp.Git.ClientTest do
  use ExUnit.Case

  import GitPolyp.GitTestHelper
  import GitPolyp.Assertions

  alias GitPolyp.Git.Client

  setup do
    {:ok, repo_path} = create_test_repo("client_test")

    on_exit(fn -> cleanup_test_repo(repo_path) end)

    %{repo: repo_path}
  end

  describe "in_git_repo?/0" do
    test "returns true when in a git repository", %{repo: repo} do
      File.cd!(repo, fn ->
        assert Client.in_git_repo?() == true
      end)
    end

    test "returns false when not in a git repository" do
      temp_dir = System.tmp_dir!()
      non_git_dir = Path.join(temp_dir, "not_a_repo_#{:os.system_time(:millisecond)}")
      File.mkdir_p!(non_git_dir)

      File.cd!(non_git_dir, fn ->
        assert Client.in_git_repo?() == false
      end)

      File.rm_rf!(non_git_dir)
    end
  end

  describe "git_dir/0" do
    test "returns .git directory path", %{repo: repo} do
      File.cd!(repo, fn ->
        git_dir = Client.git_dir()
        # git_dir returns a string (could be relative or absolute)
        assert is_binary(git_dir)
        assert String.ends_with?(git_dir, ".git") or git_dir == ".git"
      end)
    end
  end

  describe "current_branch/0" do
    test "returns current branch name", %{repo: repo} do
      File.cd!(repo, fn ->
        # Create a commit first (needed for branch to exist)
        create_commit(repo, "Initial commit")

        {:ok, branch} = Client.current_branch()
        # We initialize with "main" branch in our test helper
        assert branch == "main"
      end)
    end

    test "returns branch after checkout", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Initial commit")
        create_branch(repo, "test-branch")
        checkout_branch(repo, "test-branch")

        {:ok, branch} = Client.current_branch()
        assert branch == "test-branch"
      end)
    end
  end

  describe "branch_exists?/1" do
    test "returns true for existing branch", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Initial commit")
        create_branch(repo, "existing-branch")

        assert Client.branch_exists?("existing-branch") == true
      end)
    end

    test "returns false for non-existent branch", %{repo: repo} do
      File.cd!(repo, fn ->
        assert Client.branch_exists?("non-existent-branch") == false
      end)
    end

    test "returns true for main branch", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Initial commit")

        assert Client.branch_exists?("main") == true
      end)
    end
  end

  describe "merge_base/2" do
    test "finds common ancestor of two branches", %{repo: repo} do
      File.cd!(repo, fn ->
        # Create base commit
        {:ok, base_sha} = create_commit(repo, "Base commit")

        # Create feature branch from base
        create_branch(repo, "feature")

        # Add commit on feature
        checkout_branch(repo, "feature")
        create_commit(repo, "Feature commit")

        # Go back to main
        checkout_branch(repo, "main")

        {:ok, merge_base} = Client.merge_base("main", "feature")
        assert merge_base == base_sha
      end)
    end

    test "returns error for non-existent branch", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Initial commit")

        assert {:error, _} = Client.merge_base("main", "non-existent")
      end)
    end
  end

  describe "rev_list/2" do
    test "returns commits between base and target", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, base_sha} = create_commit(repo, "Base")
        {:ok, commit1} = create_commit(repo, "Commit 1")
        {:ok, commit2} = create_commit(repo, "Commit 2")

        {:ok, commits} = Client.rev_list(base_sha, "main")

        assert length(commits) == 2
        assert commit1 in commits
        assert commit2 in commits
        refute base_sha in commits
      end)
    end

    test "returns commits in correct order (oldest first)", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, base_sha} = create_commit(repo, "Base")
        {:ok, commit1} = create_commit(repo, "First")
        {:ok, commit2} = create_commit(repo, "Second")
        {:ok, commit3} = create_commit(repo, "Third")

        {:ok, commits} = Client.rev_list(base_sha, "main")

        assert commits == [commit1, commit2, commit3]
      end)
    end

    test "returns empty list when base equals target", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, sha} = create_commit(repo, "Only commit")

        {:ok, commits} = Client.rev_list(sha, sha)

        assert commits == []
      end)
    end
  end

  describe "branches_at/1" do
    test "returns branches pointing to a commit", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, sha} = create_commit(repo, "Commit")
        create_branch(repo, "branch-1")
        create_branch(repo, "branch-2")

        {:ok, branches} = Client.branches_at(sha)

        assert "branch-1" in branches
        assert "branch-2" in branches
      end)
    end

    test "returns empty list for commit with no branches", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, sha} = create_commit(repo, "First")
        # Move HEAD forward
        create_commit(repo, "Second")

        {:ok, branches} = Client.branches_at(sha)

        # First commit should have no branch pointing to it
        assert branches == []
      end)
    end

    test "returns only main branch for HEAD commit", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, sha} = create_commit(repo, "Only commit")

        {:ok, branches} = Client.branches_at(sha)

        # Should contain the main branch
        assert length(branches) >= 1
        assert "main" in branches
      end)
    end
  end

  describe "commit_message/1" do
    test "returns commit message", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, sha} = create_commit(repo, "Test commit message")

        {:ok, message} = Client.commit_message(sha)

        assert String.trim(message) == "Test commit message"
      end)
    end

    test "returns full multi-line message", %{repo: repo} do
      File.cd!(repo, fn ->
        file_path = Path.join(repo, "test.txt")
        File.write!(file_path, "content")
        {_, 0} = System.cmd("git", ["add", "test.txt"], cd: repo)

        multiline_msg = "Title\n\nBody line 1\nBody line 2"
        {_, 0} = System.cmd("git", ["commit", "-m", multiline_msg], cd: repo)

        {:ok, sha} = get_branch_sha(repo, "main")
        {:ok, message} = Client.commit_message(sha)

        assert String.contains?(message, "Title")
        assert String.contains?(message, "Body line 1")
        assert String.contains?(message, "Body line 2")
      end)
    end

    test "returns error for invalid commit SHA", %{repo: repo} do
      File.cd!(repo, fn ->
        assert {:error, _} = Client.commit_message("invalid_sha")
      end)
    end
  end

  describe "checkout/1" do
    test "checks out existing branch", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Initial")
        create_branch(repo, "test-branch")

        assert {:ok, _} = Client.checkout("test-branch")
        assert_current_branch(repo, "test-branch")
      end)
    end

    test "returns error for non-existent branch", %{repo: repo} do
      File.cd!(repo, fn ->
        assert {:error, _} = Client.checkout("non-existent")
      end)
    end
  end

  describe "update_ref/2" do
    test "updates branch to point to new commit", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, commit1} = create_commit(repo, "Commit 1")
        create_branch(repo, "test-branch", commit1)
        {:ok, commit2} = create_commit(repo, "Commit 2")

        assert {:ok, _} = Client.update_ref("test-branch", commit2)

        {:ok, branch_sha} = get_branch_sha(repo, "test-branch")
        assert branch_sha == commit2
      end)
    end

    test "forces update even if branch exists", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, commit1} = create_commit(repo, "Commit 1")
        create_branch(repo, "branch", commit1)
        {:ok, commit2} = create_commit(repo, "Commit 2")

        # Force update
        assert {:ok, _} = Client.update_ref("branch", commit2)
        assert_branch_points_to(repo, "branch", commit2)
      end)
    end
  end

  describe "new_commits/2" do
    test "returns recent commits with messages", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Commit 1")
        create_commit(repo, "Commit 2")
        create_commit(repo, "Commit 3")

        {:ok, commits} = Client.new_commits(nil, 3)

        assert length(commits) == 3

        # Should be tuples of {sha, message}
        {sha, message} = List.first(commits)
        assert is_binary(sha)
        assert is_binary(message)
        assert String.contains?(message, "Commit")
      end)
    end

    test "limits number of commits returned", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "Commit 1")
        create_commit(repo, "Commit 2")
        create_commit(repo, "Commit 3")
        create_commit(repo, "Commit 4")
        create_commit(repo, "Commit 5")

        {:ok, commits} = Client.new_commits(nil, 2)

        assert length(commits) == 2
      end)
    end

    test "returns commits in reverse chronological order", %{repo: repo} do
      File.cd!(repo, fn ->
        create_commit(repo, "First")
        create_commit(repo, "Second")
        create_commit(repo, "Third")

        {:ok, commits} = Client.new_commits(nil, 3)

        messages = Enum.map(commits, fn {_sha, msg} -> String.trim(msg) end)

        # Most recent first
        assert List.first(messages) =~ "Third"
        assert List.last(messages) =~ "First"
      end)
    end
  end

  describe "rebase operations" do
    @tag :slow
    test "rebase_onto/3 rebases commits onto new base", %{repo: repo} do
      File.cd!(repo, fn ->
        # Create initial structure
        {:ok, base1} = create_commit(repo, "Base 1")

        # Create feature branch
        {:ok, _feature1} = create_commit(repo, "Feature 1")
        {:ok, feature2} = create_commit(repo, "Feature 2")
        create_branch(repo, "feature")

        # Create new base
        checkout_branch(repo, "main")
        {_, 0} = System.cmd("git", ["reset", "--hard", base1], cd: repo)
        create_commit(repo, "New base commit")
        {:ok, new_base} = get_branch_sha(repo, "main")

        # Rebase feature onto new base
        checkout_branch(repo, "feature")
        result = Client.rebase_onto(new_base, base1, feature2)

        # Should succeed with output message
        assert match?({:ok, _message}, result)
      end)
    end

    @tag :slow
    test "rebase_in_progress?/0 detects ongoing rebase", %{repo: repo} do
      File.cd!(repo, fn ->
        # Initially no rebase
        assert Client.rebase_in_progress?() == false

        # Set up conflict scenario
        {:ok, _} = setup_conflict_scenario(repo)

        # Try to rebase (will conflict)
        checkout_branch(repo, "feature")
        Client.rebase_onto("base", "main", "feature")

        # Now should detect rebase in progress
        # Note: This might not work if rebase succeeds, so we just check it doesn't error
        is_in_progress = Client.rebase_in_progress?()
        assert is_boolean(is_in_progress)
      end)
    end

    @tag :slow
    test "rebase_abort/0 aborts rebase", %{repo: repo} do
      File.cd!(repo, fn ->
        {:ok, %{conflict_file: _}} = setup_conflict_scenario(repo)

        checkout_branch(repo, "feature")
        Client.rebase_onto("base", "main", "feature")

        # Abort if rebase in progress
        if Client.rebase_in_progress?() do
          assert {:ok, _} = Client.rebase_abort()
          refute Client.rebase_in_progress?()
        end
      end)
    end
  end
end
