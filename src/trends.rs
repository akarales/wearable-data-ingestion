//! Basic trend detection over normalized wearable series (pure).

use crate::normalize::Observation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Rising,
    Falling,
    Flat,
}

impl Trend {
    pub fn label(&self) -> &'static str {
        match self {
            Trend::Rising => "rising",
            Trend::Falling => "falling",
            Trend::Flat => "flat",
        }
    }
}

/// Split-half mean comparison on chronologically sorted observations.
pub fn trend(observations: &[Observation]) -> Option<Trend> {
    if observations.len() < 4 {
        return None;
    }
    let midpoint = observations.len() / 2;
    let first: f64 = observations[..midpoint]
        .iter()
        .map(|o| o.value_quantity.value)
        .sum::<f64>()
        / midpoint as f64;
    let second: f64 = observations[midpoint..]
        .iter()
        .map(|o| o.value_quantity.value)
        .sum::<f64>()
        / (observations.len() - midpoint) as f64;
    let relative = (second - first).abs() / first.abs().max(1e-9);
    if relative < 0.05 {
        Some(Trend::Flat)
    } else if second > first {
        Some(Trend::Rising)
    } else {
        Some(Trend::Falling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::{Coding, CodingWrap, Quantity};

    fn obs(value: f64, day: u32) -> Observation {
        Observation {
            resource_type: "Observation".into(),
            status: "final".into(),
            code: CodingWrap {
                coding: [Coding {
                    system: "http://loinc.org".into(),
                    code: "8867-4".into(),
                    display: "Heart rate".into(),
                }],
            },
            effective_date_time: chrono::NaiveDate::from_ymd_opt(2026, 9, day).expect("date"),
            value_quantity: Quantity {
                value,
                unit: "bpm".into(),
            },
        }
    }

    #[test]
    fn rising_detected() {
        let series: Vec<Observation> = (1..=8).map(|i| obs(60.0 + i as f64, i)).collect();
        assert_eq!(trend(&series), Some(Trend::Rising));
    }

    #[test]
    fn flat_detected() {
        let series: Vec<Observation> = (1..=8)
            .map(|i| obs(62.0 + (i % 2) as f64 * 0.1, i))
            .collect();
        assert_eq!(trend(&series), Some(Trend::Flat));
    }

    #[test]
    fn too_short_is_none() {
        assert_eq!(trend(&[obs(60.0, 1)]), None);
    }
}
