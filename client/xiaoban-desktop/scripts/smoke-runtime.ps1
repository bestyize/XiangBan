param([string]$ExecutablePath, [switch]$VerifyDownload)
$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskExe = if ($ExecutablePath) { [IO.Path]::GetFullPath($ExecutablePath) } else { Join-Path $taskRoot 'src-tauri/target/debug/xiaoban-desktop.exe' }
if (!(Test-Path -LiteralPath $taskExe)) { throw 'Run cargo build --manifest-path src-tauri/Cargo.toml --locked first. This test requires a debug build.' }
$taskArtifacts = Join-Path $taskRoot 'artifacts'
New-Item -ItemType Directory -Path $taskArtifacts -Force | Out-Null
$taskReport = Join-Path $taskArtifacts 'runtime-ui-smoke.json'
Remove-Item -LiteralPath $taskReport -ErrorAction SilentlyContinue
$taskDetection = (& $taskExe --check-runtime | Out-String) | ConvertFrom-Json
if ($taskDetection.windowsMajor -lt 10) { throw 'This test requires Windows 10 or newer.' }
Write-Output ('Detected runtime ready: ' + $taskDetection.ready + '. Native UI simulations do not require WebView2.')
$env:XIAOBAN_RUNTIME_REPORT = $taskReport
try {
    $taskArguments = @('--runtime-ui-smoke')
    if ($VerifyDownload) { $taskArguments += '--verify-runtime-download' }
    $taskProcess = Start-Process -FilePath $taskExe -ArgumentList $taskArguments -PassThru -WindowStyle Hidden
    $taskTimeout = if ($VerifyDownload) { 180000 } else { 20000 }
    if (!$taskProcess.WaitForExit($taskTimeout)) {
        Stop-Process -Id $taskProcess.Id
        throw 'Native preparation dialog smoke test timed out'
    }
    if (!(Test-Path -LiteralPath $taskReport)) { throw 'No report was generated. Use a debug build.' }
    $taskResult = Get-Content -LiteralPath $taskReport -Raw | ConvertFrom-Json
    if (!$taskResult.success -or !$taskResult.failure -or !$taskResult.cancelled) { throw 'A native success, failure or cancellation check failed.' }
    if ($VerifyDownload -and $taskResult.download.Ok -ne 'ok') { throw ('Official download or Microsoft signature validation failed: ' + ($taskResult.download | ConvertTo-Json -Compress)) }
    Write-Output ('PASS: actual runtime detection and native preparation success/failure/cancellation. Report: ' + $taskReport)
    if ($VerifyDownload) { Write-Output 'PASS: embedded downloader, process cleanup and Microsoft signature validation; no installation performed.' }
} finally {
    Remove-Item Env:XIAOBAN_RUNTIME_REPORT -ErrorAction SilentlyContinue
}
