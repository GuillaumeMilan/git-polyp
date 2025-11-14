defmodule GitPolyp.CLI do
  @moduledoc """
  Main CLI entry point for git-polyp.

  Handles command routing and global options using OptionParser.
  """

  alias GitPolyp.Commands.{RebaseStack, InstallCompletions}
  alias GitPolyp.UI.Formatter

  @doc """
  Main entry point for the escript.
  """
  def main(argv) do
    # Parse global options
    {opts, remaining_args, invalid} =
      OptionParser.parse(argv,
        strict: [
          help: :boolean,
          version: :boolean
        ],
        aliases: [
          h: :help,
          v: :version
        ]
      )

    cond do
      opts[:help] ->
        print_help()
        exit_success()

      opts[:version] ->
        print_version()
        exit_success()

      remaining_args == [] and invalid == [] ->
        print_help()
        exit_success()

      remaining_args == [] and invalid != [] ->
        # Handle invalid options for this command
        exit_error(
          "Invalid options: #{Enum.map(invalid, fn {opt, _} -> opt end) |> Enum.join(", ")}\n\n#{usage_hint()}"
        )

      true ->
        # Route with all args as invalid options might be for subcommands
        route_command(argv)
    end
  end

  # Routes to the appropriate command based on the first argument
  defp route_command([]) do
    print_help()
    exit_success()
  end

  defp route_command(["rebase-stack" | args]) do
    case RebaseStack.run(args) do
      :ok -> exit_success()
      {:error, message} -> exit_error(message)
    end
  end

  defp route_command(["install" | args]) do
    # For the moment install only installs completions
    case InstallCompletions.run(args) do
      :ok -> exit_success()
      {:error, message} -> exit_error(message)
    end
  end

  defp route_command([subcommand | _]) do
    exit_error("Unknown command: #{subcommand}\n\n#{usage_hint()}")
  end

  defp print_help do
    IO.puts("""
    #{Formatter.header("git-polyp")} - Git automation toolkit

    #{Formatter.header("USAGE:")}
      git-polyp <command> [options]

    #{Formatter.header("COMMANDS:")}
      rebase-stack         Rebase a linear stack of branches onto a new base
      install  Install shell completion scripts

    #{Formatter.header("GLOBAL OPTIONS:")}
      -h, --help      Show this help message
      -v, --version   Show version information

    #{Formatter.header("EXAMPLES:")}
      # Rebase a stack of branches
      git-polyp rebase-stack main feature-branch

      # Continue after resolving conflicts
      git-polyp rebase-stack --continue

      # Abort the rebase
      git-polyp rebase-stack --abort

      # Install shell completions
      git-polyp install

    For more information on a specific command:
      git-polyp <command> --help
    """)
  end

  defp print_version do
    version = Application.spec(:git_polyp, :vsn) || "dev"
    IO.puts("git-polyp version #{version}")
  end

  defp usage_hint do
    "Run 'git-polyp --help' for usage information."
  end

  defp exit_success, do: System.halt(0)

  defp exit_error(message) do
    IO.puts(:stderr, Formatter.error(message))
    System.halt(1)
  end
end
