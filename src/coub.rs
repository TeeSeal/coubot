use bytes::{BufMut, BytesMut};
use serde_json::Value;
use std::{error::Error, path::Path};
use tempfile::NamedTempFile;
use tokio::{fs::File, prelude::*, process::Command};
use url::Url;

type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub struct Coub {
    pub id: String,
    pub title: String,
    pub video: String,
    pub audio: String,
    pub duration: f64,
    pub size: u64,
}

impl Coub {
    pub async fn download(&self, path: &Path) -> BoxResult<()> {
        let video_path = NamedTempFile::new()?.into_temp_path();
        self.write_video_to(&video_path).await?;

        Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_path)
            .args(&["-i", &self.audio, "-shortest", "-c:v:0", "copy", "-y"])
            .arg(path)
            .output()
            .await?;

        video_path.close()?;
        Ok(())
    }

    pub async fn download_loops(&self, path: &Path, loops: usize) -> BoxResult<()> {
        let video_path = NamedTempFile::new()?.into_temp_path();
        self.write_video_to(&video_path).await?;

        let concat_file_path = NamedTempFile::new()?.into_temp_path();
        let mut concat_file = File::create(&concat_file_path).await?;
        let line = format!("file '{}'\n", video_path.display());
        concat_file.write_all(line.repeat(loops).as_bytes()).await?;

        Command::new("ffmpeg")
            .args(&["-f", "concat", "-safe", "0", "-i"])
            .arg(&concat_file_path)
            .args(&["-i", &self.audio, "-shortest", "-c:v:0", "copy", "-y"])
            .arg(path)
            .output()
            .await?;

        video_path.close()?;
        concat_file_path.close()?;
        Ok(())
    }

    async fn write_video_to(&self, path: &Path) -> BoxResult<()> {
        let mut res = reqwest::get(&self.video).await?;
        let mut file = File::create(&path).await?;
        let mut first_chunk = true;

        while let Some(mut chunk) = res.chunk().await? {
            let mut bytes = BytesMut::with_capacity(chunk.len());

            if first_chunk {
                first_chunk = false;
                bytes.put_u8(0);
                bytes.put_u8(0);
                bytes.put(chunk.split_off(2));
            } else {
                bytes.put(chunk);
            }

            file.write_all(&bytes).await?;
        }

        Ok(())
    }
}

pub async fn fetch_coub(id: &str) -> BoxResult<Coub> {
    let id = get_coub_id(id);
    let mut url = "http://coub.com/api/v2/coubs/".to_string();
    url.push_str(&id);
    let json: Value = reqwest::get(&url).await?.json().await?;
    let urls = &json["file_versions"]["html5"];

    let audio = get_highest_quality(&urls["audio"]);
    let video = get_highest_quality(&urls["video"]);

    Ok(Coub {
        id: id,
        title: json["title"].as_str().unwrap().to_string(),
        audio: audio["url"].as_str().unwrap().to_string(),
        video: video["url"].as_str().unwrap().to_string(),
        duration: json["duration"].as_f64().unwrap(),
        size: video["size"].as_u64().unwrap(),
    })
}

fn get_coub_id(string: &str) -> String {
    match Url::parse(string) {
        Ok(parsed_url) => parsed_url
            .path_segments()
            .map(|c| c.last().unwrap_or(string))
            .unwrap()
            .to_string(),
        Err(_) => string.to_string(),
    }
}

fn get_highest_quality(urls: &Value) -> &Value {
    urls.get("high").unwrap_or(urls.get("med").unwrap())
}
