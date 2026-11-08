param([Parameter(Mandatory)][string]$Destination)
$ErrorActionPreference='Stop'
$repoRoot=Split-Path $PSScriptRoot -Parent
$env:CARGO_HOME=Join-Path $repoRoot '.work/cargo-home'
$env:CARGO_TARGET_DIR=Join-Path $repoRoot 'target'
$metadataFile=Join-Path $repoRoot '.work/metadata.json'
& cargo metadata --offline --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc > $metadataFile
if ($LASTEXITCODE) {throw 'Could not inventory locked dependencies'}
$metadata=Get-Content -LiteralPath $metadataFile -Raw | ConvertFrom-Json
$notices=[System.Collections.Generic.List[string]]::new()
$notices.Add('# Third-party notices')
$notices.Add('')
$notices.Add('Locked dependency inventory for this Windows development build. License expressions come from package manifests. Included license and font notices are under licenses/. Application licensing has not been selected; this build is for local evaluation.')
$notices.Add('')
$notices.Add('| Package | License | Source |')
$notices.Add('| --- | --- | --- |')
foreach ($package in ($metadata.packages | Where-Object name -ne 'terminal-workspace' | Sort-Object name,version)) {
 $notices.Add("| $($package.name) $($package.version) | $($package.license) | $($package.repository) |")
 $packageDirectory=Split-Path $package.manifest_path -Parent
 $licenseDirectory=Join-Path $Destination "licenses/$($package.name)-$($package.version)"
 $files=@(Get-ChildItem -LiteralPath $packageDirectory -File | Where-Object Name -Match '^(LICENSE|LICENCE|COPYING|NOTICE|AUTHORS)')
 if ($package.license_file) {$files+=Get-Item -LiteralPath (Join-Path $packageDirectory $package.license_file)}
 if ($package.name -eq 'epaint_default_fonts') {$files+=Get-ChildItem -LiteralPath (Join-Path $packageDirectory 'fonts') -File | Where-Object Extension -eq '.txt'}
 if ($files.Count) {New-Item -ItemType Directory -Path $licenseDirectory -Force | Out-Null; $files | Sort-Object FullName -Unique | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination $licenseDirectory}}
}
$notices | Set-Content -LiteralPath (Join-Path $Destination 'THIRD_PARTY_NOTICES.md') -Encoding utf8
