# Native Warcraft Survivors on the incremental play profile.
# A later edit recompiles changed crates into target\play. It does not rebuild with --release.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

if (-not $env:WOW_DATA) {
    $data = Join-Path $root "wow-data\Data"
    if (Test-Path (Join-Path $data "terrain.MPQ")) {
        $env:WOW_DATA = $data
    }
}

cargo survivors
