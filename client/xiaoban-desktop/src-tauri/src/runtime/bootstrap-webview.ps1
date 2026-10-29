param([Parameter(Mandatory = $true)][string]$InstallerPath, [switch]$DownloadOnly)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

try {
    Write-Output 'XIAOBAN_STAGE_DOWNLOAD'
    # Official Evergreen bootstrapper. It chooses the architecture automatically.
    Invoke-WebRequest -UseBasicParsing -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $InstallerPath -TimeoutSec 120
    Write-Output 'XIAOBAN_STAGE_VERIFY'
    $signature = Get-AuthenticodeSignature -LiteralPath $InstallerPath
    if ($signature.Status -ne 'Valid' -or !$signature.SignerCertificate -or
        $signature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false) -ne 'Microsoft Corporation') {
        throw 'The runtime installer does not have a valid Microsoft signature.'
    }
    if ($DownloadOnly) { exit 0 }
    Write-Output 'XIAOBAN_STAGE_INSTALL'
    # No elevation request: WebView2 supports per-user installation.
    $install = Start-Process -FilePath $InstallerPath -ArgumentList '/silent', '/install' -WindowStyle Hidden -PassThru
    if (!$install.WaitForExit(600000)) {
        Stop-Process -Id $install.Id -ErrorAction SilentlyContinue
        throw 'The runtime installer timed out.'
    }
    if ($install.ExitCode -ne 0 -and $install.ExitCode -ne 3010) {
        throw "The runtime installer returned exit code $($install.ExitCode)."
    }
    exit 0
} catch {
    Write-Output ('XIAOBAN_ERROR:' + $_.Exception.Message)
    exit 1
}
