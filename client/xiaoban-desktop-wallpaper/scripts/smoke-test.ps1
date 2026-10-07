param([string]$ExecutablePath)
$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskExe = if ($ExecutablePath) { [IO.Path]::GetFullPath($ExecutablePath) } else { Join-Path $taskRoot 'src-tauri/target/release/xiaoban-desktop-wallpaper.exe' }
if (!(Test-Path -LiteralPath $taskExe)) { throw '请先运行 npm run desktop:build' }
$taskArtifacts = Join-Path $taskRoot 'artifacts'
New-Item -ItemType Directory -Path $taskArtifacts -Force | Out-Null
$taskReport = Join-Path $taskArtifacts ('smoke-' + [DateTime]::Now.ToString('yyyyMMdd-HHmmss') + '.json')
$env:XIAOBAN_SMOKE_REPORT = $taskReport
try {
    $taskProcess = Start-Process -FilePath $taskExe -ArgumentList '--smoke-test' -PassThru -WindowStyle Hidden
    if (!$taskProcess.WaitForExit(45000)) {
        Stop-Process -Id $taskProcess.Id
        throw '原型在 45 秒内未完成模式切换'
    }
    if (!(Test-Path -LiteralPath $taskReport)) { throw '程序未生成原生窗口验证报告' }
    $taskResult = Get-Content -LiteralPath $taskReport -Raw | ConvertFrom-Json
    if ($taskResult.wallpaperResult.Ok -ne 'ok') { throw ('壁纸模式失败: ' + ($taskResult.wallpaperResult | ConvertTo-Json -Compress)) }
    if ($taskResult.floatingResult.Ok -ne 'ok') { throw '无法从壁纸切回悬浮模式' }
    if (!$taskResult.floatingBefore.floating.visible -or !$taskResult.floatingAfter.floating.visible) { throw '悬浮窗口的前端没有正常显示' }
    if (!$taskResult.wallpaper.wallpaper.isChild -or $taskResult.wallpaper.wallpaper.parentClass -ne 'WorkerW') { throw '壁纸没有挂到 Explorer 的背景窗口' }
    if (!$taskResult.wallpaper.wallpaper.visible) { throw '壁纸前端没有正常显示' }
    if ($taskResult.wallpaper.floating -or $taskResult.floatingAfter.wallpaper) { throw '切换模式后残留了旧窗口' }
    Write-Output ('PASS: 悬浮 → 壁纸（WorkerW 子窗口）→ 悬浮，三个阶段的前端均已显示。报告: ' + $taskReport)
} finally {
    Remove-Item Env:XIAOBAN_SMOKE_REPORT -ErrorAction SilentlyContinue
}
