<#
.SYNOPSIS
    Automatic Git commit and push script for StoryStream
#>

param(
  [Parameter(Mandatory=$false)]
  [ValidateSet('once', 'watch')]
  [string]$Mode = 'once',

  [Parameter(Mandatory=$false)]
  [string]$Message = "",

  [Parameter(Mandatory=$false)]
  [int]$WatchInterval = 30,

  [Parameter(Mandatory=$false)]
  [switch]$Force,

  [Parameter(Mandatory=$false)]
  [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Write-Success { param([string]$msg) Write-Host "✓ $msg" -ForegroundColor Green }
function Write-Fail { param([string]$msg) Write-Host "✗ $msg" -ForegroundColor Red }
function Write-Info { param([string]$msg) Write-Host "ℹ $msg" -ForegroundColor Cyan }
function Write-Warn { param([string]$msg) Write-Host "⚠ $msg" -ForegroundColor Yellow }

function Test-GitRepo {
  try {
    git rev-parse --git-dir 2>&1 | Out-Null
    return $true
  } catch {
    return $false
  }
}

function Test-GitChanges {
  $status = git status --porcelain
  return ($null -ne $status -and $status.Length -gt 0)
}

function Get-CurrentBranch {
  try {
    $branch = git rev-parse --abbrev-ref HEAD 2>&1
    if ($LASTEXITCODE -eq 0) { return $branch.Trim() }
    return "main"
  } catch {
    return "main"
  }
}

function Get-SmartCommitMessage {
  $status = git status --short

  $added = ($status | Select-String "^A " | Measure-Object).Count
  $modified = ($status | Select-String "^M " | Measure-Object).Count
  $deleted = ($status | Select-String "^D " | Measure-Object).Count

  $changedCrates = @()
  $status | ForEach-Object {
    if ($_ -match "crates/([^/]+)/") {
      $crate = $matches[1]
      if ($changedCrates -notcontains $crate) {
        $changedCrates += $crate
      }
    }
  }

  $parts = @()
  if ($added -gt 0) { $parts += "$added added" }
  if ($modified -gt 0) { $parts += "$modified modified" }
  if ($deleted -gt 0) { $parts += "$deleted deleted" }

  $changeDesc = $parts -join ", "

  if ($changedCrates.Count -gt 0) {
    $crateList = ($changedCrates | Select-Object -First 3) -join ", "
    if ($changedCrates.Count -gt 3) {
      $crateList += ", +$($changedCrates.Count - 3) more"
    }
    return "chore: Update $crateList ($changeDesc)"
  }

  $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
  return "chore: Auto-commit - $changeDesc [$timestamp]"
}

function Invoke-Tests {
  if ($SkipTests) {
    Write-Warn "Skipping tests"
    return $true
  }

  Write-Info "Running tests..."
  Push-Location $RepoRoot

  try {
    $output = cargo test 2>&1 | Out-String

    if ($LASTEXITCODE -eq 0) {
      Write-Success "All tests passed"
      return $true
    } else {
      Write-Fail "Tests failed!"
      return $false
    }
  } catch {
    Write-Fail "Failed to run tests: $_"
    return $false
  } finally {
    Pop-Location
  }
}

function Invoke-GitCommitPush {
  param([string]$CommitMessage)

  Push-Location $RepoRoot

  try {
    $branch = Get-CurrentBranch
    Write-Info "Current branch: $branch"

    if (-not (Test-GitChanges)) {
      Write-Warn "No changes to commit"
      return $false
    }

    Write-Info "Changes detected:"
    git status --short
    Write-Host ""

    if (-not (Invoke-Tests)) {
      if (-not $Force) {
        Write-Fail "Aborting due to test failures. Use -Force to override."
        return $false
      }
      Write-Warn "Proceeding despite test failures"
    }

    Write-Info "Staging changes (excluding target/)..."
    git add -A
    git reset HEAD target/ 2>$null
    git reset HEAD Cargo.lock 2>$null

    if ($LASTEXITCODE -ne 0) {
      Write-Fail "Failed to stage changes"
      return $false
    }

    Write-Info "Committing..."
    if ([string]::IsNullOrWhiteSpace($CommitMessage)) {
      $CommitMessage = Get-SmartCommitMessage
    }

    git commit -m $CommitMessage

    if ($LASTEXITCODE -ne 0) {
      Write-Fail "Failed to commit"
      return $false
    }

    Write-Success "Committed: $CommitMessage"

    Write-Info "Pushing to origin/$branch..."
    git push origin $branch

    if ($LASTEXITCODE -ne 0) {
      Write-Fail "Failed to push"
      Write-Warn "Try: git pull origin $branch --rebase"
      return $false
    }

    Write-Success "Successfully pushed to GitHub!"

    $repoUrl = git remote get-url origin 2>$null
    if ($repoUrl -match "github.com[:/](.+?)/.+?\.git") {
      $repoPath = $repoUrl -replace ".*github.com[:/](.+)", '$1' -replace "\.git$", ""
      Write-Info "View pipeline: https://github.com/$repoPath/actions"
    }

    return $true
  } catch {
    Write-Fail "Error: $_"
    return $false
  } finally {
    Pop-Location
  }
}

function Start-FileWatcher {
  Write-Info "Watching for changes (every $WatchInterval seconds)"
  Write-Info "Press Ctrl+C to stop"
  Write-Host ""

  $lastCommitTime = Get-Date
  $cooldown = 60

  while ($true) {
    Start-Sleep -Seconds $WatchInterval

    if (Test-GitChanges) {
      $elapsed = (Get-Date) - $lastCommitTime

      if ($elapsed.TotalSeconds -lt $cooldown) {
        $wait = [math]::Ceiling($cooldown - $elapsed.TotalSeconds)
        Write-Host "Cooldown: $wait seconds..." -ForegroundColor DarkGray
        continue
      }

      Write-Host ""
      Write-Info "Changes detected! Auto-committing..."

      if (Invoke-GitCommitPush -CommitMessage "") {
        $lastCommitTime = Get-Date
        Write-Success "Auto-commit successful"
      } else {
        Write-Warn "Auto-commit failed, will retry"
      }

      Write-Host ""
      Write-Info "Watching for changes..."
    }
  }
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  StoryStream Auto-Deploy" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-GitRepo)) {
  Write-Fail "Not in a git repository!"
  exit 1
}

try {
  $userName = git config user.name
  $userEmail = git config user.email

  if ([string]::IsNullOrWhiteSpace($userName) -or [string]::IsNullOrWhiteSpace($userEmail)) {
    Write-Warn "Git not configured!"
    Write-Info "Run: git config user.name 'Your Name'"
    Write-Info "Run: git config user.email 'your@email.com'"
  }
} catch {}

switch ($Mode) {
  'once' {
    if (Invoke-GitCommitPush -CommitMessage $Message) {
      Write-Host ""
      Write-Success "Deployment complete!"
      exit 0
    } else {
      Write-Host ""
      Write-Fail "Deployment failed!"
      exit 1
    }
  }
  'watch' {
    try {
      Start-FileWatcher
    } catch {
      Write-Fail "Watcher stopped: $_"
      exit 1
    }
  }
}