use merge::Merge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateFrequency {
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "never")]
    Never,
}

impl Default for UpdateFrequency {
    fn default() -> Self {
        Self::Daily
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Merge, Default, PartialEq)]
pub struct Update {
    pub check_frequency: Option<UpdateFrequency>,
    pub auto_update: Option<bool>,
}

pub fn update_config(base: &mut Option<Update>, other: Option<Update>) {
    if let Some(other) = other {
        // If base is None, create a new update config
        let mut update = base.clone().unwrap_or_default();

        // Only update the values that are set in the other config
        if other.auto_update.is_some() {
            update.auto_update = other.auto_update;
        }

        if other.check_frequency.is_some() {
            update.check_frequency = other.check_frequency;
        }

        // Apply merged config to the base config
        *base = Some(update);
    }
}
