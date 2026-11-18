param(
    [Parameter(Mandatory = $true)][string]$Output,
    [Parameter(Mandatory = $true)][string]$CertificateThumbprint
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Invoke-Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}

function Write-Logo($Source, [int]$Size, [string]$Destination) {
    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.DrawImage($Source, 0, 0, $Size, $Size)
        $bitmap.Save($Destination, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

if ($CertificateThumbprint -notmatch '^[a-fA-F0-9]{40}$') { throw 'Invalid certificate thumbprint.' }
$outputPath = [System.IO.Path]::GetFullPath($Output)
if ([System.IO.Path]::GetExtension($outputPath) -ne '.msix' -or (Test-Path -LiteralPath $outputPath)) {
    throw 'Select a new .msix output path.'
}
$cert = Get-Item -LiteralPath "Cert:\CurrentUser\My\$CertificateThumbprint"
if (-not $cert.HasPrivateKey) { throw 'The selected certificate has no signing key.' }
$makeappx = (Get-Command MakeAppx.exe -CommandType Application).Source
$signTool = (Get-Command SignTool.exe -CommandType Application).Source
$dotnet = (Get-Command dotnet.exe -CommandType Application).Source
$temp = Join-Path ([System.IO.Path]::GetTempPath()) ('xiaoban-widget-' + [System.Guid]::NewGuid().ToString('N'))
try {
    $publish = Join-Path $temp 'publish'
    New-Item -ItemType Directory -Path $publish | Out-Null
    $project = Join-Path $PSScriptRoot 'Xiaoban.LockScreenWidget.csproj'
    Invoke-Checked $dotnet @('publish', $project, '-c', 'Release', '-r', 'win-x64',
        '--self-contained', 'true', '-o', $publish)

    [xml]$manifest = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $PSScriptRoot 'AppxManifest.xml')
    $manifest.Package.Identity.SetAttribute('Publisher', $cert.Subject)
    $manifest.Save((Join-Path $publish 'AppxManifest.xml'))

    Add-Type -AssemblyName System.Drawing
    $assets = Join-Path $publish 'Assets'
    $providerAssets = Join-Path $publish 'ProviderAssets'
    New-Item -ItemType Directory -Path $assets, $providerAssets -Force | Out-Null
    $iconPath = Join-Path $PSScriptRoot '..\..\icons\icon.png'
    $icon = [System.Drawing.Image]::FromFile($iconPath)
    try {
        Write-Logo $icon 50 (Join-Path $assets 'StoreLogo.png')
        Write-Logo $icon 150 (Join-Path $assets 'Square150x150Logo.png')
        Write-Logo $icon 44 (Join-Path $assets 'Square44x44Logo.png')
        Write-Logo $icon 64 (Join-Path $providerAssets 'Icon.png')
    } finally {
        $icon.Dispose()
    }

    $staged = Join-Path $temp 'prototype.msix'
    Invoke-Checked $makeappx @('pack', '/d', $publish, '/p', $staged)
    Invoke-Checked $signTool @('sign', '/sha1', $CertificateThumbprint, '/fd', 'SHA256', $staged)
    Invoke-Checked $signTool @('verify', '/pa', $staged)
    Move-Item -LiteralPath $staged -Destination $outputPath
    Write-Output $outputPath
} finally {
    if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Recurse -Force }
}
