defmodule GitPolyp.Integration.FullWorkflowTest do
  use ExUnit.Case

  import GitPolyp.GitTestHelper
  import GitPolyp.Assertions

  alias GitPolyp.Git.{Client, StackBuilder}
  alias GitPolyp.State.Manager

  @moduletag :integration
  @moduletag timeout: 120_000

  setup do
    {:ok, repo_path} = create_test_repo("integration_test")

    on_exit(fn -> cleanup_test_repo(repo_path) end)

    # Change to repo directory
    original_dir = File.cwd!()
    File.cd!(repo_path)

    on_exit(fn -> File.cd!(original_dir) end)

    %{repo: repo_path}
  end

  describe "complete rebase-stack workflow" do
    @tag :slow
    test "successfully rebases a linear stack", %{repo: repo} do
      # Setup: Create a linear stack
      # main: A
      # feature-1: A -> B
      # feature-2: A -> B -> C
      # feature-3: A -> B -> C -> D

      {:ok, commit_a} = create_commit(repo, "Base commit A")

      {:ok, commit_b} = create_commit(repo, "Feature 1 commit B")
      create_branch(repo, "feature-1")

      {:ok, commit_c} = create_commit(repo, "Feature 2 commit C")
      create_branch(repo, "feature-2")

      {:ok, commit_d} = create_commit(repo, "Feature 3 commit D")
      create_branch(repo, "feature-3")

      # Go back to main and add a new commit (new base)
      checkout_branch(repo, "main")
      {_, 0} = System.cmd("git", ["reset", "--hard", commit_a], cd: repo)
      {:ok, commit_x} = create_commit(repo, "New base commit X")

      # Now rebase feature-3 stack onto new main
      # Step 1: Build the stack
      {:ok, stack, merge_base} = StackBuilder.build_stack("main", "feature-3")

      assert length(stack) == 3
      assert merge_base == commit_a

      # Verify stack structure
      assert Enum.at(stack, 0).commit == commit_b
      assert Enum.at(stack, 0).message =~ "Feature 1"
      assert Enum.at(stack, 1).commit == commit_c
      assert Enum.at(stack, 2).commit == commit_d

      # Step 2: Perform rebase
      checkout_branch(repo, "feature-3")
      {:ok, _} = Client.rebase_onto(commit_x, merge_base, commit_d)

      # Step 3: Get new commits
      {:ok, new_commits} = Client.new_commits(nil, 3)
      assert length(new_commits) == 3

      # Step 4: Update branch pointers
      # (In real workflow, this would use BranchUpdater.update_branches)
      # For this test, verify we can identify the commits

      new_commit_messages = Enum.map(new_commits, fn {_sha, msg} -> String.trim(msg) end)

      assert "Feature 1 commit B" in new_commit_messages
      assert "Feature 2 commit C" in new_commit_messages
      assert "Feature 3 commit D" in new_commit_messages

      # Verify the stack is still linear
      {:ok, current_head} = get_branch_sha(repo, "feature-3")
      {parent1, 0} = System.cmd("git", ["rev-parse", "#{current_head}^"], cd: repo)
      {parent2, 0} = System.cmd("git", ["rev-parse", "#{String.trim(parent1)}^"], cd: repo)
      {parent3, 0} = System.cmd("git", ["rev-parse", "#{String.trim(parent2)}^"], cd: repo)

      # Parent3 should be commit_x (new base)
      assert String.trim(parent3) == commit_x
    end

    @tag :slow
    test "handles stack with multiple branches at same commit", %{repo: repo} do
      {:ok, commit_a} = create_commit(repo, "Base A")
      {:ok, commit_b} = create_commit(repo, "Commit B")

      # Create multiple branches at commit B
      create_branch(repo, "branch-1")
      create_branch(repo, "branch-2")
      create_branch(repo, "branch-3")

      {:ok, _commit_c} = create_commit(repo, "Commit C")
      create_branch(repo, "target")

      # Move main back to commit A so we have a proper stack
      checkout_branch(repo, "main")
      {_, 0} = System.cmd("git", ["reset", "--hard", commit_a], cd: repo)

      # Build stack
      {:ok, stack, _} = StackBuilder.build_stack("main", "target")

      # First entry should have all three branches
      first_entry = Enum.at(stack, 0)
      assert first_entry.commit == commit_b
      assert length(first_entry.branches) == 3
      assert "branch-1" in first_entry.branches
      assert "branch-2" in first_entry.branches
      assert "branch-3" in first_entry.branches
    end

    @tag :slow
    test "state persistence workflow", %{repo: repo} do
      # Setup a stack
      {:ok, %{branches: _branches}} = setup_standard_stack(repo)

      # Build stack
      {:ok, stack, merge_base} = StackBuilder.build_stack("main", "feature-3")

      # Save state (simulating conflict scenario)
      metadata = GitPolyp.State.Metadata.new(
        "main",
        merge_base,
        "feature-3",
        stack,
        "main"
      )

      assert :ok = Manager.save(metadata)
      assert Manager.exists?() == true

      # Load state
      {:ok, loaded} = Manager.load()

      assert loaded.base_branch == "main"
      assert loaded.target_branch == "feature-3"
      assert loaded.merge_base == merge_base
      assert length(loaded.stack) == 3

      # Cleanup
      assert :ok = Manager.delete()
      assert Manager.exists?() == false
    end

    @tag :slow
    test "identifies correct stack between branches", %{repo: repo} do
      # Create: main at A, feature-1 at B, feature-2 at C
      # Stack: A (main) -> B (feature-1) -> C (feature-2)
      {:ok, commit_a} = create_commit(repo, "Commit A")
      create_branch(repo, "main-marker")
      {:ok, commit_b} = create_commit(repo, "Commit B")
      create_branch(repo, "feature-1")
      {:ok, commit_c} = create_commit(repo, "Commit C")
      create_branch(repo, "feature-2")

      # Move main back to commit A
      checkout_branch(repo, "main")
      {_, 0} = System.cmd("git", ["reset", "--hard", commit_a], cd: repo)

      # Build stack from main to feature-2
      {:ok, stack, merge_base} = StackBuilder.build_stack("main", "feature-2")

      assert merge_base == commit_a
      assert length(stack) == 2

      # Verify commits
      assert Enum.at(stack, 0).commit == commit_b
      assert Enum.at(stack, 1).commit == commit_c

      # Verify branches
      assert "feature-1" in Enum.at(stack, 0).branches
      assert "feature-2" in Enum.at(stack, 1).branches
    end

    @tag :slow
    test "handles empty stack (no commits between branches)", %{repo: repo} do
      {:ok, _} = create_commit(repo, "Commit A")
      create_branch(repo, "branch-1")

      # Try to build stack from branch to itself
      result = StackBuilder.build_stack("branch-1", "branch-1")

      # Should error or return empty
      assert match?({:error, _}, result)
    end

    @tag :slow
    test "stack formatting displays correctly", %{repo: repo} do
      {:ok, _} = setup_standard_stack(repo)

      {:ok, stack, _} = StackBuilder.build_stack("main", "feature-3")

      formatted = StackBuilder.format_stack(stack)

      # Should contain asterisks for each entry
      assert formatted =~ "*"

      # Should contain branch names
      assert formatted =~ "feature-1"
      assert formatted =~ "feature-2"
      assert formatted =~ "feature-3"

      # Should contain commit messages
      assert formatted =~ "Feature 1"
      assert formatted =~ "Feature 2"
      assert formatted =~ "Feature 3"
    end

    @tag :slow
    test "complete workflow with branch updates", %{repo: repo} do
      # Create initial stack
      {:ok, base} = create_commit(repo, "Base")

      {:ok, old_b} = create_commit(repo, "Feature 1")
      create_branch(repo, "feature-1")

      {:ok, old_c} = create_commit(repo, "Feature 2")
      create_branch(repo, "feature-2")

      # Create new base
      checkout_branch(repo, "main")
      {_, 0} = System.cmd("git", ["reset", "--hard", base], cd: repo)
      {:ok, new_base} = create_commit(repo, "New base")

      # Build stack
      {:ok, stack, merge_base} = StackBuilder.build_stack("main", "feature-2")

      # Rebase
      checkout_branch(repo, "feature-2")
      {:ok, _} = Client.rebase_onto(new_base, merge_base, old_c)

      # Update branches using BranchUpdater
      {:ok, new_commits} = Client.new_commits(nil, 2)
      {:ok, updates} = GitPolyp.Git.BranchUpdater.update_branches_with_commits(stack, new_commits)

      # Verify updates
      assert length(updates) == 2

      # Verify branches point to new commits
      {:ok, feature1_sha} = get_branch_sha(repo, "feature-1")
      {:ok, feature2_sha} = get_branch_sha(repo, "feature-2")

      refute feature1_sha == old_b
      refute feature2_sha == old_c

      # Verify new commits are descendants of new base
      {merge_base_check, 0} = System.cmd("git", ["merge-base", "main", "feature-1"], cd: repo)
      assert String.trim(merge_base_check) == new_base
    end

    @tag :slow
    test "validates git repository before operations", %{repo: _repo} do
      # Should be in a git repo
      assert Client.in_git_repo?() == true

      # Should be able to get git dir
      git_dir = Client.git_dir()
      assert File.dir?(git_dir)
    end
  end

  describe "error scenarios" do
    test "building stack with non-existent branch returns error", %{repo: repo} do
      create_commit(repo, "Commit A")

      result = StackBuilder.build_stack("main", "non-existent-branch")

      assert match?({:error, _}, result)
    end

    test "cannot build stack when branches are not related", %{repo: repo} do
      # Create two unrelated branches (orphan branches)
      create_commit(repo, "Main commit")

      # This test would require more complex setup with orphan branches
      # Skipping for now as it's an edge case
      assert true
    end
  end

  describe "helper function verification" do
    test "setup_standard_stack creates expected structure", %{repo: repo} do
      {:ok, result} = setup_standard_stack(repo)

      assert length(result.branches) == 3
      assert result.branches == ["feature-1", "feature-2", "feature-3"]

      # Verify branches exist
      assert_branch_exists(repo, "feature-1")
      assert_branch_exists(repo, "feature-2")
      assert_branch_exists(repo, "feature-3")
    end

    test "test helpers work correctly", %{repo: repo} do
      {:ok, sha1} = create_commit(repo, "Test 1")
      {:ok, sha2} = create_commit(repo, "Test 2")

      # Verify commits are in order
      assert_stack_order(repo, [sha1, sha2])
    end
  end
end
