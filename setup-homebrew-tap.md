# Setting Up Homebrew Tap for pomo-tui

This guide will help you set up the Homebrew tap to make `brew tap PatrickPriestley/tap && brew install pomo-tui` work.

## Steps to Complete

### 1. Create the Homebrew Tap Repository

You need to create a separate GitHub repository for the Homebrew tap:

```bash
# Go to GitHub.com and create a new repository named: homebrew-tap
# Make sure it's under your account: PatrickPriestley/homebrew-tap
# Initialize it as public with a README
```

### 2. Push the Tap Structure

```bash
# From your pomo-tui directory:
cd homebrew-tap
git init
git remote add origin https://github.com/PatrickPriestley/homebrew-tap.git
git add .
git commit -m "Initial homebrew tap setup for pomo-tui"
git branch -M main
git push -u origin main
```

### 3. Create GitHub Personal Access Token

1. Go to GitHub Settings → Developer Settings → Personal Access Tokens
2. Create a new token with these permissions:
   - `repo` (full repository access)
   - `workflow` (if you want to trigger workflows)
3. Copy the token value

### 4. Add Secret to Main Repository

1. Go to your main repo: https://github.com/PatrickPriestley/pomo-tui/settings/secrets/actions
2. Create a new secret named: `HOMEBREW_TAP_TOKEN`
3. Paste your personal access token as the value

### 5. Create the First Release

```bash
# From your pomo-tui directory:
git add .
git commit -m "Set up Homebrew tap infrastructure"
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

This will trigger the release workflow which will:
- Build binaries for multiple platforms
- Create a GitHub release
- Automatically update the Homebrew formula with the correct SHA256

### 6. Test the Installation

After the release workflow completes:

```bash
brew tap PatrickPriestley/tap
brew install pomo-tui
pomo-tui --version
```

## Troubleshooting

### If the tap repository doesn't exist:
- Make sure you created `PatrickPriestley/homebrew-tap` on GitHub
- Check that the repository is public
- Verify the repository name is exactly `homebrew-tap`

### If the formula fails to install:
- Check that the v0.1.0 release was created successfully
- Verify the SHA256 in the formula matches the release tarball
- Look at the GitHub Actions logs for any errors

### If the workflow fails to update the tap:
- Verify the `HOMEBREW_TAP_TOKEN` secret is set correctly
- Check that the token has `repo` permissions
- Make sure the homebrew-tap repository exists and is accessible

## File Structure After Setup

```
PatrickPriestley/pomo-tui/          # Main repository
├── homebrew/pomo-tui.rb           # Formula for reference
└── homebrew-tap/                  # Local copy of tap structure

PatrickPriestley/homebrew-tap/     # Separate tap repository
├── Formula/pomo-tui.rb           # The actual formula Homebrew uses
├── README.md                     # Tap documentation
└── .github/workflows/test.yml    # Formula testing
```

The tap repository will be automatically updated whenever you create a new release tag.