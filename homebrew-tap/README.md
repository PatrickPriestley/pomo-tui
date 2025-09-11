# Homebrew Tap for pomo-tui

This is the official Homebrew tap for [pomo-tui](https://github.com/PatrickPriestley/pomo-tui), an ADHD-focused Pomodoro terminal application.

## Installation

```bash
brew tap PatrickPriestley/tap
brew install pomo-tui
```

## Available Formulas

- **pomo-tui** - ADHD-focused Pomodoro terminal application with task management

## Development

This tap is automatically updated when new releases are published to the main pomo-tui repository.

## Manual Formula Updates

If you need to manually update the formula:

1. Update `Formula/pomo-tui.rb` with the new version and SHA256
2. Commit and push the changes
3. The updated formula will be available via `brew upgrade pomo-tui`

## Issues

Please report issues with:
- **Installation/Formula**: Open an issue in this repository
- **Application bugs**: Open an issue in the [main pomo-tui repository](https://github.com/PatrickPriestley/pomo-tui)