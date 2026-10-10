param([string]$ExecutablePath)
$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskExe = if ($ExecutablePath) { [IO.Path]::GetFullPath($ExecutablePath) } else { Join-Path $taskRoot 'src-tauri/target/release/xiaoban-desktop-wallpaper.exe' }
if (!(Test-Path -LiteralPath $taskExe)) { throw '请先运行 npm run desktop:build' }
$taskArtifacts = Join-Path $taskRoot 'artifacts'
New-Item -ItemType Directory -Path $taskArtifacts -Force | Out-Null
$taskReport = Join-Path $taskArtifacts ('smoke-' + [DateTime]::Now.ToString('yyyyMMdd-HHmmss') + '.json')
$env:XIAOBAN_SMOKE_REPORT = $taskReport
$env:XIAOBAN_TEST_STARTUP_ID = [DateTime]::UtcNow.Ticks.ToString()
$taskStartupName = 'XiaoBanStartupSmoke-' + $env:XIAOBAN_TEST_STARTUP_ID
try {
    $taskProcess = Start-Process -FilePath $taskExe -ArgumentList '--smoke-test', '--autostart' -PassThru -WindowStyle Hidden
    if (!$taskProcess.WaitForExit(45000)) {
        Stop-Process -Id $taskProcess.Id
        throw '原型在 45 秒内未完成模式切换'
    }
    if (!(Test-Path -LiteralPath $taskReport)) { throw '程序未生成原生窗口验证报告' }
    $taskResult = Get-Content -LiteralPath $taskReport -Raw | ConvertFrom-Json
    if (!$taskResult.lifecycle.Ok) { throw ('Lifecycle test failed: ' + ($taskResult.lifecycle | ConvertTo-Json -Compress)) }
    if ($taskResult.lifecycle.Ok.initialControl.visible -or $taskResult.lifecycle.Ok.afterClose.visible) { throw 'Control must stay hidden during autostart and after close.' }
    if (!$taskResult.lifecycle.Ok.afterRestore.visible -or !$taskResult.lifecycle.Ok.secondInstance.visible) { throw 'Control did not reopen from the tray handler or second instance.' }
    if (!$taskResult.lifecycle.Ok.autostartToggleAndPersistence -or !$taskResult.lifecycle.Ok.petPreserved) { throw 'Startup or pet persistence check failed.' }
    if ($taskResult.wallpaperResult.Ok -ne 'ok') { throw ('壁纸模式失败: ' + ($taskResult.wallpaperResult | ConvertTo-Json -Compress)) }
    if ($taskResult.floatingResult.Ok -ne 'ok') { throw '无法从壁纸切回悬浮模式' }
    if (!$taskResult.floatingBefore.floating.visible -or !$taskResult.floatingAfter.floating.visible) { throw '悬浮窗口的前端没有正常显示' }
    $taskPetBefore = $taskResult.floatingBefore.floating
    $taskPetDesktop = $taskResult.wallpaper.floating
    $taskPetAfter = $taskResult.floatingAfter.floating
    if (!$taskPetDesktop.visible -or !$taskPetDesktop.frontendReady) { throw '融入壁纸时小伴未显示' }
    if (!$taskPetBefore.topmost -or $taskPetDesktop.topmost -or !$taskPetAfter.topmost) { throw '两种模式的置顶状态错误' }
    foreach ($taskPet in @($taskPetDesktop, $taskPetAfter)) {
        if ($taskPet.hwnd -ne $taskPetBefore.hwnd -or $taskPet.x -ne $taskPetBefore.x -or $taskPet.y -ne $taskPetBefore.y) { throw '切换模式未保留小伴窗口或位置' }
    }
    if ($taskResult.wallpaper.wallpaper -or $taskResult.floatingAfter.wallpaper) { throw '仍然创建了特殊房间壁纸' }
    Write-Output ('PASS: 两种模式共用透明小伴窗口，位置保持一致，置顶状态正确。报告: ' + $taskReport)
} finally {
    foreach ($taskRegistryPath in @('Software\Microsoft\Windows\CurrentVersion\Run', 'Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run')) {
        $taskRegistryKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($taskRegistryPath, $true)
        if ($taskRegistryKey) {
            try { $taskRegistryKey.DeleteValue($taskStartupName, $false) } finally { $taskRegistryKey.Dispose() }
        }
    }
    Remove-Item Env:XIAOBAN_TEST_STARTUP_ID -ErrorAction SilentlyContinue
    Remove-Item Env:XIAOBAN_SMOKE_REPORT -ErrorAction SilentlyContinue
}
