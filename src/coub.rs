use std::process::Command;
use serde_json::Value;
use std::{error::Error, io::Write, path::Path};
use tempfile::NamedTempFile;
use url::Url;

type BoxResult<T> = Result<T, Box<dyn Error>>;

pub struct Coub {
    pub id: String,
    pub title: String,
    pub video: String,
    pub audio: String,
    pub duration: f64,
    pub size: u64
}

impl Coub {
    pub fn download(&self, path: &Path) -> BoxResult<()> {
        let mut video = NamedTempFile::new()?;
        self.write_video_to(video.as_file_mut())?;

        Command::new("ffmpeg")
            .arg("-i")
            .arg(video.path())
            .args(&[
                "-i", &self.audio,
                "-shortest",
                "-c:v:0", "copy",
                "-y"
            ])
            .arg(path)
            .output()?;

        video.close()?;
        Ok(())
    }

    pub fn download_loops(&self, path: &Path, loops: usize) -> BoxResult<()> {
        let mut video = NamedTempFile::new()?;
        self.write_video_to(video.as_file_mut())?;

        let mut concat_file = NamedTempFile::new()?;
        let line = format!("file '{}'\n", video.path().display());
        concat_file.write_all(line.repeat(loops).as_bytes())?;

        Command::new("ffmpeg")
            .args(&[
                "-f", "concat",
                "-safe", "0",
                "-i"
            ])
            .arg(concat_file.path())
            .args(&[
                "-i", &self.audio,
                "-shortest",
                "-c:v:0", "copy",
                "-y"
            ])
            .arg(path)
            .output()?;

        video.close()?;
        concat_file.close()?;
        Ok(())
    }

    fn write_video_to(&self, file: &mut dyn Write) -> BoxResult<()> {
        let mut res = reqwest::get(&self.video)?;
        let mut buf: Vec<u8> = vec![];
        res.copy_to(&mut buf)?;
        buf[0] = 0;
        buf[1] = 0;
        file.write_all(&buf)?;
        Ok(())
    }
}

pub fn fetch_coub(id: &str) -> BoxResult<Coub> {
    let id = get_coub_id(id);
    let mut url = "http://coub.com/api/v2/coubs/".to_string();
    url.push_str(&id);
    let json: Value = reqwest::get(&url)?.json()?;
    let urls = &json["file_versions"]["html5"];

    let audio = get_highest_quality(&urls["audio"]);
    let video = get_highest_quality(&urls["video"]);

    Ok(Coub {
        id: id,
        title: json["title"].as_str().unwrap().to_string(),
        audio: audio["url"].as_str().unwrap().to_string(),
        video: video["url"].as_str().unwrap().to_string(),
        duration: json["duration"].as_f64().unwrap(),
        size: video["size"].as_u64().unwrap()
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
