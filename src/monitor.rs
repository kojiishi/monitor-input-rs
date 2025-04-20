use std::time::Instant;

use super::*;
use ddc_hi::{Ddc, DdcHost, FeatureCode};
use log::*;

/// VCP feature code for input select
const INPUT_SELECT: FeatureCode = 0x60;

static mut DRY_RUN: bool = false;

/// Represents a display monitor.
/// # Examples
/// ```no_run
/// # use monitor_input::{InputSource,Monitor};
/// let mut monitors = Monitor::enumerate();
/// monitors[0].set_current_input_source(InputSource::UsbC1.as_raw());
/// ```
pub struct Monitor {
    ddc_hi_display: ddc_hi::Display,
    is_capabilities_updated: bool,
    needs_sleep: bool,
}

impl std::fmt::Display for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ddc_hi_display.info.id)
    }
}

impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Monitor")
            .field("info", &self.ddc_hi_display.info)
            .finish()
    }
}

impl Monitor {
    /// Create an instance from [`ddc_hi::Display`].
    pub fn new(ddc_hi_display: ddc_hi::Display) -> Self {
        Monitor {
            ddc_hi_display: ddc_hi_display,
            is_capabilities_updated: false,
            needs_sleep: false,
        }
    }

    /// Enumerate all display monitors.
    /// See also [`ddc_hi::Display::enumerate()`].
    pub fn enumerate() -> Vec<Self> {
        ddc_hi::Display::enumerate()
            .into_iter()
            .map(|d| Monitor::new(d))
            .collect()
    }

    fn is_dry_run() -> bool {
        unsafe { return DRY_RUN }
    }

    /// Set the dry-run mode.
    /// When in dry-run mode, functions that are supposed to make changes
    /// don't actually make the changes.
    pub fn set_dry_run(value: bool) {
        unsafe { DRY_RUN = value }
    }

    /// Updates the display info with data retrieved from the device's
    /// reported capabilities.
    /// See also [`ddc_hi::Display::update_capabilities()`].
    pub fn update_capabilities(&mut self) -> anyhow::Result<()> {
        if self.is_capabilities_updated {
            return Ok(());
        }
        self.is_capabilities_updated = true;
        debug!("update_capabilities({self})");
        let start_time = Instant::now();
        let result = self
            .ddc_hi_display
            .update_capabilities()
            .inspect_err(|e| warn!("{self}: Failed to update capabilities: {e}"));
        debug!(
            "update_capabilities({self}) elapsed: {:?}",
            start_time.elapsed()
        );
        result
    }

    pub fn contains_backend(&self, backend: &str) -> bool {
        self.ddc_hi_display
            .info
            .backend
            .to_string()
            .contains(backend)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.ddc_hi_display.info.id.contains(name)
    }

    fn feature_descriptor(&self, feature_code: FeatureCode) -> Option<&mccs_db::Descriptor> {
        self.ddc_hi_display.info.mccs_database.get(feature_code)
    }

    fn feature_code(&self, feature_code: FeatureCode) -> FeatureCode {
        // TODO: `mccs_database` is initialized by `display.update_capabilities()`
        // which is quite slow, and it seems to work without this.
        // See also https://github.com/mjkoo/monitor-switch/blob/master/src/main.rs.
        if let Some(feature) = self.feature_descriptor(feature_code) {
            return feature.code;
        }
        feature_code
    }

    /// Get the current input source.
    pub fn current_input_source(&mut self) -> anyhow::Result<InputSourceRaw> {
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        Ok(self.ddc_hi_display.handle.get_vcp_feature(feature_code)?.sl)
    }

    /// Set the current input source.
    pub fn set_current_input_source(&mut self, value: InputSourceRaw) -> anyhow::Result<()> {
        info!(
            "InputSource({self}) = {value}{mode}",
            value = InputSource::str_from_raw(value),
            mode = if Self::is_dry_run() { " (dry-run)" } else { "" }
        );
        if Self::is_dry_run() {
            return Ok(());
        }
        let feature_code: FeatureCode = self.feature_code(INPUT_SELECT);
        self.ddc_hi_display
            .handle
            .set_vcp_feature(feature_code, value as u16)
            .inspect(|_| self.needs_sleep = true)
    }

    /// Get all input sources.
    /// Requires to call [`Monitor::update_capabilities()`] beforehand.
    pub fn input_sources(&mut self) -> Option<Vec<InputSourceRaw>> {
        if let Some(feature) = self.feature_descriptor(INPUT_SELECT) {
            debug!("INPUT_SELECT({self}) = {feature:?}");
            if let mccs_db::ValueType::NonContinuous { values, .. } = &feature.ty {
                return Some(values.keys().cloned().collect());
            }
        }
        None
    }

    /// Sleep if any previous DDC commands need time to be executed.
    /// See also [`ddc_hi::DdcHost::sleep()`].
    pub fn sleep_if_needed(&mut self) {
        if self.needs_sleep {
            debug!("sleep({self})");
            let start_time = Instant::now();
            self.needs_sleep = false;
            self.ddc_hi_display.handle.sleep();
            debug!("sleep({self}) elapsed {:?}", start_time.elapsed());
        }
    }

    /// Get a multi-line descriptive string.
    pub fn to_long_string(&mut self) -> String {
        let mut lines = Vec::new();
        lines.push(self.to_string());
        let input_source = self.current_input_source();
        lines.push(format!(
            "Input Source: {}",
            match input_source {
                Ok(value) => InputSource::str_from_raw(value),
                Err(e) => e.to_string(),
            }
        ));
        if let Some(input_sources) = self.input_sources() {
            lines.push(format!(
                "Input Sources: {}",
                input_sources
                    .iter()
                    .map(|value| InputSource::str_from_raw(*value))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some(model) = &self.ddc_hi_display.info.model_name {
            lines.push(format!("Model: {}", model));
        }
        lines.push(format!("Backend: {}", self.ddc_hi_display.info.backend));
        return lines.join("\n    ");
    }
}
