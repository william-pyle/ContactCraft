# Builds ContactSolutions into dist\: ContactSolutions.exe, the installer (MSI, when WiX is available),
# the portable zip, the guides and the download page.
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
Set-Location $root
& (Join-Path $root 'scripts\check-rename.ps1')

$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null

if (-not (Test-Path (Join-Path $root 'assets\contactsolutions.ico'))) {
    & (Join-Path $root 'packaging\make_icon.ps1')
}

Write-Host 'Checking the application and contact store...'
cargo test --workspace --locked --quiet
if ($LASTEXITCODE -ne 0) { throw 'The workspace tests failed.' }

Write-Host 'Building ContactSolutions...'
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'ContactSolutions failed to build.' }
Copy-Item (Join-Path $root 'target\release\ContactSolutions.exe') $dist -Force
Copy-Item (Join-Path $root 'NOTICE.md') $dist -Force
Copy-Item (Join-Path $root 'docs\shared-contacts.md') $dist -Force
Copy-Item (Join-Path $root 'docs\rename-migration.md') $dist -Force
Copy-Item (Join-Path $root 'docs\messagecraft-ai-instructions.md') $dist -Force

Write-Host 'Packing the download and its page...'
$zip = Join-Path $dist 'ContactSolutions-windows.zip'
if (Test-Path $zip) { Remove-Item $zip -Force }
Copy-Item (Join-Path $root 'LICENSE') (Join-Path $dist 'LICENSE.txt') -Force
Compress-Archive -Path (Join-Path $dist 'ContactSolutions.exe'), (Join-Path $dist 'NOTICE.md'), (Join-Path $dist 'LICENSE.txt'), (Join-Path $dist 'shared-contacts.md'), (Join-Path $dist 'messagecraft-ai-instructions.md'), (Join-Path $dist 'rename-migration.md') -DestinationPath $zip
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
Write-Host "Done: $dist\ContactSolutions.exe and $dist\index.html"
