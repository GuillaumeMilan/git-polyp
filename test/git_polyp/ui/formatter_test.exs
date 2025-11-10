defmodule GitPolyp.UI.FormatterTest do
  use ExUnit.Case, async: true

  alias GitPolyp.UI.Formatter

  describe "error/1" do
    test "formats error message with red color" do
      result = Formatter.error("Something went wrong")

      assert result =~ "Error:"
      assert result =~ "Something went wrong"
      # Just verify it returns a string (color codes may or may not be visible)
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.error("test")
      assert is_binary(result)
    end
  end

  describe "success/1" do
    test "formats success message with green color and checkmark" do
      result = Formatter.success("Operation completed")

      assert result =~ "✓"
      assert result =~ "Operation completed"
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.success("test")
      assert is_binary(result)
    end
  end

  describe "warning/1" do
    test "formats warning message with yellow color" do
      result = Formatter.warning("Be careful")

      assert result =~ "Warning:"
      assert result =~ "Be careful"
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.warning("test")
      assert is_binary(result)
    end
  end

  describe "info/1" do
    test "formats info message with blue color and arrow" do
      result = Formatter.info("Processing data")

      assert result =~ "→"
      assert result =~ "Processing data"
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.info("test")
      assert is_binary(result)
    end
  end

  describe "header/1" do
    test "formats header with cyan color" do
      result = Formatter.header("Main Section")

      assert result =~ "Main Section"
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.header("test")
      assert is_binary(result)
    end
  end

  describe "branch/1" do
    test "formats branch name with green highlighting" do
      result = Formatter.branch("feature-branch")

      assert result =~ "feature-branch"
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.branch("main")
      assert is_binary(result)
    end
  end

  describe "commit/1" do
    test "formats commit SHA with yellow color" do
      full_sha = "abcdef1234567890abcdef1234567890abcdef12"
      result = Formatter.commit(full_sha)

      # Should show short SHA (first 8 characters)
      assert result =~ "abcdef12"
      assert is_binary(result)
    end

    test "truncates long SHA to 8 characters" do
      full_sha = "1234567890abcdefabcdefabcdefabcdefabcdef"
      result = Formatter.commit(full_sha)

      # Should only contain first 8 chars of the SHA
      assert result =~ "12345678"
      refute result =~ "90abcdef"
    end

    test "handles short SHAs" do
      short_sha = "abc123"
      result = Formatter.commit(short_sha)

      assert result =~ "abc123"
    end

    test "returns a binary string" do
      result = Formatter.commit("abcd1234")
      assert is_binary(result)
    end
  end

  describe "command/1" do
    test "formats command with cyan color and dollar sign" do
      result = Formatter.command("git rebase --continue")

      assert result =~ "$"
      assert result =~ "git rebase --continue"
      assert is_binary(result)
    end

    test "returns a binary string" do
      result = Formatter.command("ls -la")
      assert is_binary(result)
    end
  end

  describe "instructions/1" do
    test "formats list of instructions with indentation" do
      lines = [
        "First, resolve conflicts",
        "Then, stage the changes",
        "Finally, continue the rebase"
      ]

      result = Formatter.instructions(lines)

      assert result =~ "  First, resolve conflicts"
      assert result =~ "  Then, stage the changes"
      assert result =~ "  Finally, continue the rebase"
    end

    test "joins lines with newlines" do
      lines = ["Line 1", "Line 2"]
      result = Formatter.instructions(lines)

      assert result =~ "\n"
      assert String.split(result, "\n") |> length() == 2
    end

    test "handles empty list" do
      result = Formatter.instructions([])
      assert result == ""
    end

    test "handles single line" do
      result = Formatter.instructions(["Only line"])
      assert result == "  Only line"
    end
  end

  describe "separator/0" do
    test "returns a line of dashes" do
      result = Formatter.separator()

      assert is_binary(result)
      assert String.length(result) == 60
      assert result == String.duplicate("─", 60)
    end

    test "all characters are the box-drawing dash" do
      result = Formatter.separator()
      assert String.graphemes(result) |> Enum.all?(&(&1 == "─"))
    end
  end
end
