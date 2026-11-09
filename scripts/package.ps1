param([string]$MakeNsis='makensis.exe',[switch]$SkipBuild,[string]$OutputDirectory='')
$ErrorActionPreference='Stop'
$repoRoot=Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $repoRoot
if (!$SkipBuild) {& "$PSScriptRoot/build.ps1" -Action build -Release; if ($LASTEXITCODE) {throw 'Release build failed'}}
$packageOutput=if ($OutputDirectory) {[IO.Path]::GetFullPath($OutputDirectory,$repoRoot)} else {Join-Path $repoRoot 'dist'}
$payload=Join-Path $packageOutput 'app'
New-Item -ItemType Directory -Path "$payload/plugin" -Force | Out-Null
Copy-Item target/release/terminal-workspace.exe $payload
Copy-Item README.md $payload
Copy-Item plugin/server.ts,plugin/tui.ts,plugin/core.mjs,plugin/package.json "$payload/plugin"
& "$PSScriptRoot/notices.ps1" -Destination $payload
if ($LASTEXITCODE) {throw 'License inventory failed'}
$uninstallManifest=Join-Path $repoRoot '.work/uninstall-files.nsh'
$uninstallLines=[System.Collections.Generic.List[string]]::new()
Get-ChildItem -LiteralPath $payload -File -Recurse | ForEach-Object {$relative=[IO.Path]::GetRelativePath($payload,$_.FullName); $uninstallLines.Add('Delete "$INSTDIR\'+$relative+'"')}
Get-ChildItem -LiteralPath $payload -Directory -Recurse | Sort-Object {$_.FullName.Length} -Descending | ForEach-Object {$relative=[IO.Path]::GetRelativePath($payload,$_.FullName); $uninstallLines.Add('RMDir "$INSTDIR\'+$relative+'"')}
$uninstallLines | Set-Content -LiteralPath $uninstallManifest -Encoding utf8
$output=Join-Path $packageOutput 'TerminalWorkspace-0.1.0-dev-x64-setup.exe'
& $MakeNsis "/DPAYLOAD=$payload" "/DOUTPUT=$output" "/DUNINSTALL_MANIFEST=$uninstallManifest" installer/terminal-workspace.nsi
if ($LASTEXITCODE) {throw 'Installer build failed'}
Get-FileHash $output -Algorithm SHA256
