use crate::schema::{ActivityHeader, ActivityDetail, ParsedStreams};
use crate::utils::Storage as StorageCfg;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use zstd::stream::{encode_all, decode_all};

fn weighted_avg_power(power: &[i64]) -> f64 {
    if power.is_empty() {
        return 0.0;
    }
    let window = 30.min(power.len());
    let mut sum: i64 = power[..window].iter().sum();
    let mut fourth_sum = (sum as f64 / window as f64).powi(4);
    for i in window..power.len() {
        sum += power[i] - power[i - window];
        fourth_sum += (sum as f64 / window as f64).powi(4);
    }
    let count = power.len() - window + 1;
    (fourth_sum / count as f64).powf(0.25)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FtpEntry {
    pub date: String,
    pub ftp: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeightEntry {
    pub date: String,
    pub weight: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WkgEntry {
    pub date: String,
    pub wkg: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreEntry {
    pub date: String,
    pub score: f64,
}

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

    fn ftp_path(&self) -> PathBuf {
        self.base.join("ftp.json")
    }

    fn weight_path(&self) -> PathBuf {
        self.base.join("weight.json")
    }

    fn wkg_path(&self) -> PathBuf {
        self.base.join("wkg.json")
    }

    fn enduro_path(&self) -> PathBuf {
        self.base.join("enduro.json")
    }

    fn fitness_path(&self) -> PathBuf {
        self.base.join("fitness.json")
    }

    async fn load_ftp_history(&self) -> anyhow::Result<Vec<FtpEntry>> {
        let path = self.ftp_path();
        if let Ok(data) = fs::read(&path).await {
            Ok(serde_json::from_slice(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_ftp_history(&self, hist: &[FtpEntry]) -> anyhow::Result<()> {
        let data = serde_json::to_vec(hist)?;
        if let Some(parent) = self.ftp_path().parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(self.ftp_path(), data).await?;
        Ok(())
    }

    pub async fn get_ftp_history(&self) -> anyhow::Result<Vec<FtpEntry>> {
        let mut hist = self.load_ftp_history().await?;
        if hist.is_empty() {
            hist.push(FtpEntry { date: Utc::now().date_naive().to_string(), ftp: 240.0 });
            self.save_ftp_history(&hist).await?;
        }
        Ok(hist)
    }

    pub async fn current_ftp(&self) -> anyhow::Result<f64> {
        Ok(self.get_ftp_history().await?.last().map(|e| e.ftp).unwrap_or(240.0))
    }

    pub async fn ftp_history(&self, count: Option<usize>) -> anyhow::Result<Vec<FtpEntry>> {
        let mut hist = self.get_ftp_history().await?;
        hist.reverse();
        if let Some(n) = count {
            hist.truncate(n);
        }
        Ok(hist)
    }

    pub async fn set_ftp(&self, ftp: f64) -> anyhow::Result<()> {
        let mut hist = self.get_ftp_history().await?;
        hist.push(FtpEntry { date: Utc::now().date_naive().to_string(), ftp });
        self.save_ftp_history(&hist).await?;
        let weight = self.current_weight().await.unwrap_or(75.0);
        self.record_wkg(ftp / weight).await
    }

    async fn load_weight_history(&self) -> anyhow::Result<Vec<WeightEntry>> {
        let path = self.weight_path();
        if let Ok(data) = fs::read(&path).await {
            Ok(serde_json::from_slice(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_weight_history(&self, hist: &[WeightEntry]) -> anyhow::Result<()> {
        let data = serde_json::to_vec(hist)?;
        if let Some(parent) = self.weight_path().parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(self.weight_path(), data).await?;
        Ok(())
    }

    pub async fn get_weight_history(&self) -> anyhow::Result<Vec<WeightEntry>> {
        let mut hist = self.load_weight_history().await?;
        if hist.is_empty() {
            hist.push(WeightEntry { date: Utc::now().date_naive().to_string(), weight: 75.0 });
            self.save_weight_history(&hist).await?;
        }
        Ok(hist)
    }

    pub async fn current_weight(&self) -> anyhow::Result<f64> {
        Ok(self.get_weight_history().await?.last().map(|e| e.weight).unwrap_or(75.0))
    }

    pub async fn weight_history(&self, count: Option<usize>) -> anyhow::Result<Vec<WeightEntry>> {
        let mut hist = self.get_weight_history().await?;
        hist.reverse();
        if let Some(n) = count {
            hist.truncate(n);
        }
        Ok(hist)
    }

    pub async fn set_weight(&self, weight: f64) -> anyhow::Result<()> {
        let mut hist = self.get_weight_history().await?;
        hist.push(WeightEntry { date: Utc::now().date_naive().to_string(), weight });
        self.save_weight_history(&hist).await?;
        let ftp = self.current_ftp().await.unwrap_or(240.0);
        self.record_wkg(ftp / weight).await
    }

    async fn load_wkg_history(&self) -> anyhow::Result<Vec<WkgEntry>> {
        let path = self.wkg_path();
        if let Ok(data) = fs::read(&path).await {
            Ok(serde_json::from_slice(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_wkg_history(&self, hist: &[WkgEntry]) -> anyhow::Result<()> {
        let data = serde_json::to_vec(hist)?;
        if let Some(parent) = self.wkg_path().parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(self.wkg_path(), data).await?;
        Ok(())
    }

    pub async fn get_wkg_history(&self) -> anyhow::Result<Vec<WkgEntry>> {
        let mut hist = self.load_wkg_history().await?;
        if hist.is_empty() {
            let ftp = self.current_ftp().await.unwrap_or(240.0);
            let weight = self.current_weight().await.unwrap_or(75.0);
            hist.push(WkgEntry { date: Utc::now().date_naive().to_string(), wkg: ftp / weight });
            self.save_wkg_history(&hist).await?;
        }
        Ok(hist)
    }

    pub async fn current_wkg(&self) -> anyhow::Result<f64> {
        Ok(self.get_wkg_history().await?.last().map(|e| e.wkg).unwrap_or(0.0))
    }

    pub async fn wkg_history(&self, count: Option<usize>) -> anyhow::Result<Vec<WkgEntry>> {
        let mut hist = self.get_wkg_history().await?;
        hist.reverse();
        if let Some(n) = count {
            hist.truncate(n);
        }
        Ok(hist)
    }

    async fn record_wkg(&self, wkg: f64) -> anyhow::Result<()> {
        let mut hist = self.get_wkg_history().await?;
        hist.push(WkgEntry { date: Utc::now().date_naive().to_string(), wkg });
        self.save_wkg_history(&hist).await
    }

    async fn load_score_history(&self, path: &Path) -> anyhow::Result<Vec<ScoreEntry>> {
        if let Ok(data) = fs::read(path).await {
            Ok(serde_json::from_slice(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_score_history(&self, path: &Path, hist: &[ScoreEntry]) -> anyhow::Result<()> {
        let data = serde_json::to_vec(hist)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, data).await?;
        Ok(())
    }

    async fn record_score(&self, path: &Path, score: f64) -> anyhow::Result<()> {
        let mut hist = self.load_score_history(path).await?;
        hist.push(ScoreEntry { date: Utc::now().date_naive().to_string(), score });
        self.save_score_history(path, &hist).await
    }

    pub async fn enduro_history(&self, count: Option<usize>) -> anyhow::Result<Vec<ScoreEntry>> {
        let mut hist = self.load_score_history(&self.enduro_path()).await?;
        hist.reverse();
        if let Some(n) = count { hist.truncate(n); }
        Ok(hist)
    }

    pub async fn fitness_history(&self, count: Option<usize>) -> anyhow::Result<Vec<ScoreEntry>> {
        let mut hist = self.load_score_history(&self.fitness_path()).await?;
        hist.reverse();
        if let Some(n) = count { hist.truncate(n); }
        Ok(hist)
    }

    pub async fn current_enduro(&self) -> anyhow::Result<f64> {
        Ok(self.load_score_history(&self.enduro_path()).await?.last().map(|e| e.score).unwrap_or(0.0))
    }

    pub async fn current_fitness(&self) -> anyhow::Result<f64> {
        Ok(self.load_score_history(&self.fitness_path()).await?.last().map(|e| e.score).unwrap_or(0.0))
    }

    pub async fn update_enduro(&self) -> anyhow::Result<f64> {
        let score = self.compute_enduro_score().await?;
        self.record_score(&self.enduro_path(), score).await?;
        Ok(score)
    }

    pub async fn update_fitness(&self) -> anyhow::Result<f64> {
        let score = self.compute_fitness_score().await?;
        self.record_score(&self.fitness_path(), score).await?;
        Ok(score)
    }

    async fn compute_enduro_score(&self) -> anyhow::Result<f64> {
        let acts = self.list_activities(None).await?;
        let today = Utc::now().naive_utc().date();
        let mut long_products = Vec::new();
        let mut week_volume = 0f64;
        let mut tss_sum = 0f64;
        let mut last_long: Option<i64> = None;
        for a in acts {
            let dt = chrono::DateTime::parse_from_rfc3339(&a.start_date)?.naive_utc().date();
            let days = (today - dt).num_days();
            if days <= 28 {
                let summary = self.load_activity_summary(a.id).await?;
                if days < 7 {
                    week_volume += summary.duration as f64 / 3600.0;
                }
                if let Some(tss) = summary.training_stress_score { tss_sum += tss; }
                if summary.distance >= 80000.0 {
                    long_products.push(summary.distance * summary.duration as f64);
                    if last_long.map_or(true, |d| days < d) { last_long = Some(days); }
                }
            }
        }
        let avg_long = if !long_products.is_empty() {
            long_products.iter().sum::<f64>() / long_products.len() as f64
        } else { 0.0 };
        let mut score = avg_long / 10000.0 + week_volume + tss_sum / 100.0;
        if let Some(days) = last_long {
            if days > 14 { score *= 0.9_f64.powf((days - 14) as f64); }
        }
        Ok(score)
    }

    async fn compute_fitness_score(&self) -> anyhow::Result<f64> {
        use chrono::Duration;
        let acts = self.list_activities(None).await?;
        let today = Utc::now().naive_utc().date();
        let mut week_hours = 0f64;
        let mut tss_sum = 0f64;
        let mut long_count = 0u32;
        let mut dates = std::collections::HashSet::new();
        for a in acts {
            let dt = chrono::DateTime::parse_from_rfc3339(&a.start_date)?.naive_utc().date();
            let days = (today - dt).num_days();
            if days <= 28 {
                let summary = self.load_activity_summary(a.id).await?;
                if days < 7 { week_hours += summary.duration as f64 / 3600.0; }
                if let Some(tss) = summary.training_stress_score { tss_sum += tss; }
                if summary.distance >= 80000.0 { long_count += 1; }
                dates.insert(dt);
            }
        }
        let four_week_avg = (tss_sum / 4.0) / 10.0;
        let mut score = week_hours * 4.0 + four_week_avg + long_count as f64;
        let mut rest_days = 0i64;
        for i in 0.. { let day = today - Duration::days(i); if dates.contains(&day) { break } else { rest_days += 1; } }
        if rest_days > 3 { score *= 0.985_f64.powf((rest_days - 3) as f64); }
        Ok(score)
    }

    pub async fn save(&self, meta: &serde_json::Value, streams: &serde_json::Value) -> anyhow::Result<()> {
        let date = meta["start_date"].as_str().unwrap_or("1970-01-01");
        let year = &date[..4];
        let id = meta["id"].as_u64().unwrap();
        let dir = self.activity_dir(year, id);
        fs::create_dir_all(&dir).await?;

        let mut meta = meta.clone();
        if let Some(parsed) = crate::schema::parse_streams(streams) {
            if !parsed.power.is_empty() {
                let np = weighted_avg_power(&parsed.power);
                let ftp = self.current_ftp().await.unwrap_or(240.0);
                let duration = meta
                    .get("elapsed_time")
                    .and_then(|v| v.as_i64())
                    .or_else(|| parsed.time.last().cloned())
                    .unwrap_or(0) as f64;
                let ifv = np / ftp;
                let tss = (duration * np * ifv) / (ftp * 3600.0) * 100.0;
                if let Some(obj) = meta.as_object_mut() {
                    obj.insert("normalized_power".into(), serde_json::Value::from(np));
                    obj.insert("intensity_factor".into(), serde_json::Value::from(ifv));
                    obj.insert("training_stress_score".into(), serde_json::Value::from(tss));
                }
            }
        }

        self.write_zstd(dir.join("meta.json.zst"), &meta).await?;
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
                    .unwrap_or(ParsedStreams { time: vec![], power: vec![], heartrate: vec![] });
                return Ok(ActivityDetail { meta, streams });
            }
        }
        anyhow::bail!("not found")
    }

    pub async fn load_activity_summary(&self, id: u64) -> anyhow::Result<crate::schema::ActivitySummary> {
        let detail = self.load_activity(id).await?;
        let duration = detail
            .meta
            .get("elapsed_time")
            .and_then(|v| v.as_i64())
            .or_else(|| detail.streams.time.last().cloned())
            .unwrap_or(0);
        let weighted_average_power = detail
            .meta
            .get("weighted_average_watts")
            .and_then(|v| v.as_f64())
            .or_else(|| detail.meta.get("average_watts").and_then(|v| v.as_f64()))
            .or_else(|| if !detail.streams.power.is_empty() {
                Some(weighted_avg_power(&detail.streams.power))
            } else {
                None
            });
        let distance = detail.meta.get("distance").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let average_speed = detail
            .meta
            .get("average_speed")
            .and_then(|v| v.as_f64())
            .or_else(|| if duration > 0 { Some(distance / duration as f64) } else { None })
            .map(|mps| mps * 3.6);
        let pr_count = if let Some(efforts) = detail
            .meta
            .get("segment_efforts")
            .and_then(|v| v.as_array())
        {
            Some(
                efforts
                    .iter()
                    .filter(|e| e.get("pr_rank").and_then(|v| v.as_i64()) == Some(1))
                    .count() as i64,
            )
        } else {
            detail.meta.get("pr_count").and_then(|v| v.as_i64())
        };
        let average_heartrate = if !detail.streams.heartrate.is_empty() {
            Some(detail.streams.heartrate.iter().sum::<i64>() as f64 / detail.streams.heartrate.len() as f64)
        } else {
            detail.meta.get("average_heartrate").and_then(|v| v.as_f64())
        };
        let normalized_power = detail
            .meta
            .get("normalized_power")
            .and_then(|v| v.as_f64())
            .or_else(|| if !detail.streams.power.is_empty() {
                Some(weighted_avg_power(&detail.streams.power))
            } else { None });
        let ftp = self.current_ftp().await.unwrap_or(240.0);
        let intensity_factor = detail
            .meta
            .get("intensity_factor")
            .and_then(|v| v.as_f64())
            .or_else(|| normalized_power.map(|np| np / ftp));
        let training_stress_score = detail
            .meta
            .get("training_stress_score")
            .and_then(|v| v.as_f64())
            .or_else(|| normalized_power.map(|np| (duration as f64 * np * (np / ftp)) / (ftp * 3600.0) * 100.0));
        let summary_polyline = detail
            .meta
            .get("map")
            .and_then(|m| m.get("summary_polyline"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let activity_type = detail
            .meta
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(crate::schema::ActivitySummary {
            id: detail.meta.get("id").and_then(|v| v.as_u64()).unwrap_or(id),
            name: detail
                .meta
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            start_date: detail
                .meta
                .get("start_date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            distance: detail.meta.get("distance").and_then(|v| v.as_f64()).unwrap_or(0.0),
            duration,
            weighted_average_power,
            average_speed,
            pr_count,
            average_heartrate,
            summary_polyline,
            normalized_power,
            intensity_factor,
            training_stress_score,
            activity_type,
        })
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
