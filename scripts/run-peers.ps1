# Isolated local peers. Never stop a normal FitCom installation.
[CmdletBinding()]
param([ValidateRange(2,16)][int]$Count=2,[switch]$Stop,[switch]$Release,[switch]$Logs)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$peerRoot=[IO.Path]::GetFullPath((Join-Path $root '.localpeers'))
if ($peerRoot -ne (Join-Path $root '.localpeers')) { throw 'Invalid test directory.' }
$pidFile=Join-Path $peerRoot 'processes.txt'
if ($Logs) {
    Get-ChildItem -LiteralPath $peerRoot -Recurse -Filter '*.log' | ForEach-Object { Get-Content -LiteralPath $_.FullName -Tail 25 }
    exit 0
}
if (Test-Path -LiteralPath $pidFile) {
    foreach ($peerPid in Get-Content -LiteralPath $pidFile) {
        if ($peerPid -notmatch '^\d+$') { throw 'Invalid saved process ID.' }
        $process=Get-CimInstance Win32_Process -Filter "ProcessId = $peerPid" -ErrorAction SilentlyContinue
        if ($process -and $process.Name -eq 'fitcom.exe' -and $process.CommandLine.Contains($peerRoot)) {
            Stop-Process -Id ([int]$peerPid) -ErrorAction SilentlyContinue
        }
    }
}
if ($Stop) { Write-Host 'Local test peers stopped.'; exit 0 }
$buildArgs=@('build','--locked','-p','fitcom','--bin','fitcom')
$profile='debug'
if ($Release) { $buildArgs+='--release'; $profile='release' }
Push-Location $root
try { & cargo @buildArgs; if ($LASTEXITCODE) { throw 'Build failed.' } } finally { Pop-Location }
$exe=Join-Path $root "target/$profile/fitcom.exe"
if (Test-Path -LiteralPath $peerRoot) {
    $resolved=(Resolve-Path -LiteralPath $peerRoot).Path
    if ($resolved -ne (Join-Path $root '.localpeers')) { throw 'Refusing to remove unexpected directory.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
New-Item -ItemType Directory -Path $peerRoot | Out-Null
for ($i=0;$i -lt $Count;$i++) {
    $dir=Join-Path $peerRoot "peer$($i+1)"
    New-Item -ItemType Directory -Path $dir | Out-Null
    $lines=@("display_name = `"peer$($i+1)`"","control_port = $(42650+$i*2)","media_port = $(42651+$i*2)",'port_mapping = false')
    for ($j=0;$j -lt $Count;$j++) {
        if ($j -ne $i) { $lines+=@('','[[peers]]','address = "127.0.0.1"',"label = `"peer$($j+1)`"","control_port = $(42650+$j*2)") }
    }
    $lines | Set-Content -LiteralPath (Join-Path $dir 'config.toml') -Encoding utf8
    $started=Start-Process -FilePath $exe -ArgumentList @('--data-dir',('"'+$dir+'"')) -PassThru -WindowStyle Hidden
    $started.Id | Add-Content -LiteralPath $pidFile
    Write-Host "peer$($i+1) started, ports $(42650+$i*2)/$(42651+$i*2)."
}
