use std::{fs::File, io::Write, path::Path};

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

#[derive(Debug, Default, Serialize, Deserialize)]
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
    let path = format!("{}/{}-{}.jpg", home, wp.time, wp.name);
    if Path::new(&path).exists() {
        println!("Picture exists: {}", path);
        return Ok(path);
    }

    let url = format!("https://cn.bing.com{}", wp.url);
    let content = reqwest::get(url).await?.bytes().await?;
    File::create(&path)?.write_all(&content)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_wallpapers() {
        println!("{:#?}", get_wallpapers().await);
    }
}
