#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Pack = Join-Path $Root "pack"
$Version = (Get-Content (Join-Path $Root "package.json") -Raw | ConvertFrom-Json).version
$Zip = Join-Path $Root "LaX-$Version.zip"

if (-not (Test-Path (Join-Path $Pack "lax.exe"))) {
    throw "pack\lax.exe missing. Run npm run pack first."
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
if (Test-Path $Zip) {
    Remove-Item -LiteralPath $Zip -Force
}
[System.IO.Compression.ZipFile]::CreateFromDirectory(
    $Pack,
    $Zip,
    [System.IO.Compression.CompressionLevel]::Fastest,
    $false
)

$info = Get-Item $Zip
$archive = [System.IO.Compression.ZipFile]::OpenRead($Zip)
try {
    $names = @($archive.Entries | ForEach-Object { $_.FullName })
    $dot = @($names | Where-Object { $_ -like "./*" }).Count
    $lax = @($names | Where-Object { $_ -eq "lax.exe" }).Count
    $index = @($names | Where-Object { $_ -eq "www/index.html" -or $_ -eq "www\index.html" }).Count
    $hello = @($names | Where-Object { $_ -like "www/hello*" -or $_ -like "www\hello*" }).Count
    Write-Host "ZIP=$($info.FullName)"
    Write-Host ("SIZE_MB={0}" -f [math]::Round($info.Length / 1MB, 1))
    Write-Host "ENTRIES=$($archive.Entries.Count)"
    Write-Host "LAX_EXE_AT_ROOT=$lax"
    Write-Host "WWW_INDEX=$index"
    Write-Host "DOT_SLASH=$dot"
    Write-Host "WWW_HELLO=$hello"
    $names | Select-Object -First 12 | ForEach-Object { Write-Host "  $_" }
    if ($info.Length -lt 50MB) { throw "zip too small" }
    if ($lax -ne 1) { throw "lax.exe missing at zip root" }
    if ($index -lt 1) { throw "www/index.html missing" }
    if ($dot -gt 0) { throw "zip has ./ prefixes" }
    if ($hello -gt 0) { throw "www still contains hello demo" }
    Write-Host "ZIP_OK"
}
finally {
    $archive.Dispose()
}
