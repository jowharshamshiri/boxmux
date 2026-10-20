# Prepares a Windows guest for building BoxMux: the MSVC build tools (once per guest) and
# Rust (once per tools cache). Dot-sourced by test.ps1 and build.ps1.
$env:RUSTUP_HOME = Join-Path $env:MG_TEST_CACHE_TOOLS 'rustup'
$env:CARGO_HOME = Join-Path $env:MG_TEST_CACHE_TOOLS 'cargo'
$env:CARGO_TARGET_DIR = $env:MG_TEST_CACHE_TARGET
$env:CARGO_TERM_COLOR = 'never'
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"

function Invoke-Native([string] $what, [scriptblock] $command) {
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$what failed with exit code $LASTEXITCODE" }
}

# Rust's MSVC target links with the MSVC compiler and the Windows SDK.
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$haveVc = (Test-Path $vswhere) -and (& $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)
if (-not $haveVc) {
    $installer = Join-Path $env:MG_TEST_SCRATCH 'vs_BuildTools.exe'
    Invoke-WebRequest 'https://aka.ms/vs/17/release/vs_BuildTools.exe' -OutFile $installer
    $p = Start-Process $installer -Wait -PassThru -ArgumentList @(
        '--quiet', '--wait', '--norestart', '--nocache',
        '--add', 'Microsoft.VisualStudio.Workload.VCTools', '--includeRecommended')
    # 3010: installed, a restart is pending (not needed for the tools to work).
    if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) { throw "installing the MSVC build tools failed with exit code $($p.ExitCode)" }
}

if (-not (Test-Path "$env:CARGO_HOME\bin\cargo.exe")) {
    $rustup = Join-Path $env:MG_TEST_SCRATCH 'rustup-init.exe'
    Invoke-WebRequest 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile $rustup
    Invoke-Native 'installing Rust' { & $rustup -y --no-modify-path --profile minimal --default-toolchain stable -c clippy -c rustfmt }
}
Invoke-Native 'updating Rust' { rustup update stable --no-self-update }
