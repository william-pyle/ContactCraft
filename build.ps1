# Builds ContactCraft into dist\: ContactCraft.exe, the installer (MSI, when WiX is available),
# the portable zip, the guides and the download page.
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
Set-Location $root

$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null

if (-not (Test-Path (Join-Path $root 'assets\contactcraft.ico'))) {
    & (Join-Path $root 'packaging\make_icon.ps1')
}

Write-Host 'Checking the application and contact store...'
cargo test --workspace --quiet
if ($LASTEXITCODE -ne 0) { throw 'The workspace tests failed.' }

Write-Host 'Building ContactCraft...'
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'ContactCraft failed to build.' }
Copy-Item (Join-Path $root 'target\release\ContactCraft.exe') $dist -Force
Copy-Item (Join-Path $root 'NOTICE.md') $dist -Force
Copy-Item (Join-Path $root 'docs\shared-contacts.md') $dist -Force
Copy-Item (Join-Path $root 'docs\messagecraft-ai-instructions.md') $dist -Force

Write-Host 'Packing the download and its page...'
$zip = Join-Path $dist 'ContactCraft-windows.zip'
if (Test-Path $zip) { Remove-Item $zip -Force }
Copy-Item (Join-Path $root 'LICENSE') (Join-Path $dist 'LICENSE.txt') -Force
Compress-Archive -Path (Join-Path $dist 'ContactCraft.exe'), (Join-Path $dist 'NOTICE.md'), (Join-Path $dist 'LICENSE.txt'), (Join-Path $dist 'shared-contacts.md'), (Join-Path $dist 'messagecraft-ai-instructions.md') -DestinationPath $zip
Copy-Item (Join-Path $root 'web\*') $dist -Recurse -Force

# The installer (MSI), when WiX is available (see packaging\windows\package.ps1).
$haveWix = (Get-Command wix -ErrorAction SilentlyContinue) -or (Test-Path (Join-Path $root '.tools\wix\wix.exe')) -or ($env:WIX_EXE -and (Test-Path $env:WIX_EXE))
if ($haveWix) {
    Write-Host 'Building the installer...'
    & (Join-Path $root 'packaging\windows\package.ps1')
} elseif ($env:CI) {
    throw 'WiX is needed to build the installer on the build server.'
} else {
    Write-Warning 'WiX is not installed, so no installer was built (see packaging\windows\package.ps1).'
}
Write-Host "Done: $dist\ContactCraft.exe and $dist\index.html"
