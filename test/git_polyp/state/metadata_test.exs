defmodule GitPolyp.State.MetadataTest do
  use ExUnit.Case, async: true

  alias GitPolyp.State.Metadata

  @sample_stack [
    %{commit: "abc123", branches: ["feature-1"], message: "First commit"},
    %{commit: "def456", branches: ["feature-2"], message: "Second commit"},
    %{commit: "ghi789", branches: ["feature-3"], message: "Third commit"}
  ]

  describe "new/5" do
    test "creates metadata struct with all required fields" do
      metadata =
        Metadata.new("main", "merge_base_sha", "feature-3", @sample_stack, "current-branch")

      assert metadata.base_branch == "main"
      assert metadata.merge_base == "merge_base_sha"
      assert metadata.target_branch == "feature-3"
      assert metadata.stack == @sample_stack
      assert metadata.original_branch == "current-branch"
      assert metadata.timestamp != nil
    end

    test "generates ISO8601 timestamp" do
      metadata = Metadata.new("main", "abc", "feature", [], "current")

      # Verify timestamp is in ISO8601 format
      assert is_binary(metadata.timestamp)
      assert String.contains?(metadata.timestamp, "T")
      assert String.contains?(metadata.timestamp, "Z")
    end

    test "timestamp is current time" do
      before = DateTime.utc_now()
      metadata = Metadata.new("main", "abc", "feature", [], "current")
      after_time = DateTime.utc_now()

      {:ok, timestamp, 0} = DateTime.from_iso8601(metadata.timestamp)

      assert DateTime.compare(timestamp, before) in [:gt, :eq]
      assert DateTime.compare(timestamp, after_time) in [:lt, :eq]
    end
  end

  describe "encode/1" do
    test "encodes metadata to JSON string" do
      metadata = Metadata.new("main", "abc123", "feature", @sample_stack, "current")

      assert {:ok, json} = Metadata.encode(metadata)
      assert is_binary(json)
    end

    test "base64 encodes commit messages" do
      metadata = Metadata.new("main", "abc123", "feature", @sample_stack, "current")

      {:ok, json} = Metadata.encode(metadata)
      {:ok, decoded} = Jason.decode(json)

      # Check that messages are base64 encoded
      stack = decoded["stack"]
      first_entry = List.first(stack)

      assert first_entry["message"] != "First commit"
      # Verify it's valid base64
      assert {:ok, "First commit"} = Base.decode64(first_entry["message"])
    end

    test "encodes all stack entries" do
      metadata = Metadata.new("main", "abc123", "feature", @sample_stack, "current")

      {:ok, json} = Metadata.encode(metadata)
      {:ok, decoded} = Jason.decode(json)

      assert length(decoded["stack"]) == 3
    end

    test "handles special characters in commit messages" do
      special_stack = [
        %{commit: "abc", branches: ["br"], message: "Special chars: \n\t\"quotes\" 'apostrophes'"}
      ]

      metadata = Metadata.new("main", "abc", "feature", special_stack, "current")

      {:ok, json} = Metadata.encode(metadata)
      assert is_binary(json)
    end

    test "encodes unicode characters correctly" do
      unicode_stack = [
        %{commit: "abc", branches: ["br"], message: "Unicode: 你好世界 🎉 café"}
      ]

      metadata = Metadata.new("main", "abc", "feature", unicode_stack, "current")

      {:ok, json} = Metadata.encode(metadata)
      {:ok, decoded_json} = Jason.decode(json)

      encoded_msg = List.first(decoded_json["stack"])["message"]
      {:ok, decoded_msg} = Base.decode64(encoded_msg)

      assert decoded_msg == "Unicode: 你好世界 🎉 café"
    end
  end

  describe "decode/1" do
    test "decodes valid JSON to metadata struct" do
      metadata = Metadata.new("main", "abc123", "feature-3", @sample_stack, "current")
      {:ok, json} = Metadata.encode(metadata)

      assert {:ok, decoded} = Metadata.decode(json)
      assert %Metadata{} = decoded
    end

    test "roundtrip encode/decode preserves data" do
      original = Metadata.new("main", "abc123", "feature-3", @sample_stack, "original-branch")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      assert decoded.base_branch == original.base_branch
      assert decoded.merge_base == original.merge_base
      assert decoded.target_branch == original.target_branch
      assert decoded.original_branch == original.original_branch
      assert length(decoded.stack) == length(original.stack)
    end

    test "decodes base64 commit messages" do
      original = Metadata.new("main", "abc123", "feature", @sample_stack, "current")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      # Verify messages are decoded correctly
      assert Enum.at(decoded.stack, 0).message == "First commit"
      assert Enum.at(decoded.stack, 1).message == "Second commit"
      assert Enum.at(decoded.stack, 2).message == "Third commit"
    end

    test "preserves all stack entry fields" do
      original = Metadata.new("main", "abc123", "feature", @sample_stack, "current")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      first_entry = List.first(decoded.stack)
      assert first_entry.commit == "abc123"
      assert first_entry.branches == ["feature-1"]
      assert first_entry.message == "First commit"
    end

    test "returns error for invalid JSON" do
      invalid_json = "{ invalid json }"

      assert {:error, error_msg} = Metadata.decode(invalid_json)
      assert error_msg =~ "Invalid JSON"
    end

    test "returns error for missing required fields" do
      # JSON missing target_branch
      incomplete_json =
        Jason.encode!(%{
          base_branch: "main",
          merge_base: "abc123",
          stack: [],
          original_branch: "current"
        })

      assert {:error, error_msg} = Metadata.decode(incomplete_json)
      assert error_msg =~ "Missing required fields"
      assert error_msg =~ "target_branch"
    end

    test "returns error when required field is null" do
      json_with_null =
        Jason.encode!(%{
          base_branch: "main",
          merge_base: nil,
          target_branch: "feature",
          stack: [],
          original_branch: "current"
        })

      assert {:error, error_msg} = Metadata.decode(json_with_null)
      assert error_msg =~ "Missing required fields"
      assert error_msg =~ "merge_base"
    end

    test "handles special characters in roundtrip" do
      special_stack = [
        %{commit: "abc", branches: ["br"], message: "Message with\nnewlines\tand\ttabs"}
      ]

      original = Metadata.new("main", "abc", "feature", special_stack, "current")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      assert List.first(decoded.stack).message == "Message with\nnewlines\tand\ttabs"
    end

    test "preserves timestamp" do
      original = Metadata.new("main", "abc123", "feature", [], "current")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      assert decoded.timestamp == original.timestamp
    end

    test "handles empty stack" do
      original = Metadata.new("main", "abc123", "feature", [], "current")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      assert decoded.stack == []
    end

    test "handles multiple branches per commit" do
      multi_branch_stack = [
        %{
          commit: "abc",
          branches: ["feature-1", "feature-2", "feature-3"],
          message: "Multi-branch"
        }
      ]

      original = Metadata.new("main", "abc", "feature", multi_branch_stack, "current")
      {:ok, json} = Metadata.encode(original)
      {:ok, decoded} = Metadata.decode(json)

      first_entry = List.first(decoded.stack)
      assert length(first_entry.branches) == 3
      assert first_entry.branches == ["feature-1", "feature-2", "feature-3"]
    end

    test "handles malformed base64 gracefully" do
      # Manually create JSON with invalid base64
      malformed_json =
        Jason.encode!(%{
          base_branch: "main",
          merge_base: "abc",
          target_branch: "feature",
          stack: [
            %{commit: "abc", branches: ["br"], message: "not-valid-base64!!!"}
          ],
          original_branch: "current",
          timestamp: "2024-01-01T00:00:00Z"
        })

      # Should still decode, but message might not be decoded properly
      assert {:ok, decoded} = Metadata.decode(malformed_json)
      # When base64 decode fails, it should keep the original string
      assert List.first(decoded.stack).message == "not-valid-base64!!!"
    end
  end
end
