#Requires -Version 5.1
<#
.SYNOPSIS
  Download portable Mailpit + Node into bin/ so the stack does not need system installs.
#>
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Tmp = Join-Path $Root "tmp\fetch-tools"
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null

function Get-Zip($Url, $OutFile) {
    Write-Host "GET $Url"
    Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing
}

# --- Mailpit ---
$mailDir = Join-Path $Root "bin\mailpit"
$mailExe = Join-Path $mailDir "mailpit.exe"
if (-not (Test-Path $mailExe)) {
    Write-Host "==> Mailpit"
    New-Item -ItemType Directory -Force -Path $mailDir | Out-Null
    $zip = Join-Path $Tmp "mailpit.zip"
    Get-Zip "https://github.com/axllent/mailpit/releases/latest/download/mailpit-windows-amd64.zip" $zip
    Expand-Archive -LiteralPath $zip -DestinationPath $mailDir -Force
    if (-not (Test-Path $mailExe)) {
        $found = Get-ChildItem -Path $mailDir -Recurse -Filter "mailpit.exe" | Select-Object -First 1
        if ($found) {
            Move-Item $found.FullName $mailExe -Force
        }
    }
    if (-not (Test-Path $mailExe)) {
        throw "mailpit.exe missing after extract"
    }
    Write-Host "ok $mailExe"
} else {
    Write-Host "skip Mailpit (already in bin/mailpit)"
}

# --- Node ---
$nodeDir = Join-Path $Root "bin\node"
$nodeExe = Join-Path $nodeDir "node.exe"
if (-not (Test-Path $nodeExe)) {
    $laragonNode = @(
        "C:\Laragon\bin\nodejs\node.exe",
        "C:\Laragon\bin\node\node.exe"
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($laragonNode) {
        Write-Host "==> Node from Laragon $($laragonNode)"
        New-Item -ItemType Directory -Force -Path $nodeDir | Out-Null
        Copy-Item (Join-Path (Split-Path $laragonNode) "*") $nodeDir -Recurse -Force
    } else {
        Write-Host "==> Node LTS (win-x64)"
        $idx = Invoke-RestMethod "https://nodejs.org/dist/index.json"
        $rel = $idx | Where-Object { $_.lts -and $_.version -like "v22.*" } | Select-Object -First 1
        if (-not $rel) {
            $rel = $idx | Where-Object { $_.lts } | Select-Object -First 1
        }
        $ver = $rel.version
        $name = "node-$ver-win-x64"
        $zip = Join-Path $Tmp "node.zip"
        Get-Zip "https://nodejs.org/dist/$ver/$name.zip" $zip
        $extract = Join-Path $Tmp "node"
        if (Test-Path $extract) { Remove-Item $extract -Recurse -Force }
        Expand-Archive -LiteralPath $zip -DestinationPath $extract -Force
        $inner = Join-Path $extract $name
        if (-not (Test-Path $inner)) {
            $inner = Get-ChildItem $extract -Directory | Select-Object -First 1 -ExpandProperty FullName
        }
        if (Test-Path $nodeDir) { Remove-Item $nodeDir -Recurse -Force }
        Move-Item $inner $nodeDir
    }
    if (-not (Test-Path $nodeExe)) {
        throw "node.exe missing after install"
    }
    Write-Host "ok $nodeExe"
} else {
    Write-Host "skip Node (already in bin/node)"
}

Write-Host "done"
