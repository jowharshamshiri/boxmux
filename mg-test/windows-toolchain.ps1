# Prepares a Windows guest for building BoxMux: the MSVC build tools (once per guest), QuickRust
# (once per rust cache) and Git (once per tools cache). Dot-sourced by test.ps1.
#
# Rust is built by QuickRust, and QuickRust is the one thing installed for it: it installs and
# manages the toolchain itself and decides where builds go. Its home is the rust cache, so the
# toolchain, the crates it downloaded and its build cache outlive the guest. Nothing here updates
# in the background, so nothing still writes into the cache when the run ends: the toolchain and
# QuickRust are brought up to date below, before the build.
if (-not $env:MG_TEST_CACHE_RUST) { throw 'the rust cache is not mounted; the plan must declare it' }
if (-not $env:MG_TEST_CACHE_TOOLS) { throw 'the tools cache is not mounted; the plan must declare it' }
$env:QRUST_HOME = Join-Path $env:MG_TEST_CACHE_RUST 'qrust'
$env:QRUST_TOOLCHAIN_AUTO_UPDATE = 'off'
$env:QRUST_SELF_UPDATE_CHECK = 'off'
$env:CARGO_TERM_COLOR = 'never'
$qrustBin = Join-Path $env:QRUST_HOME 'bin'
$gitHome = Join-Path $env:MG_TEST_CACHE_TOOLS 'mingit-2.56.1'
$env:PATH = "$qrustBin;$gitHome\cmd;$env:PATH"

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

# The release channel's latest QuickRust for Windows, from the release manifest, checked against
# the manifest's SHA-256 and size before anything in it runs; once installed, QuickRust updates
# itself.
if (Test-Path -LiteralPath (Join-Path $qrustBin 'qrust.exe')) {
    Invoke-Native 'updating QuickRust' { qrust self update }
} else {
    $manifest = Invoke-RestMethod 'https://release.machinefabric.com/quickrust/manifest'
    $release = $manifest.channels.release
    $build = $release.versions.($release.latestVersion).platforms.'windows-x86_64'
    if (-not $build) { throw "QuickRust $($release.latestVersion) has no windows-x86_64 build" }
    $zips = @($build.installers | Where-Object { $_.type -eq 'zip' })
    if ($zips.Count -ne 1) { throw "the QuickRust manifest lists $($zips.Count) zip archives for windows-x86_64 $($release.latestVersion), not one" }
    $archive = $zips[0]
    $unpacked = Join-Path $env:MG_TEST_SCRATCH 'quickrust-install'
    if (Test-Path -LiteralPath $unpacked) { Remove-Item -LiteralPath $unpacked -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $unpacked | Out-Null
    $zip = Join-Path $unpacked 'qrust.zip'
    Write-Host "installing QuickRust from $($archive.path)"
    Invoke-WebRequest $archive.path -OutFile $zip
    $size = (Get-Item -LiteralPath $zip).Length
    if ($size -ne [int64] $archive.size) { throw "$($archive.path) is $size bytes, not the $($archive.size) the manifest says" }
    $sha256 = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sha256 -ne $archive.sha256.ToLowerInvariant()) { throw "$($archive.path) has SHA-256 $sha256, not the manifest's $($archive.sha256)" }
    Expand-Archive -LiteralPath $zip -DestinationPath $unpacked
    Invoke-Native 'qrust setup' { & (Join-Path $unpacked 'quickrust\qrust.exe') setup --no-modify-path }
    Remove-Item -LiteralPath $unpacked -Recurse -Force
}
$cargoOnPath = (Get-Command cargo -ErrorAction SilentlyContinue).Source
if ($cargoOnPath -notlike "$qrustBin\*") { throw "cargo on PATH is '$cargoOnPath', not QuickRust's" }
Invoke-Native 'updating the stable toolchain' { qrust toolchain update --channel stable }

# The version tests make git repositories, and the build reads the commit it is built from.
if (-not (Test-Path "$gitHome\cmd\git.exe")) {
    $zip = Join-Path $env:MG_TEST_SCRATCH 'MinGit-2.56.1-64-bit.zip'
    Invoke-WebRequest 'https://github.com/git-for-windows/git/releases/download/v2.56.1.windows.1/MinGit-2.56.1-64-bit.zip' -OutFile $zip
    $sha256 = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLowerInvariant()
    if ($sha256 -ne '4a92850b6e917da4fd010669b614b55dccf5b0ec333a534cc4be4700dfefdafd') {
        throw "MinGit-2.56.1-64-bit.zip has SHA-256 $sha256, not the one Git for Windows published"
    }
    Expand-Archive $zip -DestinationPath $gitHome
}
