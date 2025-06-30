use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

use crate::storage::Storage;

#[derive(Debug, Clone, Copy)]
pub enum Period {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Default)]
struct Acc {
    count: usize,
    distance: f64,
    wp_sum: f64,
    wp_count: usize,
    if_sum: f64,
    if_count: usize,
    tss_sum: f64,
    spd_sum: f64,
    spd_count: usize,
}

#[derive(Debug, Serialize)]
pub struct StatsEntry {
    pub period: String,
    pub rides: usize,
    pub distance: f64,
    pub weighted_power: Option<f64>,
    pub intensity_factor: Option<f64>,
    pub training_stress: Option<f64>,
    pub average_speed: Option<f64>,
}

fn period_key(date: NaiveDate, p: Period) -> String {
    match p {
        Period::Day => date.to_string(),
        Period::Week => {
            let iso = date.iso_week();
            format!("{}-W{:02}", iso.year(), iso.week())
        }
        Period::Month => format!("{}-{:02}", date.year(), date.month()),
        Period::Year => format!("{}", date.year()),
    }
}

impl Storage {
    pub async fn activity_stats(
        &self,
        period: Period,
        ids: Option<&[u64]>,
        types: Option<&[String]>,
    ) -> anyhow::Result<Vec<StatsEntry>> {
        let acts = self.list_activities(None).await?;
        let filter: Option<HashSet<u64>> = ids.map(|l| l.iter().copied().collect());
        let type_filter: Option<HashSet<String>> = types.map(|l| l.iter().cloned().collect());
        let mut map: BTreeMap<String, Acc> = BTreeMap::new();
        for a in acts {
            if let Some(ref f) = filter {
                if !f.contains(&a.id) {
                    continue;
                }
            }
            let summary = self.load_activity_summary(a.id).await?;
            if let Some(ref tf) = type_filter {
                if let Some(ref t) = summary.activity_type {
                    if !tf.contains(t) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            let dt = chrono::DateTime::parse_from_rfc3339(&summary.start_date)?.naive_utc();
            let key = period_key(dt.date(), period);
            let entry = map.entry(key).or_default();
            entry.count += 1;
            entry.distance += summary.distance;
            if let Some(wp) = summary.average_power {
                entry.wp_sum += wp;
                entry.wp_count += 1;
            }
            if let Some(ifv) = summary.intensity_factor {
                entry.if_sum += ifv;
                entry.if_count += 1;
            }
            if let Some(tss) = summary.training_stress_score {
                entry.tss_sum += tss;
            }
            if let Some(spd) = summary.average_speed {
                entry.spd_sum += spd;
                entry.spd_count += 1;
            }
        }
        let mut out = Vec::new();
        for (period, acc) in map {
            out.push(StatsEntry {
                period,
                rides: acc.count,
                distance: acc.distance,
                weighted_power: if acc.wp_count > 0 { Some(acc.wp_sum / acc.wp_count as f64) } else { None },
                intensity_factor: if acc.if_count > 0 { Some(acc.if_sum / acc.if_count as f64) } else { None },
                training_stress: if acc.count > 0 { Some(acc.tss_sum) } else { None },
                average_speed: if acc.spd_count > 0 { Some(acc.spd_sum / acc.spd_count as f64) } else { None },
            });
        }
        Ok(out)
    }
}

