use std::process::Command;

use anyhow::{Result, anyhow};

pub fn list_screens() -> Result<Vec<String>> {
    let out = Command::new("dms").arg("screenshot").arg("list").output()?;
    let data = String::from_utf8(out.stdout)?;
    let res: Vec<String> = data
        .lines()
        .filter_map(|line| {
            if let Some((screen, _)) = line.split_once(":") {
                Some(screen.into())
            } else {
                None
            }
        })
        .collect();
    Ok(res)
}

pub fn set_picture_for_screen(screen: &str, path: &str) -> Result<()> {
    let status = Command::new("dms")
        .arg("ipc")
        .arg("call")
        .arg("wallpaper")
        .arg("setFor")
        .arg(screen)
        .arg(path)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("status code: {:?}", status.code()))
    }
}
