# Native fixture checks. No installation, real contacts or existing app sessions.
param([switch]$LegacyProfile)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
cargo build
if ($LASTEXITCODE -ne 0) { throw 'Debug build failed.' }
$evidence = Join-Path ([IO.Path]::GetTempPath()) ('contactsolutions-chrome-' + [guid]::NewGuid().ToString('N'))
foreach ($mode in 'main', 'contact', 'group') {
    $fixture = Join-Path $evidence $mode
    $profile = Join-Path $fixture 'profile'
    $settingsFolder = Join-Path $profile 'ContactSolutions'
    $seedFolder = if ($LegacyProfile) { Join-Path $profile 'ContactCraft' } else { $settingsFolder }
    $seedName = if ($LegacyProfile) { 'contactcraft-ui.json' } else { 'contactsolutions-ui.json' }
    New-Item -ItemType Directory -Force $seedFolder | Out-Null
    $settings = @{ appearance = 'Light'; zoom = 100; label_pane = $true; tighter = $false; sort_by_family = $false; check_updates = $false; skip_version = 'fixture-only'; future_fixture_setting = 'preserved' }
    $settings | ConvertTo-Json | Set-Content (Join-Path $seedFolder $seedName)
    $seedHash = (Get-FileHash (Join-Path $seedFolder $seedName)).Hash
    $arguments = '--demo --no-update-check --chrome-smoke-test'
    $resultPath = Join-Path $fixture 'dialog-result.json'
    if ($mode -eq 'contact') { $arguments += ' --new-contact' }
    if ($mode -eq 'group') { $arguments += ' --new-group --group-name WindowFixture' }
    if ($mode -ne 'main') { $arguments += ' --result-file "' + $resultPath + '"' }
    $launch = [Diagnostics.ProcessStartInfo]::new((Join-Path $PWD 'target/debug/ContactSolutions.exe'), $arguments)
    $launch.UseShellExecute = $false
    $launch.EnvironmentVariables['APPDATA'] = $profile
    $launch.EnvironmentVariables.Remove('CONTACTCRAFT_DATA')
    $launch.EnvironmentVariables['CONTACTSOLUTIONS_DATA'] = Join-Path $fixture 'store'
    $launch.EnvironmentVariables['CONTACTSOLUTIONS_CHROME_CHECK_DIR'] = $fixture
    $process = [Diagnostics.Process]::Start($launch)
    if (-not $process.WaitForExit(20000)) {
        $process.Kill() # Only the test process this script just created.
        throw "Native $mode fixture timed out. Evidence: $fixture"
    }
    $expectedExit = if ($mode -eq 'main') { 0 } else { 1 }
    if ($process.ExitCode -ne $expectedExit) { throw "Native $mode fixture exited $($process.ExitCode). Evidence: $fixture" }
    $result = Get-Content (Join-Path $fixture 'native-results.json') -Raw | ConvertFrom-Json
    if (-not $result.passed) { throw "Native $mode checks failed. Evidence: $fixture" }
    foreach ($name in 'normal', 'maximized', 'narrow', 'zoom-200') {
        if (-not (Test-Path (Join-Path $fixture ($name + '.png')))) { throw "Missing $mode screenshot: $name" }
    }
    $persisted = Get-Content (Join-Path $settingsFolder 'contactsolutions-ui.json') -Raw | ConvertFrom-Json
    foreach ($key in $settings.Keys) {
        if ($persisted.$key -ne $settings[$key]) { throw "Fixture setting changed: $key" }
    }
    if ($LegacyProfile) {
        if ((Get-FileHash (Join-Path $seedFolder $seedName)).Hash -ne $seedHash) { throw 'Legacy settings changed during migration.' }
        if (-not (Test-Path (Join-Path $settingsFolder 'migration-receipt.json'))) { throw 'Migration receipt missing.' }
    }
    if ($mode -ne 'main') {
        $dialogResult = Get-Content $resultPath -Raw | ConvertFrom-Json
        if ($dialogResult.status -ne 'cancelled') { throw "$mode close did not cancel the draft." }
    }
    Write-Output "$mode native checks, screenshots, settings persistence and exit contract passed at $($result.dpi) DPI."
}
Write-Output "Evidence: $evidence"
