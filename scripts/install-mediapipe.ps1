# Создаёт локальный Python 3.11 и ставит mediapipe (если py -3 — это 3.13 без колёс).
# Запуск из корня репозитория: .\scripts\install-mediapipe.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$venv = Join-Path $root ".venv-mediapipe"

$pyLauncher = $null
foreach ($ver in @("3.11", "3.12", "3.10")) {
    $try = & py "-$ver" -c "import sys; print(sys.executable)" 2>$null
    if ($LASTEXITCODE -eq 0 -and $try) {
        $pyLauncher = @("py", "-$ver")
        Write-Host "Найден: py -$ver -> $try"
        break
    }
}
if (-not $pyLauncher) {
    Write-Error "Нужен Python 3.10–3.12 (py -3.11). Установите с https://www.python.org/downloads/windows/"
}

& $pyLauncher[0] $pyLauncher[1] -m venv $venv
$pip = Join-Path $venv "Scripts\pip.exe"
& $pip install -r (Join-Path $root "python\mediapipe-requirements.txt")

$python = Join-Path $venv "Scripts\python.exe"
Write-Host ""
Write-Host "Готово. Задайте переменную пользователя или в сессии:"
Write-Host "  `$env:OSCONTROL_PYTHON = '$python'"
Write-Host "и перезапустите OSControlAssistant."
