class PomoTui < Formula
  desc "ADHD-focused Pomodoro terminal application with task management"
  homepage "https://github.com/PatrickPriestley/pomo-tui"
  url "https://github.com/PatrickPriestley/pomo-tui/archive/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256" # This will be updated automatically by release process
  license "MIT"
  head "https://github.com/PatrickPriestley/pomo-tui.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    
    # Install shell completions
    bash_completion.install "completions/pomo-tui.bash" => "pomo-tui"
    zsh_completion.install "completions/_pomo-tui"
    fish_completion.install "completions/pomo-tui.fish"
    
    # Install man page if it exists
    if (buildpath/"man/pomo-tui.1").exist?
      man1.install "man/pomo-tui.1"
    end
    
    # Install configuration examples
    if (buildpath/"config").exist?
      pkgshare.install "config"
    end
  end

  def post_install
    # Create default configuration directory
    config_dir = etc/"pomo-tui"
    config_dir.mkpath
    
    # Copy default config if it doesn't exist
    default_config = config_dir/"config.toml"
    unless default_config.exist?
      if (pkgshare/"config/default.toml").exist?
        cp pkgshare/"config/default.toml", default_config
      end
    end
    
    ohai "Configuration directory created at #{config_dir}"
    ohai "Default config copied to #{default_config}" if default_config.exist?
    ohai "Run 'pomo-tui --help' to get started!"
  end

  test do
    # Test basic functionality
    assert_match "pomo-tui", shell_output("#{bin}/pomo-tui --version")
    assert_match "ADHD-focused Pomodoro", shell_output("#{bin}/pomo-tui --help")
    
    # Test CLI commands work
    output = shell_output("#{bin}/pomo-tui task list 2>&1", 0)
    assert_match(/tasks|Tasks/, output)
    
    # Test configuration loading
    system "#{bin}/pomo-tui", "preferences", "--help"
    
    # Test that the binary is properly built and linked
    system "#{bin}/pomo-tui", "--version"
  end

  def caveats
    <<~EOS
      pomo-tui has been installed successfully!
      
      Configuration:
        • Default config: #{etc}/pomo-tui/config.toml
        • Create tasks: pomo-tui task new "Task name"
        • Start session: pomo-tui session start
        • View statistics: pomo-tui stats
      
      Integration setup (optional):
        • GitHub: pomo-tui integration github setup
        • Slack: pomo-tui integration slack setup
        
      Audio requirements:
        • macOS: Built-in audio support
        • Linux: Requires ALSA/PulseAudio
        
      Documentation:
        • Run: pomo-tui --help
        • Manual: man pomo-tui (if available)
        • GitHub: https://github.com/PatrickPriestley/pomo-tui
    EOS
  end
end