use crate::schema::{ActivityHeader, ActivityDetail, ParsedStreams};
use crate::utils::Storage as StorageCfg;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use zstd::stream::{encode_all, decode_all};

#[derive(Clone)]
pub struct Storage {
    base: PathBuf,
}

impl Storage {
    pub fn new(cfg: &StorageCfg) -> Self {
        Self { base: PathBuf::from(&cfg.data_dir).join(&cfg.user) }
    }

    fn activity_dir(&self, year: &str, id: u64) -> PathBuf {
        self.base.join(year).join(id.to_string())
    }

    pub async fn save(&self, meta: &serde_json::Value, streams: &serde_json::Value) -> anyhow::Result<()> {
        let date = meta["start_date"].as_str().unwrap_or("1970-01-01");
        let year = &date[..4];
        let id = meta["id"].as_u64().unwrap();
        let dir = self.activity_dir(year, id);
        fs::create_dir_all(&dir).await?;
        self.write_zstd(dir.join("meta.json.zst"), meta).await?;
        self.write_zstd(dir.join("streams.json.zst"), streams).await?;
        Ok(())
    }

    pub async fn activity_exists(&self, year: &str, id: u64) -> bool {
        let path = self.activity_dir(year, id).join("meta.json.zst");
        fs::metadata(path).await.is_ok()
    }

    async fn write_zstd<P: AsRef<Path>>(&self, path: P, value: &serde_json::Value) -> anyhow::Result<()> {
        let data = serde_json::to_vec(value)?;
        let compressed = encode_all(&data[..], 0)?;
        let mut f = fs::File::create(path).await?;
        f.write_all(&compressed).await?;
        Ok(())
    }

    async fn read_zstd<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<serde_json::Value> {
        let data = fs::read(path).await?;
        let decompressed = decode_all(&data[..])?;
        Ok(serde_json::from_slice(&decompressed)?)
    }

    pub async fn list_activities(&self, limit: Option<usize>) -> anyhow::Result<Vec<ActivityHeader>> {
        let mut list = Vec::new();
        if let Ok(mut years) = fs::read_dir(&self.base).await {
            while let Some(year) = years.next_entry().await? {
                if !year.file_type().await?.is_dir() { continue; }
                let mut acts = fs::read_dir(year.path()).await?;
                while let Some(act) = acts.next_entry().await? {
                    if !act.file_type().await?.is_dir() { continue; }
                    let meta = self.read_zstd(act.path().join("meta.json.zst")).await?;
                    if let Ok(header) = serde_json::from_value::<ActivityHeader>(meta.clone()) {
                        list.push(header);
                    }
                }
            }
        }
        list.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        if let Some(n) = limit {
            list.truncate(n);
        }
        Ok(list)
    }

    pub async fn load_activity(&self, id: u64) -> anyhow::Result<ActivityDetail> {
        let mut years = fs::read_dir(&self.base).await?;
        while let Some(year) = years.next_entry().await? {
            let dir = year.path().join(id.to_string());
            if fs::metadata(&dir).await.is_ok() {
                let meta = self.read_zstd(dir.join("meta.json.zst")).await?;
                let raw_streams = self.read_zstd(dir.join("streams.json.zst")).await?;
                let streams = crate::schema::parse_streams(&raw_streams)
                    .unwrap_or(ParsedStreams { time: vec![], power: vec![] });
                return Ok(ActivityDetail { meta, streams });
            }
        }
        anyhow::bail!("not found")
    }

    pub async fn list_files(&self) -> anyhow::Result<Vec<String>> {
        let mut files = Vec::new();
        self.collect_sync(&self.base, "", &mut files)?;
        Ok(files)
    }

    fn collect_sync(&self, dir: &Path, prefix: &str, out: &mut Vec<String>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = if prefix.is_empty() { entry.file_name().to_string_lossy().into() } else { format!("{}/{}", prefix, entry.file_name().to_string_lossy()) };
            if entry.file_type()?.is_dir() {
                self.collect_sync(&entry.path(), &name, out)?;
            } else {
                out.push(name);
            }
        }
        Ok(())
    }

    pub async fn read_file(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let full = self.base.join(path);
        Ok(fs::read(full).await?)
    }
}
