# Inspect tracked files, not ignored local files. Run from a clean Git snapshot.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$tracked = @(& git -C $root ls-files)
if ($LASTEXITCODE -ne 0 -or $tracked.Count -eq 0) { throw 'No tracked source inventory.' }
$forbidden = '(^|/)(AGENTS\.md|CLAUDE\.md|\.private|\.claude|\.agents|\.codex|\.opencode|\.impeccable|\.cursor|\.clinerules|\.windsurf|\.localpeers|node_modules|data|logs|target[^/]*)(/|$)|\.(pk8|key|pem|pfx|p12|exe|dll|pdb|log|sqlite|dmp|mp4|wav)$|(^|/)\.env($|\.)|(^|/)identity\.toml$|^config\.toml$'
$bad = @($tracked | Where-Object { $_ -match $forbidden })
if ($bad.Count) { throw ('Private/generated files are tracked: ' + ($bad -join ', ')) }
$secretPattern = 'gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{30,}|-----BEGIN (?:[A-Z]+ )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}'
foreach ($relative in $tracked) {
    $file = Join-Path $root $relative
    if ([IO.Path]::GetExtension($file) -notin '.png','.ico','.ttf') {
        $text = [IO.File]::ReadAllText($file)
        if ($text -match $secretPattern) { throw "Possible credential in $relative; inspect privately." }
    }
}
Write-Host "Public source inventory checked: $($tracked.Count) files. Run a dedicated secret scanner too."
