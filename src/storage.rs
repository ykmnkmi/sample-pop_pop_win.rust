use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;

const STORAGE_PATH: &str = "highscores.json";

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct HighScores {
    pub records: HashMap<String, u64>, // millis
}

impl HighScores {
    pub fn load() -> Self {
        if let Ok(file) = File::open(STORAGE_PATH) {
            if let Ok(records) = serde_json::from_reader(BufReader::new(file)) {
                return Self { records };
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.records) {
            let _ = fs::write(STORAGE_PATH, json);
        }
    }

    pub fn key(w: usize, h: usize, m: usize) -> String {
        format!("w{}-h{}-m{}", w, h, m)
    }

    pub fn get_record(&self, w: usize, h: usize, m: usize) -> Option<u64> {
        let k = Self::key(w, h, m);
        self.records.get(&k).copied()
    }

    pub fn update_record(&mut self, w: usize, h: usize, m: usize, duration_millis: u64) -> bool {
        let k = Self::key(w, h, m);
        let updated = match self.records.get(&k) {
            Some(&existing) => duration_millis < existing,
            None => true,
        };

        if updated {
            self.records.insert(k, duration_millis);
            self.save();
        }
        updated
    }
}
