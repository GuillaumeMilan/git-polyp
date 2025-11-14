defmodule GitPolyp.CLITest do
  use ExUnit.Case

  import ExUnit.CaptureIO

  alias GitPolyp.CLI

  # Note: We can't easily test System.halt/1 calls, so we'll focus on
  # testing the output and behavior before the halt

  describe "main/1 with help flag" do
    test "displays help with --help flag" do
      output =
        capture_io(fn ->
          # Catch the exit to prevent test from halting
          catch_exit(CLI.main(["--help"]))
        end)

      assert output =~ "git-polyp"
      assert output =~ "USAGE:"
      assert output =~ "COMMANDS:"
      assert output =~ "rebase-stack"
    end

    test "displays help with -h flag" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["-h"]))
        end)

      assert output =~ "git-polyp"
      assert output =~ "USAGE:"
    end

    test "displays help when no arguments provided" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main([]))
        end)

      assert output =~ "git-polyp"
      assert output =~ "USAGE:"
    end
  end

  describe "main/1 with version flag" do
    test "displays version with --version flag" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--version"]))
        end)

      assert output =~ "git-polyp version"
    end

    test "displays version with -v flag" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["-v"]))
        end)

      assert output =~ "git-polyp version"
    end
  end

  describe "main/1 with unknown command" do
    test "shows error for unknown command" do
      output =
        capture_io(:stderr, fn ->
          catch_exit(CLI.main(["unknown-command"]))
        end)

      assert output =~ "Unknown command: unknown-command"
      assert output =~ "Run 'git-polyp --help'"
    end

    test "shows error for typo in command name" do
      output =
        capture_io(:stderr, fn ->
          # typo
          catch_exit(CLI.main(["rebase-stck"]))
        end)

      assert output =~ "Unknown command: rebase-stck"
    end
  end

  describe "main/1 with rebase-stack command" do
    test "routes to rebase-stack command" do
      # We can't fully test this without mocking the RebaseStack module,
      # but we can verify the routing works by checking it doesn't error
      # on an unknown command error

      # This will likely fail due to validation, but shouldn't be "unknown command"
      output =
        capture_io(:stderr, fn ->
          catch_exit(CLI.main(["rebase-stack"]))
        end)

      # Should not say "Unknown command"
      refute output =~ "Unknown command: rebase-stack"
    end
  end

  describe "help output format" do
    test "includes all required sections" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--help"]))
        end)

      assert output =~ "USAGE:"
      assert output =~ "COMMANDS:"
      assert output =~ "GLOBAL OPTIONS:"
      assert output =~ "EXAMPLES:"
    end

    test "shows rebase-stack in commands" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--help"]))
        end)

      assert output =~ "rebase-stack"
      assert output =~ "Rebase a linear stack"
    end

    test "shows global options" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--help"]))
        end)

      assert output =~ "--help"
      assert output =~ "--version"
      assert output =~ "-h"
      assert output =~ "-v"
    end

    test "includes examples" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--help"]))
        end)

      assert output =~ "git-polyp rebase-stack main feature-branch"
      assert output =~ "--continue"
      assert output =~ "--abort"
    end
  end

  describe "version output" do
    test "shows version number or dev" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--version"]))
        end)

      # Should show either a version number or "dev"
      assert output =~ ~r/git-polyp version \S+/
    end
  end

  describe "argument parsing" do
    test "parses help flag before command" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--help", "rebase-stack"]))
        end)

      # Should show help, not run rebase-stack
      assert output =~ "USAGE:"
    end

    test "parses version flag before command" do
      output =
        capture_io(fn ->
          catch_exit(CLI.main(["--version", "rebase-stack"]))
        end)

      # Should show version, not run rebase-stack
      assert output =~ "git-polyp version"
    end
  end

  describe "error handling" do
    test "writes errors to stderr" do
      stderr_output =
        capture_io(:stderr, fn ->
          catch_exit(CLI.main(["invalid-cmd"]))
        end)

      stdout_output =
        capture_io(fn ->
          catch_exit(CLI.main(["invalid-cmd"]))
        end)

      # Error should be in stderr, not stdout
      assert stderr_output =~ "Unknown command"
      assert stdout_output == ""
    end

    test "error messages include colored formatting" do
      output =
        capture_io(:stderr, fn ->
          catch_exit(CLI.main(["bad-command"]))
        end)

      # Should have ANSI color codes (from Formatter.error)
      assert output =~ "\e["
    end
  end
end
