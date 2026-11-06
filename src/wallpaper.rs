use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::convert_url;

#[derive(Debug, Deserialize)]
struct ModelResp {
    #[serde(rename = "MediaContents")]
    media_contents: Vec<MediaContent>,
    #[serde(rename = "PreloadMediaContents")]
    preload_media_contents: Vec<MediaContent>,
}

#[derive(Debug, Deserialize)]
struct MediaContent {
    #[serde(rename = "Ssd")]
    ssd: String,
    #[serde(rename = "ImageContent")]
    image_content: ImageContent,
}

#[derive(Debug, Deserialize)]
struct ImageContent {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Image")]
    image_url: ImageUrl,
}

#[derive(Debug, Deserialize)]
struct ImageUrl {
    #[serde(rename = "Wallpaper")]
    wallpaper: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    pub time: String,
    pub name: String,
    pub description: String,
    pub url: String,
}

pub async fn get_wallpapers() -> Result<Vec<Wallpaper>> {
    let resp = reqwest::get("https://cn.bing.com/hp/api/model")
        .await?
        .json::<ModelResp>()
        .await?;

    let mut wallpapers = Vec::new();

    for content in resp.media_contents {
        let wp = Wallpaper {
            time: content.ssd,
            name: content.image_content.title,
            description: content.image_content.description,
            url: convert_url(content.image_content.image_url.wallpaper),
        };
        wallpapers.push(wp);
    }
    for content in resp.preload_media_contents {
        let wp = Wallpaper {
            time: content.ssd,
            name: content.image_content.title,
            description: content.image_content.description,
            url: convert_url(content.image_content.image_url.wallpaper),
        };
        wallpapers.push(wp);
    }

    Ok(wallpapers)
}

pub async fn download_picture(home: &str, wp: Wallpaper) -> Result<String> {
    let path = download_picture_to(Path::new(home), &wp).await?;
    Ok(path.to_string_lossy().into_owned())
}

/// 把图片下载到 `dir` 目录，返回本地文件路径。
///
/// 目标文件已存在时直接返回，不重复下载。
pub async fn download_picture_to(dir: &Path, wp: &Wallpaper) -> Result<PathBuf> {
    fs::create_dir_all(dir)?;

    let path = dir.join(format!(
        "{}-{}.jpg",
        wp.time,
        sanitize_file_stem(&wp.name)
    ));
    if path.exists() {
        return Ok(path);
    }

    let url = format!("https://cn.bing.com{}", wp.url);
    let content = reqwest::get(url).await?.bytes().await?;
    File::create(&path)?.write_all(&content)?;

    Ok(path)
}

fn sanitize_file_stem(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == ':' || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        "bing-wallpaper".to_string()
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_wallpapers() {
        println!("{:#?}", get_wallpapers().await);
    }
}
