# pomo-tui

🍅 **ADHD-focused Pomodoro Terminal Application**

A terminal-based Pomodoro timer designed specifically for people with ADHD, featuring macOS Focus mode integration, task management, and a clean terminal interface.

## Features

- **🎯 ADHD-Friendly Design**: Gentle transitions and recovery features
- **🔕 Focus Mode Integration**: Automatic macOS Focus mode control during sessions
- **📋 Task Management**: Built-in task tracking and prioritization
- **📊 Statistics**: Track your productivity patterns and progress
- **🎨 Clean TUI**: Beautiful terminal interface built with ratatui
- **🏃‍♂️ Flexible Sessions**: Customizable work and break intervals
- **🔄 Auto-Enable Focus**: Automatically manage distractions during work sessions

## Installation

### Homebrew (Recommended)

```bash
brew tap PatrickPriestley/tap
brew install pomo-tui
```

**Upgrading:**
```bash
brew update
brew upgrade pomo-tui
```

> **Note:** If `brew upgrade` doesn't work, the tap may need to be updated. You can force an update with:
> ```bash
> brew untap PatrickPriestley/tap
> brew tap PatrickPriestley/tap
> brew upgrade pomo-tui
> ```

### From Source

```bash
git clone https://github.com/PatrickPriestley/pomo-tui
cd pomo-tui
cargo install --path .
```

### Binary Release

Download the latest binary from the [releases page](https://github.com/PatrickPriestley/pomo-tui/releases).

## Quick Start

```bash
# Start the TUI application
pomo-tui

# Show help
pomo-tui --help
```

### Basic Controls

- **Space** - Start/Pause timer
- **R** - Reset current session  
- **S** - Skip to break
- **D** - Manually toggle Focus mode
- **A** - Toggle auto-enable Focus mode
- **F** - Show Focus mode setup help
- **Q** - Quit application

## Focus Mode Setup (macOS)

To enable automatic Focus mode control:

1. Open the Shortcuts app
2. Create a shortcut named "Set Focus"
   - Add action: "Set Focus"
   - Choose your preferred Focus mode
   - Set duration: "Until I turn it off"
3. Create a shortcut named "Turn Off Focus"
   - Add action: "Turn Off Focus"

The app will automatically enable Focus mode during Pomodoro sessions and disable it during breaks.

## Configuration

pomo-tui stores its configuration and data in:
- **macOS**: `~/Library/Application Support/pomo-tui/`
- **Linux**: `~/.config/pomo-tui/`
- **Windows**: `%APPDATA%\\pomo-tui\\`

## ADHD-Focused Features

- **Gentle Transitions**: Smooth session changes without jarring interruptions
- **Pause-Friendly**: Focus mode automatically disables when you pause for interruptions
- **Recovery Support**: Designed to help you get back on track after breaks
- **Visual Indicators**: Clear status indicators for current session state

## Development

```bash
# Clone and build
git clone https://github.com/PatrickPriestley/pomo-tui
cd pomo-tui
cargo build

# Run tests
cargo test

# Install locally
cargo install --path .
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

---

**Built with Rust** 🦀 | **Powered by Ratatui** 📟 | **Designed for Focus** 🎯