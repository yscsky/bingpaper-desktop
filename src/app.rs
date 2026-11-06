use std::path::{Path, PathBuf};

use anyhow::anyhow;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    sidebar::{
        Sidebar, SidebarCollapsible, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu,
        SidebarMenuItem,
    },
    spinner::Spinner,
    *,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::{
    Wallpaper, download_picture_to, format_time, list_screens, set_picture_for_screen, try_get_home,
};

const PREVIEW_DIR: &str = "bingpaper-desktop-preview";

enum LoadState {
    Loading,
    Ready,
    Failed(String),
}

enum PreviewState {
    Idle,
    Loading,
    Ready(PathBuf),
    Failed(String),
}

struct Status {
    text: SharedString,
    is_error: bool,
}

pub struct BingPaper {
    wallpapers: Vec<Wallpaper>,
    selected: Option<usize>,
    load: LoadState,
    preview: PreviewState,
    home: Option<String>,
    downloading: bool,
    applying: bool,
    status: Option<Status>,
}

impl BingPaper {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            wallpapers: Vec::new(),
            selected: None,
            load: LoadState::Loading,
            preview: PreviewState::Idle,
            home: try_get_home().ok(),
            downloading: false,
            applying: false,
            status: None,
        };
        this.reload(cx);
        this
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        if matches!(self.load, LoadState::Loading) && !self.wallpapers.is_empty() {
            return;
        }

        self.load = LoadState::Loading;
        self.selected = None;
        self.preview = PreviewState::Idle;
        self.status = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = on_tokio(crate::get_wallpapers()).await;

            this.update(cx, |this, cx| match result {
                Ok(wallpapers) if wallpapers.is_empty() => {
                    this.wallpapers = wallpapers;
                    this.load = LoadState::Failed("Bing 暂未返回图片".into());
                    cx.notify();
                }
                Ok(wallpapers) => {
                    this.wallpapers = wallpapers;
                    this.load = LoadState::Ready;
                    this.select(0, cx);
                }
                Err(error) => {
                    this.load = LoadState::Failed(format!("{error:#}"));
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.selected == Some(index) && !matches!(self.preview, PreviewState::Failed(_)) {
            return;
        }

        let Some(wallpaper) = self.wallpapers.get(index).cloned() else {
            return;
        };

        self.selected = Some(index);
        self.preview = PreviewState::Loading;
        self.status = None;
        cx.notify();

        let directory = preview_dir();
        cx.spawn(async move |this, cx| {
            let result = on_tokio(async move {
                download_picture_to(&directory, &wallpaper).await
            })
            .await;

            this.update(cx, |this, cx| {
                this.preview = match result {
                    Ok(path) => PreviewState::Ready(path),
                    Err(error) => PreviewState::Failed(format!("{error:#}")),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn current(&self) -> Option<&Wallpaper> {
        self.selected.and_then(|index| self.wallpapers.get(index))
    }

    fn download(&mut self, cx: &mut Context<Self>) {
        let Some(wallpaper) = self.current().cloned() else {
            return;
        };
        if self.downloading {
            return;
        }
        let Some(home) = self.home.clone() else {
            self.status = Some(Status {
                text: "未设置 BING_PAPER_HOME，无法下载".into(),
                is_error: true,
            });
            cx.notify();
            return;
        };

        self.downloading = true;
        self.status = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(
                    async move { download_picture_to(Path::new(&home), &wallpaper).await },
                )
                .await;

            this.update(cx, |this, cx| {
                this.downloading = false;
                this.status = Some(match result {
                    Ok(path) => Status {
                        text: format!("已保存到 {}", path.display()).into(),
                        is_error: false,
                    },
                    Err(error) => Status {
                        text: format!("下载失败：{error:#}").into(),
                        is_error: true,
                    },
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn apply_wallpaper(&mut self, cx: &mut Context<Self>) {
        let Some(wallpaper) = self.current().cloned() else {
            return;
        };
        if self.applying {
            return;
        }
        let Some(home) = self.home.clone() else {
            self.status = Some(Status {
                text: "未设置 BING_PAPER_HOME，无法设置壁纸".into(),
                is_error: true,
            });
            cx.notify();
            return;
        };

        self.applying = true;
        self.status = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = on_tokio(async move {
                let path = download_picture_to(Path::new(&home), &wallpaper).await?;
                let screens = list_screens()?;
                let screen = screens
                    .first()
                    .ok_or_else(|| anyhow!("未发现可用屏幕"))?
                    .clone();
                set_picture_for_screen(&screen, &path.to_string_lossy())?;
                Ok::<String, anyhow::Error>(screen)
            })
            .await;

            this.update(cx, |this, cx| {
                this.applying = false;
                this.status = Some(match result {
                    Ok(screen) => Status {
                        text: format!("已设置为 {screen} 的壁纸").into(),
                        is_error: false,
                    },
                    Err(error) => Status {
                        text: format!("设置壁纸失败：{error:#}").into(),
                        is_error: true,
                    },
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let items: Vec<SidebarMenuItem> = self
            .wallpapers
            .iter()
            .enumerate()
            .map(|(index, wallpaper)| {
                SidebarMenuItem::new(format_time(&wallpaper.time))
                    .icon(IconName::Image)
                    .active(self.selected == Some(index))
                    .on_click(cx.listener(move |this, _, _, cx| this.select(index, cx)))
            })
            .collect();

        Sidebar::new("bingpaper-sidebar")
            .collapsible(SidebarCollapsible::None)
            .w_56()
            .header(
                SidebarHeader::new().child(
                    v_flex()
                        .gap_1()
                        .child(div().text_sm().font_bold().child("Bing 每日图片"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("选择日期查看图片"),
                        ),
                ),
            )
            .child(
                SidebarGroup::new("日期").child(SidebarMenu::new().children(items.iter().cloned())),
            )
            .footer(
                SidebarFooter::new().child(
                    Button::new("refresh")
                        .ghost()
                        .small()
                        .icon(IconName::RefreshCcw)
                        .label("刷新")
                        .disabled(matches!(self.load, LoadState::Loading))
                        .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
                ),
            )
    }

    fn render_preview(&self, cx: &mut Context<Self>) -> AnyElement {
        match &self.preview {
            PreviewState::Ready(path) => div()
                .size_full()
                .child(img(path.clone()).object_fit(ObjectFit::Contain).size_full())
                .into_any_element(),
            PreviewState::Loading | PreviewState::Idle => v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap_3()
                .child(Spinner::new().large())
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("正在获取图片…"),
                )
                .into_any_element(),
            PreviewState::Failed(error) => v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap_3()
                .p_6()
                .child(Alert::error("preview-error", error.clone()).title("图片加载失败"))
                .child(
                    Button::new("retry-preview")
                        .outline()
                        .label("重试")
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(index) = this.selected {
                                this.preview = PreviewState::Idle;
                                this.select(index, cx);
                            }
                        })),
                )
                .into_any_element(),
        }
    }

    fn render_metadata(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(wallpaper) = self.current() else {
            return div().into_any_element();
        };

        let actions = h_flex()
            .gap_2()
            .child(
                Button::new("download")
                    .outline()
                    .icon(IconName::Download)
                    .label("下载")
                    .loading(self.downloading)
                    .disabled(self.home.is_none())
                    .when(self.home.is_none(), |this| {
                        this.tooltip("请先设置 BING_PAPER_HOME 环境变量")
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.download(cx))),
            )
            .child(
                Button::new("set-wallpaper")
                    .primary()
                    .icon(IconName::Monitor)
                    .label("设为壁纸")
                    .loading(self.applying)
                    .disabled(self.home.is_none())
                    .when(self.home.is_none(), |this| {
                        this.tooltip("请先设置 BING_PAPER_HOME 环境变量")
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.apply_wallpaper(cx))),
            );

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .items_end()
                    .justify_between()
                    .gap_6()
                    .child(
                        v_flex()
                            .gap_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format_time(&wallpaper.time)),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(wallpaper.name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(wallpaper.description.clone()),
                            ),
                    )
                    .child(actions),
            )
            .when_some(self.status.as_ref(), |this, status| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(if status.is_error {
                            cx.theme().danger
                        } else {
                            cx.theme().success
                        })
                        .child(status.text.clone()),
                )
            })
            .into_any_element()
    }

    fn render_main(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .h_full()
            .min_w_0()
            .min_h_0()
            .gap_4()
            .p_6()
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .rounded(cx.theme().radius_lg)
                    .overflow_hidden()
                    .bg(cx.theme().muted)
                    .child(match &self.load {
                        LoadState::Ready => self.render_preview(cx),
                        LoadState::Loading => v_flex()
                            .size_full()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(Spinner::new().large())
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("正在加载每日图片…"),
                            )
                            .into_any_element(),
                        LoadState::Failed(error) => v_flex()
                            .size_full()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .p_6()
                            .child(Alert::error("list-error", error.clone()).title("加载失败"))
                            .child(
                                Button::new("retry-list")
                                    .outline()
                                    .label("重试")
                                    .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
                            )
                            .into_any_element(),
                    }),
            )
            .child(self.render_metadata(cx))
    }
}

impl Render for BingPaper {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_stretch()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(cx))
            .child(self.render_main(cx))
    }
}

fn preview_dir() -> PathBuf {
    std::env::temp_dir().join(PREVIEW_DIR)
}

/// 在独立的系统线程上运行 tokio 任务。
///
/// GPUI 的执行器不是 tokio，直接在它的 worker 上 `block_on` 会让 reqwest
/// 找不到 tokio reactor，所以这里给网络和文件 IO 单独开线程和 runtime。
fn on_tokio<T>(future: impl std::future::Future<Output = T> + Send + 'static) -> impl std::future::Future<Output = T>
where
    T: Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        let _ = sender.send(runtime.block_on(future));
    });
    async move {
        receiver
            .await
            .expect("tokio worker thread dropped the result")
    }
}
