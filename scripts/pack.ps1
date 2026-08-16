#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Dest = Join-Path $Root "pack"

function Invoke-Robo($From, $To, $ExcludeDirs = @()) {
    if (-not (Test-Path $From)) { return }
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    $args = @($From, $To, "/E", "/NFL", "/NDL", "/NJH", "/NJS", "/NC", "/NS", "/NP", "/R:1", "/W:1", "/XD")
    $args += @($ExcludeDirs + @("logs", "tmp", "node_modules", ".git", "phpMyAdmin-5.2.0-english"))
    $args += @("/XF", "*.pdb", "*.log")
    & robocopy @args | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "robocopy failed ($LASTEXITCODE): $From" }
}

if (Test-Path $Dest) {
    Remove-Item -LiteralPath $Dest -Recurse -Force
}
New-Item -ItemType Directory -Force -Path @(
    $Dest,
    "$Dest\tmp",
    "$Dest\logs",
    "$Dest\usr"
) | Out-Null

$exe = Join-Path $Root "lax.exe"
if (-not (Test-Path $exe)) {
    throw "lax.exe not found. Run npm run build:exe first."
}
Copy-Item $exe (Join-Path $Dest "lax.exe") -Force

Invoke-Robo "$Root\bin" "$Dest\bin" @("logs")
Invoke-Robo "$Root\etc" "$Dest\etc" @("tmp")
Invoke-Robo "$Root\usr" "$Dest\usr"
Invoke-Robo "$Root\www" "$Dest\www" @("PHP-vizit")
Invoke-Robo "$Root\data" "$Dest\data"

New-Item -ItemType Directory -Force -Path "$Dest\etc\apps\phpMyAdmin\tmp" | Out-Null
New-Item -ItemType Directory -Force -Path "$Dest\bin\apache\Apache24\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$Dest\bin\nginx\nginx-1.14.0\logs" | Out-Null

if (Test-Path "$Root\logo.svg") {
    Copy-Item "$Root\logo.svg" "$Dest\logo.svg" -Force
}

# marker so first Start All rebases paths from the original tree
$unix = ($Dest -replace '\\', '/')
Set-Content -Path "$Dest\usr\.install-root" -Value "C:/Laragon/www/LaX" -Encoding ASCII

Write-Host "Packed to $Dest"
Write-Host "Copy the contents of pack\ to the folder where LaX should live, then run lax.exe"
