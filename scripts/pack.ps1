#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Dest = Join-Path $Root "pack"

function Invoke-Robo($From, $To, $ExcludeDirs = @()) {
    if (-not (Test-Path $From)) { return }
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    $args = @($From, $To, "/E", "/NFL", "/NDL", "/NJH", "/NJS", "/NC", "/NS", "/NP", "/R:1", "/W:1", "/XD")
    # Do not exclude node_modules globally: Node's npm lives in bin/node/node_modules.
    $args += @($ExcludeDirs + @("logs", "tmp", ".git", "phpMyAdmin-5.2.0-english"))
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
$npmCli = Join-Path $Dest "bin\node\node_modules\npm\bin\npm-cli.js"
if (-not (Test-Path $npmCli)) {
    throw "npm missing in pack: $npmCli"
}
Invoke-Robo "$Root\etc" "$Dest\etc" @("tmp")
# Keep usr/apps/dbgate/package.json; node_modules (~350 MB) is installed from the GUI.
Invoke-Robo "$Root\usr" "$Dest\usr" @("node_modules")
Invoke-Robo "$Root\data" "$Dest\data"

# Release www is only the FileManager, unpacked at document root (www/index.html, www/data, ...).
$wwwDest = Join-Path $Dest "www"
New-Item -ItemType Directory -Force -Path $wwwDest | Out-Null
$fmZip = Join-Path $Root "realese (1).zip"
if (-not (Test-Path $fmZip)) {
    throw "FileManager zip not found: $fmZip"
}
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::ExtractToDirectory($fmZip, $wwwDest)
if (-not (Test-Path (Join-Path $wwwDest "index.html"))) {
    throw "FileManager extract failed: www/index.html missing"
}

New-Item -ItemType Directory -Force -Path "$Dest\etc\apps\phpMyAdmin\tmp" | Out-Null
New-Item -ItemType Directory -Force -Path "$Dest\bin\apache\Apache24\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$Dest\bin\nginx\nginx-1.14.0\logs" | Out-Null

if (Test-Path "$Root\logo.svg") {
    Copy-Item "$Root\logo.svg" "$Dest\logo.svg" -Force
}

function Install-PmaLaxTheme($LaXRoot, $PmaRoot) {
    $src = Join-Path $LaXRoot "usr\themes\phpmyadmin\lax"
    if (-not (Test-Path (Join-Path $src "theme.json"))) { return }
    if (-not (Test-Path (Join-Path $PmaRoot "index.php"))) { return }
    $dest = Join-Path $PmaRoot "themes\lax"
    $homme = Join-Path $PmaRoot "themes\pmahomme"
    New-Item -ItemType Directory -Force -Path @(
        "$dest\css", "$dest\jquery", "$dest\fonts", "$dest\img"
    ) | Out-Null
    Copy-Item (Join-Path $src "theme.json") (Join-Path $dest "theme.json") -Force
    if (Test-Path "$homme\img") {
        Invoke-Robo "$homme\img" "$dest\img"
    }
    if (Test-Path "$src\img\logo.svg") {
        Copy-Item "$src\img\logo.svg" "$dest\img\logo.svg" -Force
    }
    if (Test-Path "$src\fonts") {
        Copy-Item "$src\fonts\*" "$dest\fonts\" -Force
    }
    if (Test-Path "$homme\screen.png") {
        Copy-Item "$homme\screen.png" "$dest\screen.png" -Force
    }
    $base = ""
    if (Test-Path "$homme\css\theme.css") { $base = Get-Content "$homme\css\theme.css" -Raw -Encoding UTF8 }
    $over = Get-Content "$src\css\lax.css" -Raw -Encoding UTF8
    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText("$dest\css\theme.css", $base + "`n`n/* ---- LaX ---- */`n" + $over, $utf8)
    if (Test-Path "$homme\css\theme.rtl.css") {
        $rtl = Get-Content "$homme\css\theme.rtl.css" -Raw -Encoding UTF8
        [System.IO.File]::WriteAllText("$dest\css\theme.rtl.css", $rtl + "`n`n/* ---- LaX ---- */`n" + $over, $utf8)
    }
    $ui = ""
    if (Test-Path "$homme\jquery\jquery-ui.css") { $ui = Get-Content "$homme\jquery\jquery-ui.css" -Raw -Encoding UTF8 }
    [System.IO.File]::WriteAllText("$dest\jquery\jquery-ui.css", $ui + "`n`n/* ---- LaX ---- */`n" + $over, $utf8)
    $cfg = Join-Path $PmaRoot "config.inc.php"
    if (Test-Path $cfg) {
        $body = Get-Content $cfg -Raw -Encoding UTF8
        $mark = "`n/* LaX theme */`n"
        $idx = $body.IndexOf($mark)
        if ($idx -ge 0) { $body = $body.Substring(0, $idx) }
        $body += $mark + "`$cfg['ThemeDefault'] = 'lax';`n`$cfg['NavigationDisplayLogo'] = true;`n`$cfg['NavigationWidth'] = 268;`n"
        $utf8 = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($cfg, $body, $utf8)
    }
    $hdr = Join-Path $PmaRoot "templates\header.twig"
    if (Test-Path $hdr) {
        $h = Get-Content $hdr -Raw -Encoding UTF8
        $h = $h.Replace('href="{{ theme_path }}/jquery/jquery-ui.css"', 'href="{{ theme_path }}/jquery/jquery-ui.css?lax=17"')
        $h = $h.Replace("css/theme{{ text_dir == 'rtl' ? '.rtl' }}.css?{{ version }}`"", "css/theme{{ text_dir == 'rtl' ? '.rtl' }}.css?{{ version }}&lax=17`"")
        $h = $h.Replace('jquery-ui.css?lax=12', 'jquery-ui.css?lax=17')
        $h = $h.Replace('jquery-ui.css?lax=13', 'jquery-ui.css?lax=17')
        $h = $h.Replace('jquery-ui.css?lax=14', 'jquery-ui.css?lax=17')
        $h = $h.Replace('&lax=12', '&lax=17')
        $h = $h.Replace('&lax=13', '&lax=17')
        $h = $h.Replace('&lax=14', '&lax=17')
        $utf8 = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($hdr, $h, $utf8)
    }
}
Install-PmaLaxTheme $Root "$Dest\etc\apps\phpMyAdmin"

# marker so first Start All rebases paths from the original tree
$unix = ($Dest -replace '\\', '/')
Set-Content -Path "$Dest\usr\.install-root" -Value "C:/Laragon/www/LaX" -Encoding ASCII

Write-Host "Packed to $Dest"
Write-Host "Copy the contents of pack\ to the folder where LaX should live, then run lax.exe"
