defmodule GitPolyp.Git.BranchUpdater do
  @moduledoc """
  Updates branch pointers after a successful rebase operation.

  Matches old commits to new commits by commit message and updates
  each branch to point to its corresponding new commit.
  """

  alias GitPolyp.Git.Client

  @doc """
  Updates all branches in the stack to point to their new commits after rebase.

  Takes the original stack and the number of commits that were rebased.
  Fetches the new commits from HEAD and matches them to the old ones by message.

  Returns `{:ok, updates}` where updates is a list of successful branch updates,
  or `{:error, reason}` if something goes wrong.

  ## Examples
      iex> GitPolyp.Git.BranchUpdater.update_branches(stack, 3)
      {:ok, [
        %{branch: "feature-1", old_commit: "abc123", new_commit: "xyz789"},
        %{branch: "feature-2", old_commit: "def456", new_commit: "uvw012"}
      ]}
  """
  def update_branches(stack, commit_count) do
    with {:ok, new_commits_with_messages} <- Client.new_commits(nil, commit_count) do
      update_branches_with_commits(stack, new_commits_with_messages)
    end
  end

  @doc """
  Updates all branches in the stack using pre-fetched commit list.

  This is useful when you need to fetch commits before changing branches.
  """
  def update_branches_with_commits(stack, new_commits_with_messages) do
    # Build a map of message -> new commit SHA
    new_commit_map =
      new_commits_with_messages
      |> Enum.map(fn {sha, message} -> {normalize_message(message), sha} end)
      |> Enum.into(%{})

    # Match and update each branch
    updates =
      stack
      |> Enum.reverse()
      |> Enum.flat_map(fn entry ->
        normalized_message = normalize_message(entry.message)

        case Map.get(new_commit_map, normalized_message) do
          nil ->
            # Commit not found - this is concerning but we'll report it
            Enum.map(entry.branches, fn branch ->
              {:unmatched, branch, entry.commit, entry.message}
            end)

          new_commit ->
            # Update all branches pointing to this commit
            Enum.map(entry.branches, fn branch ->
              case Client.update_ref(branch, new_commit) do
                {:ok, _} ->
                  {:ok, %{branch: branch, old_commit: entry.commit, new_commit: new_commit}}

                {:error, reason} ->
                  {:error, %{branch: branch, reason: reason}}
              end
            end)
        end
      end)

    # Separate successful updates from errors
    {successes, failures} =
      Enum.split_with(updates, fn
        {:ok, _} -> true
        _ -> false
      end)

    successful_updates = Enum.map(successes, fn {:ok, update} -> update end)

    unmatched =
      Enum.filter(failures, fn
        {:unmatched, _, _, _} -> true
        _ -> false
      end)

    errors =
      Enum.filter(failures, fn
        {:error, _} -> true
        _ -> false
      end)

    cond do
      not Enum.empty?(errors) ->
        {:error, {:update_failed, errors}}

      not Enum.empty?(unmatched) ->
        {:warning, {:unmatched_commits, unmatched, successful_updates}}

      true ->
        {:ok, successful_updates}
    end
  end

  # Normalizes commit messages for comparison (trims whitespace)
  defp normalize_message(message) do
    message
    |> String.trim()
    |> String.replace(~r/\s+/, " ")
  end

  @doc """
  Formats update results for display.
  """
  def format_updates(updates) do
    updates
    |> Enum.map(fn update ->
      old_short = String.slice(update.old_commit, 0..7)
      new_short = String.slice(update.new_commit, 0..7)
      "  #{update.branch}: #{old_short} → #{new_short}"
    end)
    |> Enum.join("\n")
  end
end
