defmodule GitPolyp.Git.BranchUpdaterTest do
  use ExUnit.Case, async: true

  alias GitPolyp.Git.BranchUpdater

  describe "normalize_message/1 (via update_branches_with_commits/2)" do
    test "matches commits with exact same message" do
      stack = [
        %{commit: "old123", branches: ["feature-1"], message: "Add feature"}
      ]

      new_commits = [
        {"new456", "Add feature"}
      ]

      # Mock Client.update_ref
      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:ok, updates} = BranchUpdater.update_branches_with_commits(stack, new_commits)
        assert length(updates) == 1
        assert List.first(updates).old_commit == "old123"
        assert List.first(updates).new_commit == "new456"
      end)
    end

    test "matches commits with whitespace differences" do
      stack = [
        %{commit: "old123", branches: ["feature-1"], message: "  Add   feature  "}
      ]

      new_commits = [
        {"new456", "Add feature"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:ok, updates} = BranchUpdater.update_branches_with_commits(stack, new_commits)
        assert length(updates) == 1
      end)
    end

    test "matches commits with multiple spaces normalized to single space" do
      stack = [
        %{commit: "old123", branches: ["feature-1"], message: "Add    new    feature"}
      ]

      new_commits = [
        {"new456", "Add new feature"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:ok, updates} = BranchUpdater.update_branches_with_commits(stack, new_commits)
        assert length(updates) == 1
      end)
    end

    test "matches commits with tabs and newlines normalized" do
      stack = [
        %{commit: "old123", branches: ["feature-1"], message: "Add\t\nfeature"}
      ]

      new_commits = [
        {"new456", "Add feature"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:ok, updates} = BranchUpdater.update_branches_with_commits(stack, new_commits)
        assert length(updates) == 1
      end)
    end
  end

  describe "format_updates/1" do
    test "formats single update correctly" do
      updates = [
        %{branch: "feature-1", old_commit: "abc1234567", new_commit: "xyz9876543"}
      ]

      result = BranchUpdater.format_updates(updates)

      assert result =~ "feature-1"
      assert result =~ "abc12345"
      assert result =~ "xyz98765"
      assert result =~ "→"
    end

    test "formats multiple updates with newlines" do
      updates = [
        %{branch: "feature-1", old_commit: "abc1234567", new_commit: "xyz9876543"},
        %{branch: "feature-2", old_commit: "def1234567", new_commit: "uvw9876543"}
      ]

      result = BranchUpdater.format_updates(updates)

      lines = String.split(result, "\n")
      assert length(lines) == 2
      assert Enum.at(lines, 0) =~ "feature-1"
      assert Enum.at(lines, 1) =~ "feature-2"
    end

    test "truncates commit SHAs to 8 characters" do
      updates = [
        %{branch: "br", old_commit: "abcdefghijklmnop", new_commit: "1234567890abcdef"}
      ]

      result = BranchUpdater.format_updates(updates)

      assert result =~ "abcdefgh"
      assert result =~ "12345678"
      refute result =~ "ijklmnop"
      refute result =~ "90abcdef"
    end

    test "handles empty list" do
      result = BranchUpdater.format_updates([])
      assert result == ""
    end

    test "indents each line with spaces" do
      updates = [
        %{branch: "feature-1", old_commit: "abc1234567", new_commit: "xyz9876543"}
      ]

      result = BranchUpdater.format_updates(updates)

      assert String.starts_with?(result, "  ")
    end
  end

  describe "update_branches_with_commits/2" do
    test "returns successful updates when all commits match" do
      stack = [
        %{commit: "old1", branches: ["feature-1"], message: "First"},
        %{commit: "old2", branches: ["feature-2"], message: "Second"}
      ]

      new_commits = [
        {"new1", "First"},
        {"new2", "Second"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:ok, updates} = BranchUpdater.update_branches_with_commits(stack, new_commits)
        assert length(updates) == 2
      end)
    end

    test "updates multiple branches pointing to same commit" do
      stack = [
        %{commit: "old1", branches: ["feature-1", "feature-2", "feature-3"], message: "Commit"}
      ]

      new_commits = [
        {"new1", "Commit"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:ok, updates} = BranchUpdater.update_branches_with_commits(stack, new_commits)
        assert length(updates) == 3

        branch_names = Enum.map(updates, & &1.branch)
        assert "feature-1" in branch_names
        assert "feature-2" in branch_names
        assert "feature-3" in branch_names
      end)
    end

    test "returns warning when some commits don't match" do
      stack = [
        %{commit: "old1", branches: ["feature-1"], message: "First"},
        %{commit: "old2", branches: ["feature-2"], message: "Unmatched message"}
      ]

      new_commits = [
        {"new1", "First"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:warning, {:unmatched_commits, unmatched, successful}} =
                 BranchUpdater.update_branches_with_commits(stack, new_commits)

        assert length(successful) == 1
        assert length(unmatched) == 1

        {:unmatched, branch, commit, message} = List.first(unmatched)
        assert branch == "feature-2"
        assert commit == "old2"
        assert message == "Unmatched message"
      end)
    end

    test "returns error when update_ref fails" do
      stack = [
        %{commit: "old1", branches: ["feature-1"], message: "First"}
      ]

      new_commits = [
        {"new1", "First"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:error, "update failed"} end, fn ->
        assert {:error, {:update_failed, errors}} =
                 BranchUpdater.update_branches_with_commits(stack, new_commits)

        assert length(errors) == 1
        {:error, error_info} = List.first(errors)
        assert error_info.branch == "feature-1"
        assert error_info.reason == "update failed"
      end)
    end

    test "processes stack in reverse order" do
      # Stack should be processed from newest to oldest
      stack = [
        %{commit: "old1", branches: ["first"], message: "First"},
        %{commit: "old2", branches: ["second"], message: "Second"},
        %{commit: "old3", branches: ["third"], message: "Third"}
      ]

      new_commits = [
        {"new1", "First"},
        {"new2", "Second"},
        {"new3", "Third"}
      ]

      with_mock_update_ref(
        fn branch, _sha ->
          send(self(), {:update, branch})
          {:ok, nil}
        end,
        fn ->
          BranchUpdater.update_branches_with_commits(stack, new_commits)

          # Collect all update calls
          updates = collect_update_messages([])

          # Stack is reversed, so third should be updated first
          assert List.first(updates) == "third"
          assert List.last(updates) == "first"
        end
      )
    end

    test "handles empty stack" do
      assert {:ok, []} = BranchUpdater.update_branches_with_commits([], [])
    end

    test "handles empty new commits with non-empty stack" do
      stack = [
        %{commit: "old1", branches: ["feature-1"], message: "First"}
      ]

      with_mock_update_ref(fn _branch, _sha -> {:ok, nil} end, fn ->
        assert {:warning, {:unmatched_commits, unmatched, successful}} =
                 BranchUpdater.update_branches_with_commits(stack, [])

        assert length(successful) == 0
        assert length(unmatched) == 1
      end)
    end
  end

  # Helper function to mock Client.update_ref
  defp with_mock_update_ref(mock_fn, test_fn) do
    try do
      # Replace with mock
      :meck.new(GitPolyp.Git.Client, [:passthrough])
      :meck.expect(GitPolyp.Git.Client, :update_ref, mock_fn)

      test_fn.()
    after
      # Restore original
      :meck.unload(GitPolyp.Git.Client)
    end
  end

  # Helper to collect update messages
  defp collect_update_messages(acc) do
    receive do
      {:update, branch} -> collect_update_messages([branch | acc])
    after
      10 -> Enum.reverse(acc)
    end
  end
end
