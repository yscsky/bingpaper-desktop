use std::env;

use anyhow::{Result, anyhow};

const HOME: &str = "BING_PAPER_HOME";

pub fn try_get_home() -> Result<String> {
    env::var(HOME).map_err(|_| anyhow!("请先设置 BING_PAPER_HOME 环境变量"))
}

pub fn get_home() -> String {
    env::var(HOME).expect("You may export BING_PAPER_HOME environment")
}

/// 把 bing 的 `Ssd` 时间戳（如 `20260923`）格式化为 `2026-09-23`。
pub fn format_time(time: &str) -> String {
    let digits = time.trim();
    if digits.len() == 8 && digits.bytes().all(|b| b.is_ascii_digit()) {
        format!(
            "{}-{}-{}",
            &digits[0..4],
            &digits[4..6],
            &digits[6..8]
        )
    } else {
        digits.to_string()
    }
}

pub fn convert_url(s: String) -> String {
    if let Some((frist, _)) = s.split_once('&') {
        frist.replace("1920x1200", "UHD")
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_home() {
        assert_eq!(get_home(), "/data/pictures/WallPapers");
    }

    #[test]
    fn test_convert_url() {
        assert_eq!(
            convert_url(
                "/th?id=OHR.ArmbrugBridge_ZH-CN4600005169_1920x1200.jpg&rf=LaDigue_1920x1200.jpg"
                    .into()
            ),
            "/th?id=OHR.ArmbrugBridge_ZH-CN4600005169_UHD.jpg"
        );

        assert_eq!(
            convert_url("/th?id=OHR.WildflowerValley_ZH-CN7048300331_1920x1200.jpg&rf=LaDigue_1920x1200.jpg".into()),
            "/th?id=OHR.WildflowerValley_ZH-CN7048300331_UHD.jpg"
        );

        assert_eq!(
            convert_url(
                "/th?id=OHR.ParadiseCave_ZH-CN6371627729_1920x1200.jpg&rf=LaDigue_1920x1200.jpg"
                    .into()
            ),
            "/th?id=OHR.ParadiseCave_ZH-CN6371627729_UHD.jpg"
        );
    }
}
