param([ValidateSet('check','test','build','run','clippy','fmt')][string]$Action='build',[switch]$Release,[switch]$Workspace)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $repoRoot
# Keep this checkout's build cache off the full system volume.
$env:CARGO_HOME = Join-Path $repoRoot '.work/cargo-home'
$env:CARGO_TARGET_DIR = Join-Path $repoRoot 'target'
$env:CARGO_BUILD_JOBS = '4'
$env:CARGO_INCREMENTAL = '0'
# The machine's existing SDK is portable; normal developer shells need none of this.
if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
    $vcRoot = 'E:/VS2022BuildTools/VC/Tools/MSVC/14.44.35207'
    $sdkRoot = 'E:/Temp/nexus-xwin-cache/xwin/sdk'
    if (-not (Test-Path -LiteralPath "$vcRoot/bin/Hostx64/x64/link.exe")) { throw 'Run this script from an x64 Visual Studio developer shell.' }
    $env:PATH = "$vcRoot/bin/Hostx64/x64;$env:PATH"
    $env:LIB = "$vcRoot/lib/x64;$sdkRoot/lib/um/x86_64;$sdkRoot/lib/ucrt/x86_64"
    $env:INCLUDE = "$vcRoot/include;$sdkRoot/include/um;$sdkRoot/include/shared;$sdkRoot/include/ucrt"
}
$cargoArguments = @($Action)
if ($Workspace) {$cargoArguments += '--workspace'}
if ($Release -and $Action -notin @('fmt')) { $cargoArguments += '--release' }
& cargo @cargoArguments
exit $LASTEXITCODE
