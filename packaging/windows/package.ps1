<#
.SYNOPSIS
  Build ContactCraft's Windows installer (MSI) from the program in dist\.

.DESCRIPTION
  Run build.ps1 first (it calls this script itself when WiX is available). Produces
  dist\ContactCraft-windows-x64.msi: a per-machine installer into Program Files with a Start Menu
  shortcut, like MailCraft's and MessageCraft's.

  Needs WiX v5 with its UI and Util extensions, found in this order:
    1. `wix` on PATH:
         dotnet tool install --global wix --version 5.0.2
    2. app\.tools\wix\wix.exe (dotnet tool install --tool-path .tools\wix wix --version 5.0.2),
       with app\.tools\dotnet used as the .NET runtime when it's there;
    3. the WIX_EXE environment variable (and DOTNET_ROOT for its .NET runtime).
  The extensions are added to WiX's own cache when they're missing.
#>
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$root = Resolve-Path (Join-Path $here '..\..')
$dist = Join-Path $root 'dist'

$wix = Get-Command wix -ErrorAction SilentlyContinue
if ($wix) { $wixExe = $wix.Source }
elseif (Test-Path (Join-Path $root '.tools\wix\wix.exe')) {
    $wixExe = Join-Path $root '.tools\wix\wix.exe'
    $dotnetDir = Join-Path $root '.tools\dotnet'
    if (Test-Path $dotnetDir) { $env:DOTNET_ROOT = $dotnetDir }
}
elseif ($env:WIX_EXE -and (Test-Path $env:WIX_EXE)) { $wixExe = $env:WIX_EXE }
else { throw 'WiX is not installed (see the notes at the top of package.ps1).' }
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'

# The version comes from Cargo.toml, so the app, its update check and the installer always agree.
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$msiVersion = ($version -split '[-+]')[0]

foreach ($ext in 'WixToolset.UI.wixext/5.0.2', 'WixToolset.Util.wixext/5.0.2') {
    $name = ($ext -split '/')[0]
    $have = & $wixExe extension list -g 2>$null | Select-String -SimpleMatch $name
    if (-not $have) { & $wixExe extension add -g $ext | Out-Null }
}

# The licence page: the licence and the notices, as RTF.
$text = (Get-Content (Join-Path $root 'LICENSE') -Raw) + "`n`n" + (Get-Content (Join-Path $root 'NOTICE.md') -Raw)
$escaped = $text -replace '\\', '\\' -replace '\{', '\{' -replace '\}', '\}'
$escaped = ($escaped -replace '(?m)^#+\s*', '' -replace '\*\*', '' -replace '`', '')
$sb = New-Object System.Text.StringBuilder
foreach ($ch in $escaped.ToCharArray()) {
    $code = [int]$ch
    if ($ch -eq "`n") { [void]$sb.Append('\par ') }
    elseif ($ch -eq "`r") { }
    elseif ($code -gt 127) { [void]$sb.Append("\u$code?") }
    else { [void]$sb.Append($ch) }
}
$rtfPath = Join-Path $env:TEMP 'contactcraft-license.rtf'
"{\rtf1\ansi\deff0{\fonttbl{\f0 Segoe UI;}}\fs18 $($sb.ToString())}" | Set-Content -Path $rtfPath -Encoding ascii

$msi = Join-Path $dist 'ContactCraft-windows-x64.msi'
if (Test-Path $msi) { Remove-Item $msi -Force }
& $wixExe build (Join-Path $here 'contactcraft.wxs') -arch x64 `
    -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
    -d "Version=$msiVersion" -d "BinDir=$dist" `
    -d "IconPath=$(Join-Path $root 'assets\contactcraft.ico')" -d "LicenseRtf=$rtfPath" `
    -o $msi
if ($LASTEXITCODE -ne 0) { throw 'The installer failed to build.' }
Remove-Item (Join-Path $dist 'ContactCraft-windows-x64.wixpdb') -ErrorAction SilentlyContinue
Write-Host "Installer: $msi"
