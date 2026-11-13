//! Shared bench state, independent of device access, analysis providers and UI.
//!
//! Missing measurements stay `None`; zero is a real measured value. Source labels
//! describe provenance and do not authenticate a device or qualify its results.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

pub mod diagnostics;

/// JSON Schema for the persisted snapshot contract, not a transport authorization.
pub const SNAPSHOT_SCHEMA: &str = include_str!("../schemas/snapshot.json");

/// Input format for bounded client-error collection, not for automatic uploads.
pub const CLIENT_ERRORS_SCHEMA: &str = include_str!("../schemas/client-errors.json");

/// Version accepted by [`BenchSnapshot::validate`].
pub const SCHEMA_VERSION: u32 = 1;

/// The producer's declared origin, not proof of a current connection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// Model-generated state, not a measurement.
    Simulated,
    /// Previously captured state, not a live feed.
    Recorded,
    /// State declared live by its producer; freshness requires separate checks.
    Live,
}

/// Machine coordinates and operating state. Unknown numeric values remain absent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineState {
    /// X coordinate in millimetres, when known.
    pub x_mm: Option<f64>,
    /// Z coordinate in millimetres, when known.
    pub z_mm: Option<f64>,
    /// Non-negative spindle speed, when known.
    pub spindle_rpm: Option<f64>,
    /// Producer-supplied operating label; it is not an executable command.
    pub program_state: String,
}

/// Sample identity and freshness reported by an instrument adapter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstrumentState {
    /// Project-local device identity; public examples must not use serial numbers.
    pub device_id: String,
    /// Capture time in Unix milliseconds, when available.
    pub captured_at_ms: Option<u64>,
    /// Whether the producer considers the sample stale.
    pub stale: bool,
    /// Measurement units, when known.
    pub units: Option<String>,
}

/// State shared by the bench views without embedding firmware or device access.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BenchSnapshot {
    /// Persisted contract version.
    pub schema_version: u32,
    /// Producer-defined sequence within its own session, not a wall clock.
    pub tick: u64,
    /// Declared data provenance.
    pub source: SourceKind,
    /// Current or retained machine state.
    pub machine: MachineState,
    /// Adapter-reported instrument state.
    pub instruments: Vec<InstrumentState>,
    /// Local operator warnings; review their content before sharing a report.
    pub warnings: Vec<String>,
}

impl BenchSnapshot {
    /// Check version and numeric validity without connecting to hardware.
    ///
    /// This does not validate transport identity, freshness, units or calibration.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SCHEMA_VERSION {
            return Err("unsupported schema version");
        }
        if [
            self.machine.x_mm,
            self.machine.z_mm,
            self.machine.spindle_rpm,
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite())
        {
            return Err("non-finite machine value");
        }
        if self.machine.spindle_rpm.is_some_and(|rpm| rpm < 0.0) {
            return Err("negative spindle speed");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_roundtrip() {
        let snapshot = BenchSnapshot {
            schema_version: SCHEMA_VERSION,
            tick: 42,
            source: SourceKind::Simulated,
            machine: MachineState {
                x_mm: Some(12.5),
                z_mm: None,
                spindle_rpm: Some(600.0),
                program_state: "idle".into(),
            },
            instruments: vec![],
            warnings: vec![],
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let decoded: BenchSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, decoded);
        assert_eq!(decoded.validate(), Ok(()));
    }

    #[test]
    fn reject_unsupported_version() {
        let snapshot = BenchSnapshot {
            schema_version: 999,
            tick: 0,
            source: SourceKind::Recorded,
            machine: MachineState {
                x_mm: None,
                z_mm: None,
                spindle_rpm: None,
                program_state: "unknown".into(),
            },
            instruments: vec![],
            warnings: vec![],
        };
        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn reject_non_finite_values() {
        let snapshot = BenchSnapshot {
            schema_version: SCHEMA_VERSION,
            tick: 0,
            source: SourceKind::Live,
            machine: MachineState {
                x_mm: Some(f64::NAN),
                z_mm: None,
                spindle_rpm: None,
                program_state: "unknown".into(),
            },
            instruments: vec![],
            warnings: vec![],
        };
        assert!(snapshot.validate().is_err());
    }
    #[test]
    fn reject_negative_spindle_speed() {
        let snapshot = BenchSnapshot {
            schema_version: SCHEMA_VERSION,
            tick: 0,
            source: SourceKind::Simulated,
            machine: MachineState {
                x_mm: None,
                z_mm: None,
                spindle_rpm: Some(-1.0),
                program_state: "idle".into(),
            },
            instruments: vec![],
            warnings: vec![],
        };
        assert_eq!(snapshot.validate(), Err("negative spindle speed"));
    }

    #[test]
    fn unknown_measurements_remain_absent_in_json() {
        let snapshot = BenchSnapshot {
            schema_version: SCHEMA_VERSION,
            tick: 0,
            source: SourceKind::Recorded,
            machine: MachineState {
                x_mm: None,
                z_mm: None,
                spindle_rpm: None,
                program_state: "unknown".into(),
            },
            instruments: vec![],
            warnings: vec![],
        };
        let value = serde_json::to_value(&snapshot).unwrap();
        assert!(value["machine"]["x_mm"].is_null());
        assert!(value["machine"]["spindle_rpm"].is_null());
        assert_eq!(snapshot.validate(), Ok(()));
    }
}
