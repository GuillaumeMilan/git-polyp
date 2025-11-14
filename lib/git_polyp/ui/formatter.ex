defmodule GitPolyp.UI.Formatter do
  @moduledoc """
  Provides color-coded formatting for terminal output.

  Uses IO.ANSI for colorization to improve readability of command output.
  """

  @doc """
  Formats an error message in red.
  """
  def error(message) do
    IO.ANSI.format([:red, :bright, "Error: ", :reset, :red, message, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats a success message in green.
  """
  def success(message) do
    IO.ANSI.format([:green, :bright, "✓ ", :reset, :green, message, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats a warning message in yellow.
  """
  def warning(message) do
    IO.ANSI.format([:yellow, :bright, "Warning: ", :reset, :yellow, message, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats an info message in blue.
  """
  def info(message) do
    IO.ANSI.format([:blue, :bright, "→ ", :reset, message])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats a section header.
  """
  def header(message) do
    IO.ANSI.format([:cyan, :bright, message, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats a branch name with highlighting.
  """
  def branch(name) do
    IO.ANSI.format([:green, :bright, name, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats a commit SHA with highlighting.
  """
  def commit(sha) do
    short_sha = String.slice(sha, 0..7)

    IO.ANSI.format([:yellow, short_sha, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats a command that the user should run.
  """
  def command(cmd) do
    IO.ANSI.format([:cyan, :bright, "  $ ", :reset, :cyan, cmd, :reset])
    |> IO.iodata_to_binary()
  end

  @doc """
  Formats instructions for the user.
  """
  def instructions(lines) when is_list(lines) do
    lines
    |> Enum.map(&"  #{&1}")
    |> Enum.join("\n")
  end

  @doc """
  Prints a horizontal separator line.
  """
  def separator do
    String.duplicate("─", 60)
  end
end
