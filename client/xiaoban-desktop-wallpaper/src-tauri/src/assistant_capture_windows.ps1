$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -ReferencedAssemblies System.Drawing.dll -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class CaptureDpi {
  [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr context);
  [DllImport("user32.dll")] static extern bool SetProcessDPIAware();
  public static void Init() {
    try { if (SetProcessDpiAwarenessContext(new IntPtr(-4))) return; } catch (EntryPointNotFoundException) { }
    SetProcessDPIAware();
  }
}
'@
[CaptureDpi]::Init()
$codec = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() | Where-Object { $_.MimeType -eq 'image/jpeg' } | Select-Object -First 1
$frames = @()
$total = 0
foreach ($screen in [System.Windows.Forms.Screen]::AllScreens) {
  $bounds = $screen.Bounds
  $bitmap = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  try {
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try { $graphics.CopyFromScreen($bounds.Left, $bounds.Top, 0, 0, $bounds.Size) } finally { $graphics.Dispose() }
    $stream = New-Object System.IO.MemoryStream
    $parameters = New-Object System.Drawing.Imaging.EncoderParameters(1)
    try {
      $parameters.Param[0] = New-Object System.Drawing.Imaging.EncoderParameter([System.Drawing.Imaging.Encoder]::Quality, [long]90)
      $bitmap.Save($stream, $codec, $parameters)
      $total += $stream.Length
      if ($total -gt 32 * 1024 * 1024) { throw 'Desktop snapshot exceeds memory limit' }
      $frames += @{ x=$bounds.X; y=$bounds.Y; width=$bounds.Width; height=$bounds.Height; image=('data:image/jpeg;base64,' + [Convert]::ToBase64String($stream.ToArray())) }
    } finally { $parameters.Dispose(); $stream.Dispose() }
  } finally { $bitmap.Dispose() }
}
ConvertTo-Json -InputObject @($frames) -Compress -Depth 3
