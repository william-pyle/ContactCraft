# Current-source name audit. Old names require a specific migration/history reason.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$allowed = @{
    'README.md' = 'Latest published product remains historical v1.2.4.'
    'packaging/release-notes.md' = 'Historical release notes are preserved verbatim.'
    'docs/rename-migration.md' = 'Migration/recovery instructions and immutable Alpha mapping.'
    'store/src/migration.rs' = 'Readers for legacy data/settings/environment and executable metadata cleanup.'
    'store/tests/migration.rs' = 'Synthetic old-data/export and conflict fixtures.'
    'store/src/vcard.rs' = 'Read old exports; write only the new extension.'
    'src/io.rs' = 'Synthetic legacy JSON import fixture.'
    'web/index.html' = 'Unreleased notice distinguishes historical downloads.'
    'web/help.html' = 'Unreleased notice distinguishes historical downloads.'
    'scripts/check-rename.ps1' = 'This exception audit pattern and allowlist.'
    'scripts/check-window-chrome.ps1' = 'Opt-in isolated legacy-profile migration fixture and environment isolation.'
    'scripts/check-legacy-export.py' = 'Historical v1.2.4 exporter and synthetic import round-trip fixture.'
}
$pattern = '(?i)contact[-_ ]?craft'
$files = @(git ls-files --cached --others --exclude-standard) | Sort-Object -Unique
if ($LASTEXITCODE -ne 0) { throw 'Could not inventory source files.' }
$count = 0
foreach ($file in $files) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { continue }
    if ($file -match $pattern) { throw "Legacy tracked filename remains: $file" }
    if ([IO.Path]::GetExtension($file) -notin '.rs','.toml','.lock','.md','.json','.ps1','.py','.yml','.html','.svg','.wxs') { continue }
    $hits = [regex]::Matches([IO.File]::ReadAllText((Join-Path $PWD $file)), $pattern).Count
    if ($hits -and -not $allowed.ContainsKey($file)) { throw "Unexplained old product name in $file ($hits matches)" }
    if ($hits) { $count += $hits; Write-Output "$file : $hits documented matches - $($allowed[$file])" }
}
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$storeVersion = (Select-String -Path store/Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$nodeVersion = (Get-Content package.json -Raw | ConvertFrom-Json).version
if ($version -ne $storeVersion -or $version -ne $nodeVersion) { throw 'Source versions differ.' }
[xml]$wix = Get-Content packaging/windows/contactsolutions.wxs -Raw
if ($wix.Wix.Package.UpgradeCode -ne '9CC84440-9F4C-4EBB-BCCC-468A2E210F25') { throw 'MSI upgrade family changed.' }
Write-Output "Rename audit passed: $count explained old-name matches; no legacy source filenames; versions $version; MSI upgrade family preserved."
