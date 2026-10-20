# BoxMux's suite on a Windows guest: installs what it needs (once per guest or cache), then
# checks lints and runs every test. Each test's outcome goes to out\results.jsonl.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Set-Location $env:MG_TEST_WORKSPACE
. (Join-Path $PSScriptRoot 'windows-toolchain.ps1')
New-Item -ItemType Directory -Force out | Out-Null

$log = Join-Path (Get-Location) 'out\test.log'
Set-Content $log ''
$stage = ''
$status = 0
# Formatting does not depend on the platform; the Linux guests check it.
foreach ($step in @(
        @('clippy', 'cargo clippy --all-targets --locked -- -D warnings'),
        @('tests', 'cargo test --locked --no-fail-fast'))) {
    $stage = $step[0]
    Add-Content $log "== $stage"
    cmd /c "$($step[1]) >> `"$log`" 2>&1"
    $status = $LASTEXITCODE
    if ($status -ne 0 -and $stage -ne 'tests') { break }
}

$out = [System.Collections.Generic.List[string]]::new()
$out.Add('{"record": "run", "version": "boxmux", "host_os": "windows", "mode": "cargo-test"}')
$count = @{ ok = 0; failed = 0; skipped = 0 }
$suite = ''
foreach ($line in Get-Content -Encoding UTF8 $log) {
    if ($line -match '^\s*Running (\S+)') { $suite = $Matches[1] -replace '\\', '/' }
    elseif ($line -match '^test (\S+) \.\.\. (ok|FAILED|ignored)$') {
        $outcome = @{ ok = 'ok'; FAILED = 'failed'; ignored = 'skipped' }[$Matches[2]]
        $out.Add((@{ record = 'item'; group = $suite; item = $Matches[1]; outcome = $outcome } | ConvertTo-Json -Compress))
        $count[$outcome]++
    }
}
if ($stage -ne 'tests') {
    $out.Add((@{ record = 'item'; group = 'checks'; item = $stage; outcome = 'failed'; fault = 'workload'; log = 'out/test.log' } | ConvertTo-Json -Compress))
    $count.failed++
}
$out.Add((@{ record = 'summary'; passed = $count.ok; failed = $count.failed; skipped = $count.skipped } | ConvertTo-Json -Compress))
[System.IO.File]::WriteAllLines((Join-Path (Get-Location) 'out\results.jsonl'), $out)
Get-Content -Encoding UTF8 $log -Tail 40
exit $status
