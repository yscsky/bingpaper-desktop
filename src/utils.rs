use std::env;

const HOME: &str = "BING_PAPER_HOME";

pub fn get_home() -> String {
    env::var(HOME).expect("You may export BING_PAPER_HOME environment")
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
