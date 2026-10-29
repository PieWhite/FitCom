# Build distributable binaries without embedding the builder's personal path.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$previousFlags = $env:CARGO_ENCODED_RUSTFLAGS
$previousCl = $env:CL
$previousClAppend = $env:_CL_
$previousCFlags = $env:CFLAGS
$previousCxxFlags = $env:CXXFLAGS
try {
    # Rust remapping does not cover __FILE__ in native Opus/SQLite assertions.
    # MSVC requires deterministic mode before honoring its native /pathmap option.
    # /Z7 keeps CMake's Debug compiler probe from opening a remapped external PDB.
    $nativeFlags = '/Z7 /experimental:deterministic /pathmap:"' + $root + '=/build/fitcom"'
    if ($env:USERPROFILE) { $nativeFlags += ' /pathmap:"' + $env:USERPROFILE + '=/build/home"' }
    $env:CL = "$previousCl $nativeFlags".Trim()
    # MSVC appends _CL_ after command-line flags, so CMake cannot override /Z7 with /Zi.
    $env:_CL_ = "$previousClAppend $nativeFlags".Trim()
    $env:CFLAGS = "$previousCFlags $nativeFlags".Trim()
    $env:CXXFLAGS = "$previousCxxFlags $nativeFlags".Trim()
    $flags = @('-C', 'control-flow-guard=yes')
    if ($env:USERPROFILE) { $flags += "--remap-path-prefix=$($env:USERPROFILE)=/build/home" }
    $flags += "--remap-path-prefix=$root=/build/fitcom"
    $env:CARGO_ENCODED_RUSTFLAGS = $flags -join [char]31
    Push-Location $root
    try {
        # audiopus_sys does not track compiler environment changes. Its CMake cache
        # must be recreated or old __FILE__ strings can survive a release rebuild.
        $releaseTarget = [IO.Path]::GetFullPath((Join-Path $root 'target'))
        if (-not $releaseTarget.StartsWith($root + [IO.Path]::DirectorySeparatorChar)) {
            throw 'Release output escaped the workspace.'
        }
        & cargo clean --release -p audiopus_sys --target-dir $releaseTarget
        if ($LASTEXITCODE -ne 0) { throw 'Native release cache cleanup failed.' }
        & cargo build --locked --release -p fitcom --bins --target-dir $releaseTarget
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    } finally { Pop-Location }
    foreach ($name in @('fitcom.exe', 'fitcom-updater.exe', 'fitcom-release.exe')) {
        $file = Join-Path $root "target/release/$name"
        $bytes = [IO.File]::ReadAllBytes($file)
        foreach ($encoding in @([Text.Encoding]::UTF8, [Text.Encoding]::Unicode)) {
            $text = $encoding.GetString($bytes)
            foreach ($personalPath in @($env:USERPROFILE, $env:USERPROFILE.Replace('\','/'))) {
                if ($personalPath -and $text.IndexOf($personalPath, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
                    throw "Personal build path found in $name. Do not distribute."
                }
            }
        }
    }
    Write-Host 'Release binaries built; personal home paths absent.'
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previousFlags
    $env:CL = $previousCl
    $env:_CL_ = $previousClAppend
    $env:CFLAGS = $previousCFlags
    $env:CXXFLAGS = $previousCxxFlags
}
