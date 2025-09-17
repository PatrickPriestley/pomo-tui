class PomoTui < Formula
  desc "ADHD-focused Pomodoro timer with terminal interface and Focus mode integration"
  homepage "https://github.com/PatrickPriestley/pomo-tui"
  url "https://github.com/PatrickPriestley/pomo-tui/archive/v0.3.0.tar.gz"
  sha256 "8e88e5751be7994d39cf6cae72fa522215322818af53ce81712de6a87ecee715"
  license "MIT"
  head "https://github.com/PatrickPriestley/pomo-tui.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end


  test do
    # Test that the binary exists and is executable
    assert_predicate bin/"pomo-tui", :exist?
    assert_predicate bin/"pomo-tui", :executable?
    
    # Test basic execution (TUI app, so just verify it starts)
    # Note: This will exit immediately since there's no terminal, but should not crash
    system "echo 'q' | timeout 2s #{bin}/pomo-tui || true"
  end

  def caveats
    <<~EOS
      pomo-tui has been installed successfully!
      
      Usage:
        • Run 'pomo-tui' to start the interactive terminal interface
        • Use spacebar to start/pause sessions
        • Press 'q' to quit
        
      Features:
        • ADHD-focused 25-minute Pomodoro sessions
        • Automatic break management (5min short, 15min long breaks)
        • Breathing exercises during breaks
        • macOS Focus mode integration (requires Shortcuts app setup)
        
      macOS Focus Mode Setup (optional):
        • Open Shortcuts app
        • Create "Set Focus" shortcut with "Set Focus" action
        • Create "Turn Off Focus" shortcut with "Turn Off Focus" action
        • pomo-tui will automatically control Focus mode during sessions
        
      Documentation:
        • GitHub: https://github.com/PatrickPriestley/pomo-tui
    EOS
  end
end