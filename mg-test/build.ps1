# Builds the release binary on a Windows guest and packages it as a zip under out\dist.
$ErrorActionPreference = 'Stop'
Set-Location $env:MG_TEST_WORKSPACE
. (Join-Path $PSScriptRoot 'windows-toolchain.ps1')
New-Item -ItemType Directory -Force out\dist | Out-Null

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"$').Matches[0].Groups[1].Value
cmd /c "cargo build --release --locked > out\build.log 2>&1"
if ($LASTEXITCODE -ne 0) { Get-Content out\build.log -Tail 40; throw "cargo build failed with exit code $LASTEXITCODE" }

$stage = "out\boxmux-$version-windows-x86_64"
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item "$env:CARGO_TARGET_DIR\release\boxmux.exe", README.md, LICENSE $stage
Compress-Archive -Path $stage -DestinationPath "out\dist\boxmux-$version-windows-x86_64.zip" -Force
Get-ChildItem out\dist
