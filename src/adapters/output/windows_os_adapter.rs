#[cfg(windows)]
use std::process::Command;

use crate::gesture_os_control::application::ports::output::os_command_port::OsCommandPort;
use crate::gesture_os_control::domain::entities::command::{CommandExecutionResult, OsCommand};

/// Выполнение абстрактных команд в Windows (PowerShell + user32 через Add-Type).
pub struct WindowsPipelineOsAdapter;

impl WindowsPipelineOsAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: OsCommand) -> CommandExecutionResult {
        #[cfg(windows)]
        {
            self.execute_windows(command)
        }
        #[cfg(not(windows))]
        {
            let _ = command;
            CommandExecutionResult {
                ok: false,
                description: "Windows-адаптер недоступен на этой платформе.".to_owned(),
                system_error: None,
            }
        }
    }

    #[cfg(windows)]
    fn execute_windows(&self, command: OsCommand) -> CommandExecutionResult {
        match command {
            OsCommand::NoAction => CommandExecutionResult {
                ok: true,
                description: "Нет действия.".to_owned(),
                system_error: None,
            },
            OsCommand::LockWorkstation => match Command::new("rundll32.exe").args(["user32.dll,LockWorkStation"]).spawn() {
                Ok(_) => CommandExecutionResult {
                    ok: true,
                    description: "Сеанс Windows заблокирован.".to_owned(),
                    system_error: None,
                },
                Err(error) => CommandExecutionResult {
                    ok: false,
                    description: "Не удалось заблокировать сеанс.".to_owned(),
                    system_error: Some(error.to_string()),
                },
            },
            OsCommand::OpenNotepad => match Command::new("notepad").spawn() {
                Ok(_) => CommandExecutionResult {
                    ok: true,
                    description: "Блокнот запущен.".to_owned(),
                    system_error: None,
                },
                Err(error) => CommandExecutionResult {
                    ok: false,
                    description: "Не удалось запустить Блокнот.".to_owned(),
                    system_error: Some(error.to_string()),
                },
            },
            OsCommand::OpenExplorer => match Command::new("explorer").spawn() {
                Ok(_) => CommandExecutionResult {
                    ok: true,
                    description: "Проводник запущен.".to_owned(),
                    system_error: None,
                },
                Err(error) => CommandExecutionResult {
                    ok: false,
                    description: "Не удалось запустить Проводник.".to_owned(),
                    system_error: Some(error.to_string()),
                },
            },
            other => {
                let (script, ok_msg) = match other {
                    OsCommand::VolumeUp => (AUDIO_VOLUME_UP_SCRIPT.to_owned(), "Громкость увеличена."),
                    OsCommand::VolumeDown => (AUDIO_VOLUME_DOWN_SCRIPT.to_owned(), "Громкость уменьшена."),
                    OsCommand::Mute => (AUDIO_VOLUME_MUTE_SCRIPT.to_owned(), "Mute переключён."),
                    OsCommand::BrightnessUp => (BRIGHTNESS_UP_SCRIPT.to_owned(), "Яркость увеличена."),
                    OsCommand::BrightnessDown => (BRIGHTNESS_DOWN_SCRIPT.to_owned(), "Яркость уменьшена."),
                    OsCommand::SwitchAudioOutput => {
                        (SWITCH_AUDIO_OUTPUT_SCRIPT.to_owned(), "Аудиовыход переключён.")
                    }
                    OsCommand::SwitchNextWindow => (
                        "$wshell = New-Object -ComObject WScript.Shell; Start-Sleep -Milliseconds 80; $wshell.SendKeys('%{TAB}')"
                            .to_owned(),
                        "Переключение окна отправлено.",
                    ),
                    OsCommand::MinimizeAllWindows => (
                        "$shell = New-Object -ComObject Shell.Application; $shell.MinimizeAll()".to_owned(),
                        "Все окна свернуты.",
                    ),
                    OsCommand::BrowserNextTab => (
                        "$wshell = New-Object -ComObject WScript.Shell; Start-Sleep -Milliseconds 80; $wshell.SendKeys('^{TAB}')"
                            .to_owned(),
                        "Следующая вкладка браузера.",
                    ),
                    OsCommand::BrowserPreviousTab => (
                        "$wshell = New-Object -ComObject WScript.Shell; Start-Sleep -Milliseconds 80; $wshell.SendKeys('^+{TAB}')"
                            .to_owned(),
                        "Предыдущая вкладка браузера.",
                    ),
                    OsCommand::NextDesktop => (WIN_CTRL_ARROW_RIGHT.to_owned(), "Следующий виртуальный рабочий стол."),
                    OsCommand::PreviousDesktop => (WIN_CTRL_ARROW_LEFT.to_owned(), "Предыдущий виртуальный рабочий стол."),
                    OsCommand::ScrollUp => (MOUSE_WHEEL_UP.to_owned(), "Прокрутка вверх."),
                    OsCommand::ScrollDown => (MOUSE_WHEEL_DOWN.to_owned(), "Прокрутка вниз."),
                    OsCommand::PlayPause => (MEDIA_PLAY_PAUSE.to_owned(), "Play/Pause."),
                    OsCommand::OpenMenu => (WIN_X_MENU.to_owned(), "Открыто меню Win+X."),
                    OsCommand::ShutdownComputer => (
                        "Stop-Computer -Force".to_owned(),
                        "Команда выключения отправлена.",
                    ),
                    OsCommand::NoAction
                    | OsCommand::LockWorkstation
                    | OsCommand::OpenNotepad
                    | OsCommand::OpenExplorer => unreachable!("уже обработано выше"),
                };

                match run_powershell_status(&script) {
                    Ok(()) => CommandExecutionResult {
                        ok: true,
                        description: ok_msg.to_owned(),
                        system_error: None,
                    },
                    Err(error) => CommandExecutionResult {
                        ok: false,
                        description: "Ошибка выполнения команды в Windows.".to_owned(),
                        system_error: Some(error),
                    },
                }
            }
        }
    }
}

impl Default for WindowsPipelineOsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl OsCommandPort for WindowsPipelineOsAdapter {
    fn execute_command(&self, command: OsCommand) -> CommandExecutionResult {
        self.execute(command)
    }
}

#[cfg(windows)]
fn run_powershell_status(script: &str) -> Result<(), String> {
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .status()
        .map_err(|error| format!("PowerShell: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("PowerShell завершился с кодом: {status}"))
    }
}

const AUDIO_VOLUME_UP_SCRIPT: &str = r#"
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class Keyboard {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);
}
"@
[Keyboard]::keybd_event(0xAF, 0, 0, 0)
[Keyboard]::keybd_event(0xAF, 0, 2, 0)
"#;

const AUDIO_VOLUME_DOWN_SCRIPT: &str = r#"
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class Keyboard {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);
}
"@
[Keyboard]::keybd_event(0xAE, 0, 0, 0)
[Keyboard]::keybd_event(0xAE, 0, 2, 0)
"#;

const AUDIO_VOLUME_MUTE_SCRIPT: &str = r#"
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class Keyboard {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);
}
"@
[Keyboard]::keybd_event(0xAD, 0, 0, 0)
[Keyboard]::keybd_event(0xAD, 0, 2, 0)
"#;

const BRIGHTNESS_UP_SCRIPT: &str = r#"
$monitor = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction Stop | Select-Object -First 1
$current = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1
$target = [Math]::Min($current.CurrentBrightness + 10, 100)
$monitor.WmiSetBrightness(1, $target) | Out-Null
"#;

const BRIGHTNESS_DOWN_SCRIPT: &str = r#"
$monitor = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction Stop | Select-Object -First 1
$current = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1
$target = [Math]::Max($current.CurrentBrightness - 10, 0)
$monitor.WmiSetBrightness(1, $target) | Out-Null
"#;

const SWITCH_AUDIO_OUTPUT_SCRIPT: &str = r#"
if (Get-Module -ListAvailable -Name AudioDeviceCmdlets) {
    Import-Module AudioDeviceCmdlets
} elseif (Test-Path 'C:\Users\Tsukimi\psmodules\AudioDeviceCmdlets\3.1.0.2\AudioDeviceCmdlets.psd1') {
    Import-Module 'C:\Users\Tsukimi\psmodules\AudioDeviceCmdlets\3.1.0.2\AudioDeviceCmdlets.psd1'
} else {
    throw 'Для переключения аудиовывода нужен PowerShell-модуль AudioDeviceCmdlets.'
}
$devices = Get-AudioDevice -List | Where-Object { $_.Type -eq 'Playback' }
$current = Get-AudioDevice -Playback
if (-not $devices -or $devices.Count -lt 2) {
    throw 'Недостаточно доступных устройств воспроизведения для переключения.'
}
$index = [Array]::IndexOf($devices, ($devices | Where-Object { $_.ID -eq $current.ID } | Select-Object -First 1))
if ($index -lt 0) { $index = 0 }
$next = $devices[($index + 1) % $devices.Count]
Set-AudioDevice -ID $next.ID | Out-Null
"#;

const WIN_CTRL_ARROW_RIGHT: &str = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class K {
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtra);
  public const int KEYEVENTF_KEYUP = 2;
}
"@
[K]::keybd_event(0x5B,0,0,0)
[K]::keybd_event(0x11,0,0,0)
[K]::keybd_event(0x27,0,0,0)
[K]::keybd_event(0x27,0,[K]::KEYEVENTF_KEYUP,0)
[K]::keybd_event(0x11,0,[K]::KEYEVENTF_KEYUP,0)
[K]::keybd_event(0x5B,0,[K]::KEYEVENTF_KEYUP,0)
"#;

const WIN_CTRL_ARROW_LEFT: &str = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class K {
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtra);
  public const int KEYEVENTF_KEYUP = 2;
}
"@
[K]::keybd_event(0x5B,0,0,0)
[K]::keybd_event(0x11,0,0,0)
[K]::keybd_event(0x25,0,0,0)
[K]::keybd_event(0x25,0,[K]::KEYEVENTF_KEYUP,0)
[K]::keybd_event(0x11,0,[K]::KEYEVENTF_KEYUP,0)
[K]::keybd_event(0x5B,0,[K]::KEYEVENTF_KEYUP,0)
"#;

const MOUSE_WHEEL_UP: &str = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class M {
  [DllImport("user32.dll")] public static extern void mouse_event(int flags, int dx, int dy, int data, int extra);
  public const int MOUSEEVENTF_WHEEL = 0x800;
}
[M]::mouse_event([M]::MOUSEEVENTF_WHEEL,0,0,120,0)
"#;

const MOUSE_WHEEL_DOWN: &str = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class M {
  [DllImport("user32.dll")] public static extern void mouse_event(int flags, int dx, int dy, int data, int extra);
  public const int MOUSEEVENTF_WHEEL = 0x800;
}
[M]::mouse_event([M]::MOUSEEVENTF_WHEEL,0,0,-120,0)
"#;

const MEDIA_PLAY_PAUSE: &str = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class K {
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtra);
  public const int KEYEVENTF_KEYUP = 2;
}
"@
[K]::keybd_event(0xB3,0,0,0)
[K]::keybd_event(0xB3,0,[K]::KEYEVENTF_KEYUP,0)
"#;

const WIN_X_MENU: &str = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class K {
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtra);
  public const int KEYEVENTF_KEYUP = 2;
}
"@
[K]::keybd_event(0x5B,0,0,0)
[K]::keybd_event(0x58,0,0,0)
[K]::keybd_event(0x58,0,[K]::KEYEVENTF_KEYUP,0)
[K]::keybd_event(0x5B,0,[K]::KEYEVENTF_KEYUP,0)
"#;
