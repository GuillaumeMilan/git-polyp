defmodule GitPolyp.State.Manager do
  @moduledoc """
  Manages persistence of rebase-stack operation state.

  State is stored in `.git/rebase-stack-metadata` to track progress
  and allow resuming operations after conflict resolution.
  """

  alias GitPolyp.Git.Client
  alias GitPolyp.State.Metadata

  @metadata_filename "rebase-stack-metadata"

  @doc """
  Saves metadata to the .git directory.

  ## Examples
      iex> GitPolyp.State.Manager.save(metadata)
      :ok
  """
  def save(%Metadata{} = metadata) do
    case Metadata.encode(metadata) do
      {:ok, json} ->
        metadata_path()
        |> File.write(json)

      {:error, error} ->
        {:error, "Failed to encode metadata: #{inspect(error)}"}
    end
  end

  @doc """
  Loads metadata from the .git directory.

  ## Examples
      iex> GitPolyp.State.Manager.load()
      {:ok, %Metadata{}}
  """
  def load do
    path = metadata_path()

    if File.exists?(path) do
      case File.read(path) do
        {:ok, json} ->
          Metadata.decode(json)

        {:error, reason} ->
          {:error, "Failed to read metadata file: #{inspect(reason)}"}
      end
    else
      {:error, :not_found}
    end
  end

  @doc """
  Checks if a rebase-stack operation is in progress.

  ## Examples
      iex> GitPolyp.State.Manager.exists?()
      true
  """
  def exists? do
    File.exists?(metadata_path())
  end

  @doc """
  Deletes the metadata file, cleaning up after an operation completes or is aborted.

  ## Examples
      iex> GitPolyp.State.Manager.delete()
      :ok
  """
  def delete do
    path = metadata_path()

    if File.exists?(path) do
      File.rm(path)
    else
      :ok
    end
  end

  @doc """
  Gets the full path to the metadata file.
  """
  def metadata_path do
    git_dir = Client.git_dir()
    Path.join(git_dir, @metadata_filename)
  end
end
