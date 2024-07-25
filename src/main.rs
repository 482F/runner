use serde::Deserialize;
use std::env;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct Setting {
    cmd: String,
}

fn main() {
    let exe_path = env::current_exe().unwrap();
    let exe_name = exe_path
        .file_stem()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap();
    let toml_path = exe_path
        .parent()
        .unwrap()
        .to_owned()
        .join(exe_name + ".toml");

    let raw_toml = &fs::read_to_string(&toml_path).unwrap();
    let setting: Setting = toml::from_str(&raw_toml).unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    let (script_name, get_command): (&str, fn(&str) -> Command) = if cfg!(target_os = "windows") {
        ("temp.ps1", |script_path| {
            let mut command = Command::new("powershell.exe");
            command.args(["-ExecutionPolicy", "Bypass", "-File", script_path]);
            command
        })
    } else {
        ("temp.sh", |script_path| {
            let mut command = Command::new("bash");
            command.args([script_path]);
            command
        })
    };

    let script_path = temp_path.join(script_name);
    let mut script = "".to_owned();
    script.push_str(r#"
        # from https://stackoverflow.com/a/74976541
        function Hide-ConsoleWindow() {
          $ShowWindowAsyncCode = '[DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr hWnd, int nCmdShow);'
          $ShowWindowAsync = Add-Type -MemberDefinition $ShowWindowAsyncCode -name Win32ShowWindowAsync -namespace Win32Functions -PassThru

          $hwnd = (Get-Process -PID $pid).MainWindowHandle
          if ($hwnd -ne [System.IntPtr]::Zero) {
            # When you got HWND of the console window:
            # (It would appear that Windows Console Host is the default terminal application)
            $ShowWindowAsync::ShowWindowAsync($hwnd, 0)
          } else {
            # When you failed to get HWND of the console window:
            # (It would appear that Windows Terminal is the default terminal application)

            # Mark the current console window with a unique string.
            $UniqueWindowTitle = New-Guid
            $Host.UI.RawUI.WindowTitle = $UniqueWindowTitle
            $StringBuilder = New-Object System.Text.StringBuilder 1024

            # Search the process that has the window title generated above.
            $TerminalProcess = (Get-Process | Where-Object { $_.MainWindowTitle -eq $UniqueWindowTitle })
            # Get the window handle of the terminal process.
            # Note that GetConsoleWindow() in Win32 API returns the HWND of
            # powershell.exe itself rather than the terminal process.
            # When you call ShowWindowAsync(HWND, 0) with the HWND from GetConsoleWindow(),
            # the Windows Terminal window will be just minimized rather than hidden.
            $hwnd = $TerminalProcess.MainWindowHandle
            if ($hwnd -ne [System.IntPtr]::Zero) {
              $ShowWindowAsync::ShowWindowAsync($hwnd, 0)
            } else {
              Write-Host "Failed to hide the console window."
            }
          }
        }
        Hide-ConsoleWindow
    "#);
    script.push_str(&setting.cmd);
    fs::write(&script_path, &script).unwrap();
    let mut command = get_command(script_path.to_str().unwrap());
    command.spawn().unwrap().wait().unwrap();
}
