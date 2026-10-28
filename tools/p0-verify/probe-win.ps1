# Windows 探测（UI Automation）：目标应用的输入框能否被找到，以及“激活并粘贴”要多久。
# 用法：powershell -ExecutionPolicy Bypass -File probe-win.ps1 -Name Claude -Mode tree|paste
# 没在真机上跑过；出错请把完整报错原样带回来。目标应用若以管理员身份运行，本脚本也要以管理员运行才能操作。
param([string]$Name = "Claude", [string]$Mode = "tree")
[Console]::OutputEncoding = [Text.Encoding]::UTF8
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class Win32 { [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h); [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c); }
"@
$proc = Get-Process -Name $Name -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $proc) { @{ ok = $false; error = "没有名为 $Name 的带窗口进程（先打开它，并打开一个对话页）" } | ConvertTo-Json -Compress; exit 0 }
$out = @{ ok = $true; name = $Name; pid = $proc.Id; title = $proc.MainWindowTitle }
$sw = [Diagnostics.Stopwatch]::StartNew()

if ($Mode -eq "tree") {
  $root = [Windows.Automation.AutomationElement]::FromHandle($proc.MainWindowHandle)
  $edit = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Edit)
  $doc = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Document)
  $any = New-Object Windows.Automation.OrCondition($edit, $doc)
  # Chromium 在检测到有 UIA 客户端后才会构建完整的树，第一次常常是空的，所以重试几次。
  $found = $null
  for ($i = 0; $i -lt 4; $i++) {
    $found = $root.FindAll([Windows.Automation.TreeScope]::Descendants, $any)
    if ($found.Count -gt 0) { break }
    Start-Sleep -Milliseconds 800
  }
  $items = @()
  foreach ($e in $found) {
    $c = $e.Current
    $items += @{ type = $c.ControlType.ProgrammaticName; name = $c.Name; class = $c.ClassName; focusable = $c.IsKeyboardFocusable; hasFocus = $c.HasKeyboardFocus }
    if ($items.Count -ge 20) { break }
  }
  $out.candidateCount = $found.Count
  $out.candidates = $items
  $out.ms = $sw.ElapsedMilliseconds
  $out | ConvertTo-Json -Depth 4 -Compress
  exit 0
}

# paste：保存剪贴板 → 写入测试文字 → 激活应用 → Ctrl+V → 恢复剪贴板。不按回车。
$old = Get-Clipboard -Raw -ErrorAction SilentlyContinue
$text = "P0-粘贴测试 " + (Get-Date -Format o)
Set-Clipboard -Value $text
$sw.Restart()
[Win32]::ShowWindow($proc.MainWindowHandle, 9) | Out-Null
[Win32]::SetForegroundWindow($proc.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 150
[System.Windows.Forms.SendKeys]::SendWait("^v")
$out.activateAndPasteMs = $sw.ElapsedMilliseconds
Start-Sleep -Milliseconds 300
if ($null -ne $old) { Set-Clipboard -Value $old } else { Set-Clipboard -Value " " }
$out.pastedText = $text
$out.note = "请肉眼确认：这行文字是否出现在对方应用的输入框里（没有被发送）"
$out | ConvertTo-Json -Depth 4 -Compress
