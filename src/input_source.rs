use anyhow::Context;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString, FromRepr};

/// The raw representation of an input source value.
/// See also [`InputSource`].
pub type InputSourceRaw = u8;

#[derive(Copy, Clone, Debug, PartialEq, AsRefStr, EnumString, FromRepr)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
/// An input source value.
/// See also [`InputSourceRaw`].
pub enum InputSource {
    #[strum(serialize = "DP1")]
    DisplayPort1 = 0x0F,
    #[strum(serialize = "DP2")]
    DisplayPort2 = 0x10,
    Hdmi1 = 0x11,
    Hdmi2 = 0x12,
    UsbC1 = 0x19,
    UsbC2 = 0x1B,
}

impl InputSource {
    /// Get [`InputSourceRaw`].
    /// ```
    /// # use monitor_input::InputSource;
    /// assert_eq!(InputSource::Hdmi1.as_raw(), 17);
    /// ```
    pub fn as_raw(self) -> InputSourceRaw {
        self as InputSourceRaw
    }

    /// Get [`InputSourceRaw`] from a string.
    /// The string is either the name of an [`InputSource`] or a number.
    /// # Examples
    /// ```
    /// # use monitor_input::InputSource;
    /// // Input strings are either an [`InputSource`] or a number.
    /// assert_eq!(
    ///     InputSource::raw_from_str("Hdmi1").unwrap(),
    ///     InputSource::Hdmi1.as_raw()
    /// );
    /// assert_eq!(InputSource::raw_from_str("27").unwrap(), 27);
    ///
    /// // Undefined string will be an error.
    /// assert!(InputSource::raw_from_str("xyz").is_err());
    /// // The error message should contain the original string.
    /// assert!(
    ///     InputSource::raw_from_str("xyz")
    ///         .unwrap_err()
    ///         .to_string()
    ///         .contains("xyz")
    /// );
    /// ```
    pub fn raw_from_str(input: &str) -> anyhow::Result<InputSourceRaw> {
        if let Ok(value) = input.parse::<InputSourceRaw>() {
            return Ok(value);
        }
        InputSource::from_str(input)
            .map(|value| value.as_raw())
            .with_context(|| format!("\"{input}\" is not a valid input source"))
    }

    /// Get a string from [`InputSourceRaw`].
    /// # Examples
    /// ```
    /// # use monitor_input::InputSource;
    /// assert_eq!(InputSource::str_from_raw(InputSource::Hdmi1.as_raw()), "Hdmi1");
    /// assert_eq!(InputSource::str_from_raw(17), "Hdmi1");
    /// assert_eq!(InputSource::str_from_raw(255), "255");
    /// ```
    pub fn str_from_raw(value: InputSourceRaw) -> String {
        match InputSource::from_repr(value) {
            Some(input_source) => input_source.as_ref().to_string(),
            None => value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_source_from_str() {
        assert_eq!(InputSource::from_str("Hdmi1"), Ok(InputSource::Hdmi1));
        // Test `ascii_case_insensitive`.
        assert_eq!(InputSource::from_str("hdmi1"), Ok(InputSource::Hdmi1));
        assert_eq!(InputSource::from_str("HDMI1"), Ok(InputSource::Hdmi1));
        // Test `serialize`.
        assert_eq!(InputSource::from_str("DP1"), Ok(InputSource::DisplayPort1));
        assert_eq!(InputSource::from_str("dp2"), Ok(InputSource::DisplayPort2));
        // Test failures.
        assert!(InputSource::from_str("xyz").is_err());
    }
}
