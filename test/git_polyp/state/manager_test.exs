defmodule GitPolyp.State.ManagerTest do
  use ExUnit.Case

  import GitPolyp.GitTestHelper
  import GitPolyp.Assertions

  alias GitPolyp.State.{Manager, Metadata}

  setup do
    {:ok, repo_path} = create_test_repo("manager_test")

    on_exit(fn -> cleanup_test_repo(repo_path) end)

    # Change to repo directory for tests
    original_dir = File.cwd!()

    File.cd!(repo_path)

    on_exit(fn -> File.cd!(original_dir) end)

    %{repo: repo_path}
  end

  describe "metadata_path/0" do
    test "returns path in .git directory", %{repo: _repo} do
      path = Manager.metadata_path()

      assert is_binary(path)
      assert String.ends_with?(path, "rebase-stack-metadata")
    end

    test "path includes git directory", %{repo: _repo} do
      path = Manager.metadata_path()

      # Path should contain .git and end with the metadata filename
      assert String.contains?(path, ".git") or String.ends_with?(path, "rebase-stack-metadata")
    end
  end

  describe "save/1" do
    test "saves metadata to file", %{repo: _repo} do
      stack = [
        %{commit: "abc123", branches: ["feature-1"], message: "Test commit"}
      ]

      metadata = Metadata.new("main", "merge_base", "feature-1", stack, "current")

      assert :ok = Manager.save(metadata)

      # Verify file was created
      assert_file_exists(Manager.metadata_path())
    end

    test "creates valid JSON file", %{repo: _repo} do
      stack = [
        %{commit: "abc123", branches: ["feature-1"], message: "Test commit"}
      ]

      metadata = Metadata.new("main", "merge_base", "feature-1", stack, "current")

      Manager.save(metadata)

      # Read file and verify it's valid JSON
      {:ok, content} = File.read(Manager.metadata_path())
      assert {:ok, _} = Jason.decode(content)
    end

    test "overwrites existing file", %{repo: _repo} do
      stack1 = [
        %{commit: "abc", branches: ["br1"], message: "First"}
      ]

      stack2 = [
        %{commit: "def", branches: ["br2"], message: "Second"}
      ]

      metadata1 = Metadata.new("main", "base1", "target1", stack1, "current")
      metadata2 = Metadata.new("main", "base2", "target2", stack2, "current")

      Manager.save(metadata1)
      Manager.save(metadata2)

      # Load and verify it's the second one
      {:ok, loaded} = Manager.load()
      assert loaded.merge_base == "base2"
      assert loaded.target_branch == "target2"
    end

    test "saves complete metadata structure", %{repo: _repo} do
      stack = [
        %{commit: "abc", branches: ["feat-1", "feat-2"], message: "Multi branch"},
        %{commit: "def", branches: [], message: "No branches"},
        %{commit: "ghi", branches: ["feat-3"], message: "Last commit"}
      ]

      metadata = Metadata.new("main", "merge123", "feature-3", stack, "original-branch")

      Manager.save(metadata)

      {:ok, loaded} = Manager.load()

      assert loaded.base_branch == "main"
      assert loaded.merge_base == "merge123"
      assert loaded.target_branch == "feature-3"
      assert loaded.original_branch == "original-branch"
      assert length(loaded.stack) == 3
    end
  end

  describe "load/0" do
    test "loads saved metadata", %{repo: _repo} do
      stack = [
        %{commit: "abc123", branches: ["feature-1"], message: "Test commit"}
      ]

      original = Metadata.new("main", "merge_base", "feature-1", stack, "current")

      Manager.save(original)

      {:ok, loaded} = Manager.load()

      assert loaded.base_branch == original.base_branch
      assert loaded.merge_base == original.merge_base
      assert loaded.target_branch == original.target_branch
      assert loaded.original_branch == original.original_branch
    end

    test "returns error when file doesn't exist", %{repo: _repo} do
      # Ensure no file exists
      Manager.delete()

      assert {:error, :not_found} = Manager.load()
    end

    test "returns error for corrupted file", %{repo: _repo} do
      # Write invalid JSON
      File.write!(Manager.metadata_path(), "{ invalid json }")

      assert {:error, error_msg} = Manager.load()
      assert error_msg =~ "Invalid JSON"
    end

    test "loads metadata with all stack details", %{repo: _repo} do
      stack = [
        %{commit: "abc", branches: ["br1", "br2"], message: "Message 1"},
        %{commit: "def", branches: [], message: "Message 2"}
      ]

      original = Metadata.new("main", "base", "target", stack, "orig")

      Manager.save(original)

      {:ok, loaded} = Manager.load()

      assert length(loaded.stack) == 2
      assert Enum.at(loaded.stack, 0).commit == "abc"
      assert Enum.at(loaded.stack, 0).branches == ["br1", "br2"]
      assert Enum.at(loaded.stack, 0).message == "Message 1"
    end

    test "preserves timestamp", %{repo: _repo} do
      stack = []
      original = Metadata.new("main", "base", "target", stack, "orig")

      Manager.save(original)

      {:ok, loaded} = Manager.load()

      assert loaded.timestamp == original.timestamp
    end
  end

  describe "exists?/0" do
    test "returns false when no metadata file exists", %{repo: _repo} do
      Manager.delete()

      assert Manager.exists?() == false
    end

    test "returns true when metadata file exists", %{repo: _repo} do
      stack = [%{commit: "abc", branches: ["br"], message: "msg"}]
      metadata = Metadata.new("main", "base", "target", stack, "orig")

      Manager.save(metadata)

      assert Manager.exists?() == true
    end

    test "returns false after deleting metadata", %{repo: _repo} do
      stack = [%{commit: "abc", branches: ["br"], message: "msg"}]
      metadata = Metadata.new("main", "base", "target", stack, "orig")

      Manager.save(metadata)
      assert Manager.exists?() == true

      Manager.delete()
      assert Manager.exists?() == false
    end
  end

  describe "delete/0" do
    test "deletes existing metadata file", %{repo: _repo} do
      stack = [%{commit: "abc", branches: ["br"], message: "msg"}]
      metadata = Metadata.new("main", "base", "target", stack, "orig")

      Manager.save(metadata)
      assert_file_exists(Manager.metadata_path())

      assert :ok = Manager.delete()

      refute_file_exists(Manager.metadata_path())
    end

    test "returns ok when file doesn't exist", %{repo: _repo} do
      # Ensure it doesn't exist
      Manager.delete()

      assert :ok = Manager.delete()
    end

    test "can save again after delete", %{repo: _repo} do
      stack = [%{commit: "abc", branches: ["br"], message: "msg"}]
      metadata = Metadata.new("main", "base", "target", stack, "orig")

      Manager.save(metadata)
      Manager.delete()

      # Should be able to save again
      assert :ok = Manager.save(metadata)
      assert Manager.exists?() == true
    end
  end

  describe "roundtrip save/load" do
    test "preserves all data through save/load cycle", %{repo: _repo} do
      stack = [
        %{
          commit: "abc123def456",
          branches: ["feature-1", "feature-2", "hotfix"],
          message: "Complex commit message\nwith multiple lines\nand special chars: 你好"
        },
        %{
          commit: "ghi789jkl012",
          branches: [],
          message: "Simple message"
        }
      ]

      original = Metadata.new("main", "merge_base_sha", "feature-final", stack, "my-branch")

      Manager.save(original)
      {:ok, loaded} = Manager.load()

      assert loaded.base_branch == original.base_branch
      assert loaded.merge_base == original.merge_base
      assert loaded.target_branch == original.target_branch
      assert loaded.original_branch == original.original_branch
      assert loaded.timestamp == original.timestamp
      assert length(loaded.stack) == length(original.stack)

      # Check first entry
      loaded_first = Enum.at(loaded.stack, 0)
      original_first = Enum.at(original.stack, 0)

      assert loaded_first.commit == original_first.commit
      assert loaded_first.branches == original_first.branches
      assert loaded_first.message == original_first.message

      # Check second entry
      loaded_second = Enum.at(loaded.stack, 1)
      original_second = Enum.at(original.stack, 1)

      assert loaded_second.commit == original_second.commit
      assert loaded_second.branches == original_second.branches
      assert loaded_second.message == original_second.message
    end
  end
end
