use serde::{Deserialize, Serialize};

use super::card::Card;
use super::errors::{JabeXError, JabeXResult};

pub const LEITNER_BOX_COUNT: usize = Card::MAX_BOX as usize;
pub const PROFILE_ALGORITHM_SETTING_KEY: &str = "leitner.algorithm";

/// Named interval packages shipped with the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LeitnerPresetId {
    #[default]
    Recommended,
    Intensive,
    Relaxed,
    LongTerm,
}

impl LeitnerPresetId {
    pub const ALL: [Self; 4] = [
        Self::Recommended,
        Self::Intensive,
        Self::Relaxed,
        Self::LongTerm,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recommended => "recommended",
            Self::Intensive => "intensive",
            Self::Relaxed => "relaxed",
            Self::LongTerm => "longTerm",
        }
    }

    pub fn intervals_days(self) -> [i64; LEITNER_BOX_COUNT] {
        match self {
            // Classic Leitner defaults used since MVP.
            Self::Recommended => [1, 3, 7, 14, 30],
            // Shorter gaps for denser practice.
            Self::Intensive => [1, 2, 4, 7, 14],
            // Longer gaps for lighter daily load.
            Self::Relaxed => [2, 5, 12, 25, 60],
            // Stretch intervals for durable retention.
            Self::LongTerm => [1, 4, 10, 30, 90],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AlgorithmMode {
    #[default]
    Preset,
    Custom,
}

/// Resolved intervals used by the scheduler (always length = box count).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeitnerConfig {
    pub preset_id: Option<LeitnerPresetId>,
    pub intervals_days: [i64; LEITNER_BOX_COUNT],
}

impl LeitnerConfig {
    pub fn recommended() -> Self {
        Self::from_preset(LeitnerPresetId::Recommended)
    }

    pub fn from_preset(preset_id: LeitnerPresetId) -> Self {
        Self {
            preset_id: Some(preset_id),
            intervals_days: preset_id.intervals_days(),
        }
    }

    pub fn from_custom(intervals_days: [i64; LEITNER_BOX_COUNT]) -> JabeXResult<Self> {
        validate_intervals(&intervals_days)?;
        Ok(Self {
            preset_id: None,
            intervals_days,
        })
    }

    pub fn interval_for_box(&self, box_position: u32) -> i64 {
        let idx = (Card::clamp_box(box_position) - Card::MIN_BOX) as usize;
        self.intervals_days[idx]
    }
}

impl Default for LeitnerConfig {
    fn default() -> Self {
        Self::recommended()
    }
}

/// Profile-wide default algorithm (stored in `settings`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileLeitnerSettings {
    #[serde(default)]
    pub mode: AlgorithmMode,
    #[serde(default)]
    pub preset_id: LeitnerPresetId,
    #[serde(default, alias = "intervals_days")]
    pub intervals_days: Option<Vec<i64>>,
}

impl Default for ProfileLeitnerSettings {
    fn default() -> Self {
        Self {
            mode: AlgorithmMode::Preset,
            preset_id: LeitnerPresetId::Recommended,
            intervals_days: None,
        }
    }
}

impl ProfileLeitnerSettings {
    pub fn resolve(&self) -> JabeXResult<LeitnerConfig> {
        match self.mode {
            AlgorithmMode::Preset => Ok(LeitnerConfig::from_preset(self.preset_id)),
            AlgorithmMode::Custom => {
                let intervals = parse_intervals(self.intervals_days.as_deref())?;
                LeitnerConfig::from_custom(intervals)
            }
        }
    }
}

/// Per-deck override. When `inherit` is true, the profile default applies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckAlgorithmSettings {
    #[serde(default = "default_inherit")]
    pub inherit: bool,
    #[serde(default)]
    pub mode: AlgorithmMode,
    #[serde(default)]
    pub preset_id: LeitnerPresetId,
    #[serde(default, alias = "intervals_days")]
    pub intervals_days: Option<Vec<i64>>,
}

fn default_inherit() -> bool {
    true
}

impl Default for DeckAlgorithmSettings {
    fn default() -> Self {
        Self {
            inherit: true,
            mode: AlgorithmMode::Preset,
            preset_id: LeitnerPresetId::Recommended,
            intervals_days: None,
        }
    }
}

impl DeckAlgorithmSettings {
    pub fn inherit_profile() -> Self {
        Self::default()
    }

    pub fn from_profile(profile: &ProfileLeitnerSettings) -> Self {
        Self {
            inherit: false,
            mode: profile.mode,
            preset_id: profile.preset_id,
            intervals_days: profile.intervals_days.clone(),
        }
    }

    pub fn resolve(&self, profile: &ProfileLeitnerSettings) -> JabeXResult<LeitnerConfig> {
        if self.inherit {
            return profile.resolve();
        }
        match self.mode {
            AlgorithmMode::Preset => Ok(LeitnerConfig::from_preset(self.preset_id)),
            AlgorithmMode::Custom => {
                let intervals = parse_intervals(self.intervals_days.as_deref())?;
                LeitnerConfig::from_custom(intervals)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeitnerPresetInfo {
    pub id: LeitnerPresetId,
    pub intervals_days: [i64; LEITNER_BOX_COUNT],
}

pub fn list_presets() -> Vec<LeitnerPresetInfo> {
    LeitnerPresetId::ALL
        .into_iter()
        .map(|id| LeitnerPresetInfo {
            id,
            intervals_days: id.intervals_days(),
        })
        .collect()
}

fn parse_intervals(raw: Option<&[i64]>) -> JabeXResult<[i64; LEITNER_BOX_COUNT]> {
    let Some(values) = raw else {
        return Err(JabeXError::Validation(
            "custom Leitner intervals require five day values".into(),
        ));
    };
    if values.len() != LEITNER_BOX_COUNT {
        return Err(JabeXError::Validation(format!(
            "custom Leitner intervals must have {LEITNER_BOX_COUNT} values"
        )));
    }
    let mut out = [0i64; LEITNER_BOX_COUNT];
    out.copy_from_slice(values);
    validate_intervals(&out)?;
    Ok(out)
}

fn validate_intervals(intervals: &[i64; LEITNER_BOX_COUNT]) -> JabeXResult<()> {
    for (index, days) in intervals.iter().enumerate() {
        if *days < 1 {
            return Err(JabeXError::Validation(format!(
                "box {} interval must be at least 1 day",
                index + 1
            )));
        }
        if *days > 3650 {
            return Err(JabeXError::Validation(format!(
                "box {} interval must be at most 3650 days",
                index + 1
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_matches_classic_defaults() {
        assert_eq!(
            LeitnerPresetId::Recommended.intervals_days(),
            [1, 3, 7, 14, 30]
        );
    }

    #[test]
    fn deck_inherit_uses_profile() {
        let profile = ProfileLeitnerSettings {
            mode: AlgorithmMode::Preset,
            preset_id: LeitnerPresetId::Intensive,
            intervals_days: None,
        };
        let deck = DeckAlgorithmSettings::inherit_profile();
        let config = deck.resolve(&profile).unwrap();
        assert_eq!(config.intervals_days, [1, 2, 4, 7, 14]);
        assert_eq!(config.preset_id, Some(LeitnerPresetId::Intensive));
    }

    #[test]
    fn deck_override_custom() {
        let profile = ProfileLeitnerSettings::default();
        let deck = DeckAlgorithmSettings {
            inherit: false,
            mode: AlgorithmMode::Custom,
            preset_id: LeitnerPresetId::Recommended,
            intervals_days: Some(vec![1, 1, 2, 3, 5]),
        };
        let config = deck.resolve(&profile).unwrap();
        assert_eq!(config.intervals_days, [1, 1, 2, 3, 5]);
        assert_eq!(config.preset_id, None);
    }

    #[test]
    fn rejects_zero_day_interval() {
        let err = LeitnerConfig::from_custom([0, 1, 2, 3, 4]).unwrap_err();
        assert!(matches!(err, JabeXError::Validation(_)));
    }
}
