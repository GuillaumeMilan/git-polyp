defmodule GitPolyp.Commands.InstallCompletions do
  @moduledoc """
  Implements the install command.

  Installs shell completion scripts for bash, zsh, or fish.
  """

  alias GitPolyp.UI.Formatter

  @doc """
  Runs the install command.

  Accepts arguments:
  - [] - Auto-detect shell and install
  - [shell] - Install for specified shell (bash, zsh, or fish)
  """
  def run(args) do
    # Parse options with OptionParser
    {opts, positional, invalid} =
      OptionParser.parse(args,
        strict: [
          help: :boolean
        ],
        aliases: [
          h: :help
        ]
      )

    cond do
      opts[:help] ->
        print_help()
        :ok

      not Enum.empty?(invalid) ->
        invalid_opts = Enum.map(invalid, fn {opt, _} -> opt end) |> Enum.join(", ")

        {:error,
         "Unknown option: #{invalid_opts}\nRun 'git-polyp install --help' for usage information."}

      length(positional) > 1 ->
        {:error,
         "Too many arguments. Expected: [shell]\nRun 'git-polyp install --help' for usage information."}

      length(positional) == 1 ->
        [shell] = positional
        install_for_shell(shell)

      true ->
        # Auto-detect shell
        install_auto_detect()
    end
  end

  defp install_auto_detect do
    case detect_shell() do
      {:ok, shell} ->
        IO.puts(Formatter.info("Detected shell: #{shell}"))
        IO.puts("")
        install_for_shell(shell)

      {:error, :unknown} ->
        {:error, "Could not detect shell. Please specify: bash, zsh, or fish"}
    end
  end

  defp install_for_shell(shell) when shell in ["bash", "zsh", "fish"] do
    IO.puts(Formatter.info("Installing completion for: #{shell}"))
    IO.puts("")

    case shell do
      "bash" -> install_bash_completion()
      "zsh" -> install_zsh_completion()
      "fish" -> install_fish_completion()
    end
  end

  defp install_for_shell(shell) do
    {:error, "Unknown shell: #{shell}\nSupported shells: bash, zsh, fish"}
  end

  # Shell detection
  defp detect_shell do
    # First try SHELL environment variable
    case System.get_env("SHELL") do
      nil -> detect_shell_from_parent()
      shell_path -> parse_shell_from_path(shell_path)
    end
  end

  defp detect_shell_from_parent do
    # Fallback: try to detect from parent process
    case System.cmd("ps", ["-o", "comm=", "-p", "#{System.pid()}"], stderr_to_stdout: true) do
      {output, 0} ->
        parent_cmd = String.trim(output)
        parse_shell_from_path(parent_cmd)

      _ ->
        {:error, :unknown}
    end
  rescue
    _ -> {:error, :unknown}
  end

  defp parse_shell_from_path(path) do
    cond do
      String.contains?(path, "bash") -> {:ok, "bash"}
      String.contains?(path, "zsh") -> {:ok, "zsh"}
      String.contains?(path, "fish") -> {:ok, "fish"}
      true -> {:error, :unknown}
    end
  end

  # Bash installation
  defp install_bash_completion do
    completion_source = get_completion_file_path("git-polyp.bash")

    install_dirs = [
      "/usr/local/etc/bash_completion.d",
      "/etc/bash_completion.d",
      Path.expand("~/.local/share/bash-completion/completions")
    ]

    case try_install_to_directories(completion_source, "git-polyp", install_dirs) do
      {:ok, installed_path} ->
        print_success("Installed to #{installed_path}")
        IO.puts("")
        IO.puts(Formatter.info("Restart your shell or run:"))
        IO.puts(Formatter.command("  source #{installed_path}"))
        IO.puts("")
        :ok

      {:error, :no_writable_dir} ->
        print_manual_install_instructions("bash", completion_source)
        :ok
    end
  end

  # Zsh installation
  defp install_zsh_completion do
    completion_source = get_completion_file_path("git-polyp.zsh")

    install_dirs = [
      "/usr/local/share/zsh/site-functions",
      Path.expand("~/.zsh/completions")
    ]

    # Try user directory first (create if needed)
    user_dir = Path.expand("~/.zsh/completions")
    File.mkdir_p(user_dir)

    case File.cp(completion_source, Path.join(user_dir, "_git-polyp")) do
      :ok ->
        print_success("Installed to #{user_dir}/_git-polyp")
        IO.puts("")
        configure_zsh_fpath(user_dir)
        :ok

      {:error, _reason} ->
        # Try system directory
        case try_install_to_directories(completion_source, "_git-polyp", install_dirs) do
          {:ok, installed_path} ->
            print_success("Installed to #{installed_path}")
            IO.puts("")
            IO.puts(Formatter.info("Restart your shell or run:"))
            IO.puts(Formatter.command("  compinit"))
            IO.puts("")
            :ok

          {:error, :no_writable_dir} ->
            print_manual_install_instructions("zsh", completion_source)
            :ok
        end
    end
  end

  defp configure_zsh_fpath(_completions_dir) do
    zshrc = Path.expand("~/.zshrc")

    if File.exists?(zshrc) do
      content = File.read!(zshrc)

      if String.contains?(content, ".zsh/completions") do
        IO.puts(Formatter.info("fpath already configured in ~/.zshrc"))
        IO.puts(Formatter.info("Restart your shell or run:"))
        IO.puts(Formatter.command("  exec zsh"))
        IO.puts("")
      else
        # Add fpath configuration
        case add_fpath_to_zshrc(zshrc, content) do
          :ok ->
            print_success("Added fpath configuration to ~/.zshrc")
            IO.puts("")
            IO.puts(Formatter.info("Restart your shell to activate completions"))
            IO.puts("")

          {:error, reason} ->
            print_warning("Could not update ~/.zshrc: #{reason}")
            print_manual_zshrc_instructions()
        end
      end
    else
      print_warning("~/.zshrc not found")
      print_manual_zshrc_instructions()
    end
  end

  defp add_fpath_to_zshrc(zshrc, content) do
    fpath_config = """
    # Custom completions
    fpath=(~/.zsh/completions $fpath)
    autoload -Uz compinit && compinit

    """

    new_content =
      if String.contains?(content, "oh-my-zsh.sh") do
        # Insert before oh-my-zsh is sourced
        String.replace(
          content,
          ~r/(.*source.*oh-my-zsh\.sh)/,
          "# Custom completions (must be before oh-my-zsh is sourced)\nfpath=(~/.zsh/completions $fpath)\n\n\\1"
        )
      else
        # Add at the beginning
        fpath_config <> content
      end

    File.write(zshrc, new_content)
  end

  defp print_manual_zshrc_instructions do
    IO.puts("")
    IO.puts(Formatter.info("Add this to your ~/.zshrc:"))
    IO.puts("")
    IO.puts("  fpath=(~/.zsh/completions $fpath)")
    IO.puts("  autoload -Uz compinit && compinit")
    IO.puts("")
  end

  # Fish installation
  defp install_fish_completion do
    completion_source = get_completion_file_path("git-polyp.fish")
    install_dir = Path.expand("~/.config/fish/completions")

    # Create directory if it doesn't exist
    File.mkdir_p(install_dir)

    dest_path = Path.join(install_dir, "git-polyp.fish")

    case File.cp(completion_source, dest_path) do
      :ok ->
        print_success("Installed to #{dest_path}")
        IO.puts("")
        IO.puts(Formatter.info("Completion will be available in new Fish shell sessions"))
        IO.puts("")
        :ok

      {:error, reason} ->
        {:error, "Failed to install Fish completion: #{reason}"}
    end
  end

  # Helper functions
  defp get_completion_file_path(filename) do
    # Get the path to the completion file relative to the escript
    # When running as escript, use the directory where git-polyp is installed
    # When running in dev, use the project's completions directory

    cond do
      # Development: use project's completions directory
      File.dir?("completions") ->
        Path.join("completions", filename)

      # Try to find completions next to the executable
      true ->
        executable_path = System.argv() |> List.first() || "git-polyp"
        executable_dir = Path.dirname(executable_path)
        completion_path = Path.join([executable_dir, "completions", filename])

        if File.exists?(completion_path) do
          completion_path
        else
          # Fallback: assume it's in a standard location
          "/usr/local/share/git-polyp/completions/#{filename}"
        end
    end
  end

  defp try_install_to_directories(_source, _filename, []) do
    {:error, :no_writable_dir}
  end

  defp try_install_to_directories(source, filename, [dir | rest]) do
    if File.dir?(dir) do
      dest_path = Path.join(dir, filename)

      case File.cp(source, dest_path) do
        :ok ->
          {:ok, dest_path}

        {:error, :eacces} ->
          # No write permission, try next directory
          try_install_to_directories(source, filename, rest)

        {:error, _reason} ->
          # Other error, try next directory
          try_install_to_directories(source, filename, rest)
      end
    else
      # Directory doesn't exist, try next
      try_install_to_directories(source, filename, rest)
    end
  end

  defp print_manual_install_instructions(shell, completion_source) do
    print_warning("Could not find writable #{shell} completion directory")
    IO.puts("")

    IO.puts(
      Formatter.info("Manually add this line to your ~/.#{shell}rc or ~/.#{shell}_profile:")
    )

    IO.puts("")
    IO.puts("  source #{completion_source}")
    IO.puts("")
  end

  defp print_success(message) do
    IO.puts(Formatter.success("✓ #{message}"))
  end

  defp print_warning(message) do
    IO.puts(Formatter.warning("⚠ #{message}"))
  end

  defp print_help do
    IO.puts("""
    #{Formatter.header("git-polyp install")} - Install shell completions

    #{Formatter.header("USAGE:")}
      git-polyp install [shell]

    #{Formatter.header("ARGUMENTS:")}
      [shell]    Shell type: bash, zsh, or fish (auto-detected if not specified)

    #{Formatter.header("OPTIONS:")}
      -h, --help    Show this help message

    #{Formatter.header("DESCRIPTION:")}
      Installs shell completion scripts for git-polyp commands.

      The command will:
      1. Detect your shell (or use the specified shell)
      2. Copy the completion script to the appropriate location
      3. Configure your shell to load the completions

      Supported shells: bash, zsh, fish

    #{Formatter.header("EXAMPLES:")}
      # Auto-detect shell and install
      git-polyp install

      # Install for a specific shell
      git-polyp install bash
      git-polyp install zsh
      git-polyp install fish
    """)
  end
end
