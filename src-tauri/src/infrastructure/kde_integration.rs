#[cfg(target_os = "linux")]
mod linux {
    use std::{env, path::Path, process::Command};

    use tauri::{path::BaseDirectory, AppHandle, Manager};

    const PLUGIN_ID: &str = "kairo-keep-above";
    const RESOURCE_PACKAGE_DIR: &str = "kde/kwin/kairo-keep-above";

    pub fn setup(app: &AppHandle) {
        if !is_kde_wayland_session() {
            return;
        }

        if let Err(error) = install_keep_above_script(app) {
            eprintln!("failed to prepare KDE keep-above integration: {error}");
        }
    }

    fn install_keep_above_script(app: &AppHandle) -> Result<(), String> {
        ensure_required_commands()?;

        let package_dir = app
            .path()
            .resolve(RESOURCE_PACKAGE_DIR, BaseDirectory::Resource)
            .map_err(|error| format!("failed to resolve bundled KWin script: {error}"))?;

        if !package_dir.join("metadata.json").is_file() {
            return Err(format!(
                "bundled KWin script metadata not found at `{}`",
                package_dir.display()
            ));
        }

        install_or_upgrade_package(&package_dir)?;
        enable_package()?;
        reload_kwin()?;
        warn_if_script_is_not_loaded();

        Ok(())
    }

    fn is_kde_wayland_session() -> bool {
        is_wayland_session() && is_kde_session()
    }

    fn is_wayland_session() -> bool {
        env_value("XDG_SESSION_TYPE") == "wayland" || env::var_os("WAYLAND_DISPLAY").is_some()
    }

    fn is_kde_session() -> bool {
        let current_desktop = env_value("XDG_CURRENT_DESKTOP");
        let desktop_session = env_value("DESKTOP_SESSION");

        current_desktop.contains("kde")
            || current_desktop.contains("plasma")
            || desktop_session.contains("kde")
            || desktop_session.contains("plasma")
    }

    fn env_value(key: &str) -> String {
        env::var(key).unwrap_or_default().to_lowercase()
    }

    fn ensure_required_commands() -> Result<(), String> {
        for command in ["kpackagetool6", "kwriteconfig6", "qdbus-qt6"] {
            if !command_exists(command) {
                return Err(format!("required KDE command `{command}` was not found"));
            }
        }

        Ok(())
    }

    fn command_exists(command: &str) -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {command} >/dev/null 2>&1"))
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn install_or_upgrade_package(package_dir: &Path) -> Result<(), String> {
        let installed_scripts = run_command("kpackagetool6", &["--type=KWin/Script", "--list"])?;
        let action = if installed_scripts.contains(PLUGIN_ID) {
            "--upgrade"
        } else {
            "--install"
        };
        let package_dir = package_dir.to_string_lossy().into_owned();

        run_command(
            "kpackagetool6",
            &["--type=KWin/Script", action, &package_dir],
        )?;

        Ok(())
    }

    fn enable_package() -> Result<(), String> {
        run_command(
            "kwriteconfig6",
            &[
                "--file",
                "kwinrc",
                "--group",
                "Plugins",
                "--key",
                "kairo-keep-aboveEnabled",
                "true",
            ],
        )?;

        Ok(())
    }

    fn reload_kwin() -> Result<(), String> {
        run_command("qdbus-qt6", &["org.kde.KWin", "/KWin", "reconfigure"])?;
        Ok(())
    }

    fn warn_if_script_is_not_loaded() {
        match run_command(
            "qdbus-qt6",
            &[
                "org.kde.KWin",
                "/Scripting",
                "org.kde.kwin.Scripting.isScriptLoaded",
                PLUGIN_ID,
            ],
        ) {
            Ok(output) if output.trim() == "true" => {}
            Ok(output) => eprintln!(
                "KDE keep-above integration was installed, but KWin reports loaded=`{}`",
                output.trim()
            ),
            Err(error) => {
                eprintln!("failed to verify KDE keep-above script load state: {error}");
            }
        }
    }

    fn run_command(command: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|error| format!("failed to run `{command}`: {error}"))?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        Err(format!(
            "`{}` failed with status {}: {}",
            command,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(target_os = "linux")]
pub fn setup(app: &tauri::AppHandle) {
    linux::setup(app);
}

#[cfg(not(target_os = "linux"))]
pub fn setup(_app: &tauri::AppHandle) {}
