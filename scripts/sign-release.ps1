# Подпись release-сборки Authenticode (снижает блокировки SmartScreen / «неизвестный издатель»).
#
# Требуется сертификат от публичного УЦ (DigiCert, Sectigo и т.д.) или корпоративный код-сайнинг.
# Самоподписанный сертификат на чужих ПК не устранит жёлтое окно — только установка корня в «Доверенные».
#
# Установка Windows SDK (есть signtool): https://developer.microsoft.com/windows/downloads/windows-sdk/
#
# Пример (PFX от УЦ):
#   $env:OSCA_PFX = "C:\certs\codesign.pfx"
#   $env:OSCA_PFX_PASSWORD = "***"
#   .\scripts\sign-release.ps1
#
# Пример (сертификат в хранилище Windows, отпечаток SHA1):
#   $env:OSCA_THUMBPRINT = "A1B2C3..."
#   .\scripts\sign-release.ps1

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$exe = Join-Path $repoRoot "target\release\OSControlAssistant.exe"

if (-not (Test-Path $exe)) {
    Write-Error "Не найден $exe — сначала: cargo build --release"
}

function Find-SignTool {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe",
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe"
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }
    $root = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    if (Test-Path $root) {
        $found = Get-ChildItem -Path $root -Recurse -Filter "signtool.exe" -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' } |
            Sort-Object FullName -Descending |
            Select-Object -First 1
        if ($found) { return $found.FullName }
    }
    return $null
}

$signtool = Find-SignTool
if (-not $signtool) {
    Write-Error "signtool.exe не найден. Установите Windows SDK или поправьте путь в скрипте."
}

Write-Host "signtool: $signtool"
Write-Host "Файл:   $exe"

$args = @(
    "sign",
    "/fd", "sha256",
    "/td", "sha256",
    "/tr", "http://timestamp.digicert.com"
)

if ($env:OSCA_THUMBPRINT) {
    $args += "/sha1", $env:OSCA_THUMBPRINT
} elseif ($env:OSCA_PFX) {
    if (-not $env:OSCA_PFX_PASSWORD) {
        Write-Error "Задайте OSCA_PFX_PASSWORD для PFX."
    }
    $args += "/f", $env:OSCA_PFX
    $args += "/p", $env:OSCA_PFX_PASSWORD
} else {
    Write-Error "Задайте OSCA_PFX + OSCA_PFX_PASSWORD или OSCA_THUMBPRINT (сертификат в хранилище)."
}

$args += $exe

& $signtool @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Проверка подписи..."
& $signtool verify /pa $exe
exit $LASTEXITCODE
