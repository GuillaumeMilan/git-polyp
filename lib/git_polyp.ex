defmodule GitPolyp do
  @moduledoc """
  GitPolyp - A CLI and Elixir library toolkit for advanced git automation.

  GitPolyp provides powerful commands and programmatic interfaces to streamline
  complex git workflows and repository management.

  ## CLI Usage

  The main entry point for the CLI is through the `git-polyp` executable:

      git-polyp rebase-stack main feature-branch

  ## Available Commands

  - `rebase-stack` - Rebase a linear stack of branches onto a new base

  For more information, run:

      git-polyp --help
  """

  @doc """
  Returns the version of GitPolyp.
  """
  def version do
    Application.spec(:git_polyp, :vsn)
    |> to_string()
  end
end
