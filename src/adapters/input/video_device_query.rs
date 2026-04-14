use std::process::Command;

use nokhwa::utils::ApiBackend;
use nokhwa::query;

/// Список имён видеоустройств для выбора камеры жестов.
pub fn list_video_inputs() -> Vec<String> {
    let mut devices = vec!["Камера по умолчанию".to_owned()];

    if let Ok(video_devices) = query(ApiBackend::Auto) {
        devices.extend(
            video_devices
                .into_iter()
                .map(|camera| camera.human_name())
                .collect::<Vec<_>>(),
        );
    }

    devices.extend(run_powershell_lines(
        r#"
Get-CimInstance Win32_PnPEntity |
Where-Object {
    $_.Status -eq 'OK' -and (
        $_.PNPClass -eq 'Camera' -or
        $_.Name -match '(?i)camera|webcam|камера'
    )
} |
Select-Object -ExpandProperty Name -Unique
"#,
    ));

    dedupe(devices)
}

fn run_powershell_lines(script: &str) -> Vec<String> {
    match Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
    {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    for item in items {
        if !result.iter().any(|existing| existing == &item) {
            result.push(item);
        }
    }

    result
}
