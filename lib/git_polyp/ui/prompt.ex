defmodule GitPolyp.UI.Prompt do
  @moduledoc """
  Handles user prompts and input validation.
  """

  @doc """
  Prompts the user for confirmation with a yes/no question.

  Returns `true` if user confirms, `false` otherwise.
  Default is "yes" if user just presses Enter.

  ## Examples
      iex> GitPolyp.UI.Prompt.confirm("Proceed with rebase?")
      # User input: y
      true
  """
  def confirm(message, default \\ true) do
    default_hint = if default, do: "Y/n", else: "y/N"
    prompt = "#{message} [#{default_hint}]: "

    IO.write(prompt)

    case IO.gets("") |> String.trim() |> String.downcase() do
      "" -> default
      "y" -> true
      "yes" -> true
      "n" -> false
      "no" -> false
      _ -> confirm(message, default)
    end
  end

  @doc """
  Prompts the user with a message and waits for them to press Enter.
  """
  def press_enter(message \\ "Press Enter to continue") do
    IO.write("#{message}...")
    IO.gets("")
    :ok
  end
end
