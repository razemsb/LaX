#Requires -Version 5.1
<#
.SYNOPSIS
  Download portable Mailpit + Node. DbGate is optional and installed from the LaX GUI.
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
# Pinned version + SHA256: exactly what the release ships.
$MailpitVersion = "v1.33.0"
$MailpitSha256 = "47122dd828ff80dcaef62b2952c478b178afadadf066c990489608e537121bfd"
$mailDir = Join-Path $Root "bin\mailpit"
$mailExe = Join-Path $mailDir "mailpit.exe"
if (-not (Test-Path $mailExe)) {
    Write-Host "==> Mailpit $MailpitVersion"
    New-Item -ItemType Directory -Force -Path $mailDir | Out-Null
    $zip = Join-Path $Tmp "mailpit.zip"
    Get-Zip "https://github.com/axllent/mailpit/releases/download/$MailpitVersion/mailpit-windows-amd64.zip" $zip
    if ($MailpitSha256 -ne "CHANGE_ME_MAILPIT_SHA256") {
        $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash.ToLower()
        if ($got -ne $MailpitSha256.ToLower()) { throw "mailpit sha256 mismatch: $got" }
    }
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
# Pinned version + SHA256: exactly what the release ships.
$NodeVersion = "v22.23.4"
$NodeSha256 = "cbc74b5e0982c9dbb41525fd8fb5dcd32fbee237863f7829072f4639a1ce5f20"
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
        Write-Host "==> Node $NodeVersion (win-x64)"
        $ver = $NodeVersion
        $name = "node-$ver-win-x64"
        $zip = Join-Path $Tmp "node.zip"
        Get-Zip "https://nodejs.org/dist/$ver/$name.zip" $zip
        $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash.ToLower()
        if ($got -ne $NodeSha256) { throw "node sha256 mismatch: $got" }
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

# --- cloudflared (quick tunnels, public share) ---
# Pinned version: exactly the binary the release was tested with.
$CfVersion = "2026.10.7"
$CfSha256 = "2493dbc168923bb3e59e5e7716f71deced92992eea9acd6e9d74f84ac7751343"
$cfDir = Join-Path $Root "bin\cloudflared"
$cfExe = Join-Path $cfDir "cloudflared.exe"
if (-not (Test-Path $cfExe)) {
    Write-Host "==> cloudflared $CfVersion (win-amd64)"
    New-Item -ItemType Directory -Force -Path $cfDir | Out-Null
    Get-Zip "https://github.com/cloudflare/cloudflared/releases/download/$CfVersion/cloudflared-windows-amd64.exe" $cfExe
    $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $cfExe).Hash.ToLower()
    if ($got -ne $CfSha256) { throw "cloudflared sha256 mismatch: $got" }
    Write-Host "ok $cfExe"
} else {
    Write-Host "skip cloudflared (already in bin/cloudflared)"
}

# --- DbGate (web, community) ---
$dbgateDir = Join-Path $Root "usr\apps\dbgate"
$dbgateJs = Join-Path $dbgateDir "node_modules\dbgate-serve\bin\dbgate-serve.js"
if (-not (Test-Path $dbgateJs)) {
    Write-Host "==> DbGate (dbgate-serve)"
    if (-not (Test-Path $nodeExe)) {
        throw "Node missing. Cannot install DbGate."
    }
    New-Item -ItemType Directory -Force -Path $dbgateDir | Out-Null
    $pkg = Join-Path $dbgateDir "package.json"
    if (-not (Test-Path $pkg)) {
        Set-Content -Path $pkg -Encoding UTF8 -Value @'
{
  "name": "lax-dbgate",
  "private": true,
  "dependencies": {
    "dbgate-serve": "7.2.5"
  }
}
'@
    }
    $npmCli = Join-Path $nodeDir "node_modules\npm\bin\npm-cli.js"
    if (-not (Test-Path $npmCli)) {
        $npmCli = Join-Path $nodeDir "npm.cmd"
    }
    Push-Location $dbgateDir
    try {
        if (Test-Path (Join-Path $nodeDir "node_modules\npm\bin\npm-cli.js")) {
            & $nodeExe (Join-Path $nodeDir "node_modules\npm\bin\npm-cli.js") install --omit=dev --no-fund --no-audit
        } else {
            & (Join-Path $nodeDir "npm.cmd") install --omit=dev --no-fund --no-audit
        }
        if ($LASTEXITCODE -ne 0) { throw "npm install dbgate-serve failed ($LASTEXITCODE)" }
    } finally {
        Pop-Location
    }
    if (-not (Test-Path $dbgateJs)) {
        throw "DbGate missing after npm install: $dbgateJs"
    }
    Write-Host "ok $dbgateJs"
} else {
    Write-Host "skip DbGate (already in usr/apps/dbgate)"
}

Write-Host "done"
