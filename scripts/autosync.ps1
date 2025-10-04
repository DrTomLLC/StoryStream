<# 
  Auto-syncs every change to the remote branch (default: first-dev).
  Works on Windows PowerShell 5.1 and PowerShell 7.
#>

param(
  [string]$Branch = "first-dev",
  [string]$RemoteUrl = "https://github.com/DrTomLLC/StoryStream.git",
  [int]$PollSeconds = 2
)

$ErrorActionPreference = "Stop"

function In-GitRepo {
  try { git rev-parse --is-inside-work-tree | Out-Null; return $true } catch { return $false }
}

# 1) Ensure we are in a git repo; init and wire remote if not
if (-not (In-GitRepo)) {
  Write-Host "Initializing new git repo and wiring remote..." -ForegroundColor Yellow
  git init | Out-Null
  $hasOrigin = (git remote 2>$null) -match '^origin$'
  if (-not $hasOrigin) {
    git remote add origin $RemoteUrl | Out-Null
  } else {
    git remote set-url origin $RemoteUrl | Out-Null
  }
}

# 2) Ensure target branch exists locally and is checked out
try { $cur = (git rev-parse --abbrev-ref HEAD).Trim() } catch { $cur = "" }
if ($cur -ne $Branch) {
  Write-Host "Checking out branch $Branch ..." -ForegroundColor Cyan
  try { git fetch origin $Branch | Out-Null } catch { }  # ignore if branch doesn't exist remotely yet
  git checkout -B $Branch | Out-Null
}

# 3) One-time initial push if remote branch missing
try {
  git ls-remote --exit-code --heads origin $Branch | Out-Null
} catch {
  Write-Host "Creating remote branch $Branch ..." -ForegroundColor Cyan
  git push -u origin $Branch | Out-Null
}

Write-Host "Auto-sync ON → pushing changes to origin/$Branch every $PollSeconds s. Press Ctrl+C to stop." -ForegroundColor Green

# Helper: returns $true if there are unstaged, staged, or untracked changes
function Has-Changes {
  git diff --quiet; $changed = ($LASTEXITCODE -ne 0)
  git diff --cached --quiet; $staged = ($LASTEXITCODE -ne 0)
  $untracked = ((git ls-files --others --exclude-standard) | Measure-Object).Count -gt 0
  return ($changed -or $staged -or $untracked)
}

# 4) Polling loop (simple & robust)
while ($true) {
  try {
    if (Has-Changes) {
      git add -A | Out-Null
      # commit only if there is something staged
      git diff --cached --quiet; $hasStaged = ($LASTEXITCODE -ne 0)
      if ($hasStaged) {
        $ts = Get-Date -Format "yyyy-MM-ddTHH:mm:ss"
        git commit -m "chore(autosync): $ts" | Out-Null
      }
      # rebase to keep linear history if remote advanced (e.g., CI bots)
      try { git pull --rebase origin $Branch | Out-Null } catch { }
      git push -u origin $Branch | Out-Null
      Write-Host "Synced at $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor DarkGray
    }
  } catch {
    Write-Host "Sync error: $($_.Exception.Message)" -ForegroundColor Red
  }
  Start-Sleep -Seconds $PollSeconds
}
