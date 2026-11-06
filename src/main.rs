use bingpaper_desktop::app::BingPaper;
use gpui_kit::*;

fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    app.run(|cx| {
        gpui_kit::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1100.), px(720.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("Bing 每日图片".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        open_window(window_options, cx, |_, cx| cx.new(BingPaper::new))
            .expect("Failed to open window");
    });
}
