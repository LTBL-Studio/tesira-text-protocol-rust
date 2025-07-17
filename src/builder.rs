//! Command builder helper

use std::{error::Error, fmt::Display, ops::Deref, time::Duration};

use crate::proto::{InstanceTag, Command, IndexValue, IntoTTP, commands::*};
use chrono::naive::NaiveDateTime;

/// Helper to construct valid Tesira Commands
pub struct CommandBuilder;

impl CommandBuilder {
    /// Create a new command builder
    pub fn new() -> Self {
        CommandBuilder
    }
}

/// Value of a delay in Tesira system
pub enum DelayValue {
    /// A delay in miliseconds
    Milliseconds(Duration),
    /// A delay in centimeters
    Centimeters(f64),
    /// A delay in meters
    Meters(f64),
    /// A delay in inches
    Inches(f64),
    /// A delay in feets
    Feet(f64),
}

impl IntoTTP for DelayValue {
    fn into_ttp(self) -> String {
        match self {
            DelayValue::Milliseconds(v) => format!("{{\"units\":Milliseconds \"delay\":{}}}", v.as_millis().into_ttp()),
            DelayValue::Centimeters(v) => format!("{{\"units\":Centimeters \"delay\":{}}}", v.into_ttp()),
            DelayValue::Meters(v) => format!("{{\"units\":Meters \"delay\":{}}}", v.into_ttp()),
            DelayValue::Inches(v) => format!("{{\"units\":Inches \"delay\":{}}}", v.into_ttp()),
            DelayValue::Feet(v) => format!("{{\"units\":Feet \"delay\":{}}}", v.into_ttp()),
        }
    }
}

/// A Tesira type of filter
pub enum FilterType {
    /// Butterworth filter
    Butterworth,
    /// Linksitz-Riley filter
    LinkwitzRiley,
    /// Bessel filter
    Bessel,
}

impl IntoTTP for FilterType {
    fn into_ttp(self) -> String {
        match self {
            FilterType::Butterworth => "Butterworth".to_owned(),
            FilterType::LinkwitzRiley => "Linkwitz-Riley".to_owned(),
            FilterType::Bessel => "Bessel".to_owned(),
        }
    }
}

/// Slope of filter
#[derive(Debug)]
pub struct FilterSlope(u64);

/// Supported filter slopes
const VALID_SLOPES: [u64; 8] = [6,12,18,24,30,36,42,48];

impl FilterSlope {
    /// Create a new slope from a value, checking if this slope is supported
    pub fn new(slope: u64) -> Result<FilterSlope, InvalidSlopeError> {
        if ! VALID_SLOPES.contains(&slope) {
            return Err(InvalidSlopeError)
        }
        Ok(Self(slope))
    }

    /// A slope of 6
    pub const SIX: Self = Self(6);
    /// A slope of 12
    pub const TWELVE: Self = Self(12);
    /// A slope of 18
    pub const HEIGHTEEN: Self = Self(18);
    /// A slope of 24
    pub const TWENTYFOUR: Self = Self(24);
    /// A slope of 30
    pub const THIRTY: Self = Self(30);
    /// A slope of 36
    pub const THIRTYSIX: Self = Self(36);
    /// A slope of 42
    pub const FOURTYTWO: Self = Self(42);
    /// A slope of 48
    pub const FOURTYHEIGHT: Self = Self(48);
}

impl Deref for FilterSlope {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoTTP for FilterSlope {
    fn into_ttp(self) -> String {
        self.0.into_ttp()
    }
}

/// Provided slope value is invalid
#[derive(Debug)]
pub struct InvalidSlopeError;

impl Error for InvalidSlopeError {}

impl Display for InvalidSlopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid slope, allowed slopes are {}", VALID_SLOPES.map(|it| it.to_string()).join(", "))
    }
}

include!("../generated/tesira-blocks.rs");