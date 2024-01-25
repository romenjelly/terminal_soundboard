use std::{error::Error, fmt, str::FromStr};

use clap::Parser;
use global_hotkey::hotkey::Code;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Settings {
    /// Path to the soundboard
    #[arg(short, long, name = "PATH")]
    pub board: String,
    /// Main output volume, value ranging from 0 to 1
    #[arg(short, long, name = "SOUND VOLUME", default_value_t = 1.0_f64)]
    pub volume: f64,
    /// Separator character, which separates the key and the sound name
    #[arg(short, long, name = "CHARACTER", default_value_t = '_')]
    pub separator: char,
    /// Overlap behavior. Possible values: prevent, overlap, replace
    #[arg(short, long, name = "OVERLAP", default_value_t = OverlapBehavior::default())]
    pub overlap: OverlapBehavior,
    /// Whether to create a monitor, with volume ranging from 0 to 1 [default: 1]
    #[arg(short, long, name = "MONITOR VOLUME")]
    pub monitor: Option<Option<f64>>,

    /// A key to interrupt playback of the currently playing sound
    #[arg(long, name = "KEY TO STOP")]
    pub key_stop: Option<Code>,
    /// A key to toggle the soundboard working
    #[arg(long, name = "KEY TO TOGGLE")]
    pub key_toggle: Option<Code>,
    /// A key to return to the root of the soundboard
    #[arg(long, name = "KEY TO ROOT", default_value_t = Code::Escape)]
    pub key_root: Code,

    /// Whether to prevent clearing the screen upon navigation
    #[arg(long)]
    pub no_clear: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum OverlapBehavior {
    #[default]
    Prevent,
    Overlap,
    Replace,
}

impl FromStr for OverlapBehavior {
    type Err = OverlapNotFound;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prevent" => Ok(OverlapBehavior::Prevent),
            "overlap" => Ok(OverlapBehavior::Overlap),
            "replace" => Ok(OverlapBehavior::Replace),
            _ => Err(OverlapNotFound),
        }
    }
}

impl fmt::Display for OverlapBehavior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = match self {
            OverlapBehavior::Prevent => "prevent",
            OverlapBehavior::Overlap => "overlap",
            OverlapBehavior::Replace => "replace",
        };
        f.write_str(data)
    }
}

#[derive(Debug)]
pub struct OverlapNotFound;

impl fmt::Display for OverlapNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown overlap")
    }
}

impl Error for OverlapNotFound { }
