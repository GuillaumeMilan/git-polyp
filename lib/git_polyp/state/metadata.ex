defmodule GitPolyp.State.Metadata do
  @moduledoc """
  Defines the metadata structure for tracking rebase-stack operations.

  The metadata is persisted to disk to allow resuming operations after
  conflict resolution.
  """

  @derive Jason.Encoder
  defstruct [
    :base_branch,
    :merge_base,
    :target_branch,
    :stack,
    :original_branch,
    :timestamp
  ]

  @type stack_entry :: %{
          commit: String.t(),
          branches: [String.t()],
          message: String.t()
        }

  @type t :: %__MODULE__{
          base_branch: String.t(),
          merge_base: String.t(),
          target_branch: String.t(),
          stack: [stack_entry()],
          original_branch: String.t(),
          timestamp: String.t()
        }

  @doc """
  Creates a new metadata struct.

  ## Examples
      iex> GitPolyp.State.Metadata.new("main", "abc123", "feature", stack, "current")
      %GitPolyp.State.Metadata{...}
  """
  def new(base_branch, merge_base, target_branch, stack, original_branch) do
    %__MODULE__{
      base_branch: base_branch,
      merge_base: merge_base,
      target_branch: target_branch,
      stack: stack,
      original_branch: original_branch,
      timestamp: DateTime.utc_now() |> DateTime.to_iso8601()
    }
  end

  @doc """
  Encodes metadata to JSON string.

  Uses base64 encoding for commit messages to handle special characters.
  """
  def encode(metadata) do
    # Base64 encode commit messages for safety
    encoded_stack =
      Enum.map(metadata.stack, fn entry ->
        %{
          entry
          | message: Base.encode64(entry.message)
        }
      end)

    encoded_metadata = %{metadata | stack: encoded_stack}
    Jason.encode(encoded_metadata)
  end

  @doc """
  Decodes metadata from JSON string.

  Decodes base64-encoded commit messages back to original strings.
  """
  def decode(json_string) do
    with {:ok, data} <- Jason.decode(json_string, keys: :atoms),
         {:ok, metadata} <- validate_and_build(data) do
      # Base64 decode commit messages
      decoded_stack =
        Enum.map(metadata.stack, fn entry ->
          case Base.decode64(entry.message) do
            {:ok, decoded} -> %{entry | message: decoded}
            :error -> entry
          end
        end)

      {:ok, %{metadata | stack: decoded_stack}}
    else
      {:error, %Jason.DecodeError{} = error} ->
        {:error, "Invalid JSON: #{Exception.message(error)}"}

      {:error, reason} ->
        {:error, reason}
    end
  end

  # Validates decoded data and builds metadata struct
  defp validate_and_build(data) do
    required_fields = [:base_branch, :merge_base, :target_branch, :stack, :original_branch]

    missing_fields =
      Enum.filter(required_fields, fn field ->
        not Map.has_key?(data, field) or is_nil(Map.get(data, field))
      end)

    if missing_fields != [] do
      {:error, "Missing required fields: #{inspect(missing_fields)}"}
    else
      stack =
        Enum.map(data.stack, fn entry ->
          %{
            commit: entry.commit,
            branches: entry.branches,
            message: entry.message
          }
        end)

      metadata = %__MODULE__{
        base_branch: data.base_branch,
        merge_base: data.merge_base,
        target_branch: data.target_branch,
        stack: stack,
        original_branch: data.original_branch,
        timestamp: Map.get(data, :timestamp)
      }

      {:ok, metadata}
    end
  end
end
