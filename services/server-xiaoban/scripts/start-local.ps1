$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$postgresRoot = Join-Path $env:LOCALAPPDATA 'WallpaperCollect/postgresql'
$pgCtl = Join-Path $postgresRoot 'runtime/bin/pg_ctl.exe'
$dataDir = Join-Path $postgresRoot 'data'
$serverExe = Join-Path $projectRoot 'target/debug/wallpaper_collect.exe'

if (!(Test-Path $pgCtl) -or !(Test-Path (Join-Path $dataDir 'PG_VERSION'))) {
    throw 'Local PostgreSQL is not configured in %LOCALAPPDATA%/WallpaperCollect/postgresql.'
}
if (!(Test-Path $serverExe)) {
    throw 'Build the frontend with npm ci and npm run build in web/, then run cargo build in the project root.'
}

& $pgCtl -D $dataDir status *> $null
if ($LASTEXITCODE -ne 0) {
    & $pgCtl -D $dataDir -l (Join-Path $postgresRoot 'postgresql.log') -w start
    if ($LASTEXITCODE -ne 0) { throw 'PostgreSQL failed to start. Check postgresql.log.' }
}

$serverProcess = Get-CimInstance Win32_Process -Filter "name = 'wallpaper_collect.exe'" |
    Where-Object { $_.ExecutablePath -eq $serverExe } | Select-Object -First 1
if (!$serverProcess) {
    $serverProcess = Start-Process -FilePath $serverExe -WorkingDirectory $projectRoot -WindowStyle Hidden `
        -RedirectStandardOutput (Join-Path $projectRoot 'target/wallpaper-server.log') `
        -RedirectStandardError (Join-Path $projectRoot 'target/wallpaper-server-error.log') -PassThru
}
$serverPid = if ($serverProcess -is [System.Diagnostics.Process]) { $serverProcess.Id } else { $serverProcess.ProcessId }
$serverPid | Set-Content -LiteralPath (Join-Path $projectRoot 'target/wallpaper-server.pid')

$serverPort = 8080
foreach ($line in Get-Content -LiteralPath (Join-Path $projectRoot '.env')) {
    if ($line -match '^SERVER_PORT=(\d+)$') { $serverPort = [int]$Matches[1] }
}
$siteUrl = "http://127.0.0.1:$serverPort"
$healthy = $false
for ($attempt = 0; $attempt -lt 30; $attempt++) {
    try {
        $health = Invoke-RestMethod "$siteUrl/health" -TimeoutSec 2
        if ($health.status -eq 'healthy' -and $health.database.postgres_connected) {
            $healthy = $true
            break
        }
    } catch { }
    Start-Sleep -Milliseconds 500
}
if (!$healthy) { throw 'The website did not connect to PostgreSQL. Check target/wallpaper-server-error.log and target/wallpaper-server.log.' }
Write-Output "Website: $siteUrl"
Write-Output "Admin: $siteUrl/admin"
Write-Output 'PostgreSQL is connected.'
