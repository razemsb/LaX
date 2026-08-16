#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
$vcvars = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvars)) {
    $vcvars = "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
}
Set-Location $Root
if (Test-Path $vcvars) {
    cmd /c "`"$vcvars`" && npx tauri build --no-bundle"
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} else {
    npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
$built = Join-Path $Root "src-tauri\target\release\lax.exe"
if (-not (Test-Path $built)) {
    throw "release exe not found: $built"
}
Copy-Item $built (Join-Path $Root "lax.exe") -Force
Write-Host "Ready: $Root\lax.exe"
