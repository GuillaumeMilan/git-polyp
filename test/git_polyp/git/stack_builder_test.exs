defmodule GitPolyp.Git.StackBuilderTest do
  use ExUnit.Case, async: true

  alias GitPolyp.Git.StackBuilder
  import GitPolyp.Assertions

  describe "format_stack/1" do
    test "formats single stack entry correctly" do
      stack = [
        %{commit: "abc1234567890", branches: ["feature-1"], message: "Add feature"}
      ]

      result = StackBuilder.format_stack(stack)

      assert result =~ "*"
      assert result =~ "abc123456"
      assert result =~ "feature-1"
      assert result =~ "Add feature"
    end

    test "formats multiple stack entries" do
      stack = [
        %{commit: "abc1234567890", branches: ["feature-1"], message: "First commit"},
        %{commit: "def4567890123", branches: ["feature-2"], message: "Second commit"},
        %{commit: "ghi7890123456", branches: ["feature-3"], message: "Third commit"}
      ]

      result = StackBuilder.format_stack(stack)

      lines = String.split(result, "\n")

      # Should have 3 lines (1 per entry)
      assert length(lines) == 3

      # Check all lines start with *
      assert Enum.all?(lines, &String.starts_with?(&1, "*"))

      # Check commit SHAs are truncated to 9 chars
      assert Enum.at(lines, 0) =~ "abc123456"
      assert Enum.at(lines, 1) =~ "def456789"
      assert Enum.at(lines, 2) =~ "ghi789012"
    end

    test "formats entry with multiple branches" do
      stack = [
        %{
          commit: "abc1234567890",
          branches: ["feature-1", "feature-2", "hotfix"],
          message: "Multi-branch commit"
        }
      ]

      result = StackBuilder.format_stack(stack)

      assert result =~ "feature-1, feature-2, hotfix"
    end

    test "formats entry with no branches" do
      stack = [
        %{commit: "abc1234567890", branches: [], message: "No branch commit"}
      ]

      result = StackBuilder.format_stack(stack)

      # Should not have parentheses when there are no branches
      refute result =~ "("
      assert result =~ "abc123456"
      assert result =~ "No branch commit"
    end

    test "shows only first line of multi-line commit message" do
      stack = [
        %{
          commit: "abc1234567890",
          branches: ["feature-1"],
          message: "First line\nSecond line\nThird line"
        }
      ]

      result = StackBuilder.format_stack(stack)

      assert result =~ "First line"
      refute result =~ "Second line"
      refute result =~ "Third line"
    end

    test "truncates commit SHA to 9 characters" do
      stack = [
        %{
          commit: "abcdefghijklmnopqrstuvwxyz123456",
          branches: ["br"],
          message: "Msg"
        }
      ]

      result = StackBuilder.format_stack(stack)

      assert result =~ "abcdefghi"
      refute result =~ "jklmnop"
    end

    test "handles empty stack" do
      result = StackBuilder.format_stack([])
      assert result == ""
    end

    test "format includes dash separator" do
      stack = [
        %{commit: "abc1234567890", branches: ["br"], message: "Message"}
      ]

      result = StackBuilder.format_stack(stack)

      # Should include " - " separator
      assert result =~ " - "
    end

    test "formats complete example correctly" do
      stack = [
        %{commit: "a1b2c3d4e5f6", branches: ["feat-1"], message: "Add authentication"},
        %{commit: "f6e5d4c3b2a1", branches: [], message: "Update tests"},
        %{commit: "123456789abc", branches: ["feat-2", "dev"], message: "Fix bug\nAdditional details"}
      ]

      result = StackBuilder.format_stack(stack)

      # Strip ANSI codes for testing
      stripped = strip_ansi(result)

      # Verify structure - new format: * <hash> - (<branches>) <message>
      assert stripped =~ "* a1b2c3d4e"
      assert stripped =~ "feat-1"
      assert stripped =~ "Add authentication"
      assert stripped =~ "* f6e5d4c3b"
      assert stripped =~ "Update tests"
      assert stripped =~ "* 123456789"
      assert stripped =~ "feat-2, dev"
      assert stripped =~ "Fix bug"
      refute stripped =~ "Additional details"
    end
  end

  describe "validate_linear_stack/1" do
    test "always returns :ok for now" do
      stack = [
        %{commit: "abc", branches: ["br1"], message: "msg"}
      ]

      assert :ok = StackBuilder.validate_linear_stack(stack)
    end

    test "returns :ok for empty stack" do
      assert :ok = StackBuilder.validate_linear_stack([])
    end

    test "returns :ok for multi-entry stack" do
      stack = [
        %{commit: "abc", branches: ["br1"], message: "msg1"},
        %{commit: "def", branches: ["br2"], message: "msg2"},
        %{commit: "ghi", branches: ["br3"], message: "msg3"}
      ]

      assert :ok = StackBuilder.validate_linear_stack(stack)
    end
  end

  describe "build_stack/2 (integration)" do
    # These tests require actual Git operations and will be tested
    # in the integration test file with real repositories

    test "placeholder for integration tests" do
      # Integration tests for build_stack/2 will be in client_test.exs
      # and full_workflow_test.exs with real git repositories
      assert true
    end
  end
end
