use std::path::{Path, PathBuf};
use std::fs;
use std::str::FromStr;
use anyhow::{Result, Context};
use global_hotkey::hotkey::Code;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::FromFileError;
use super::{SoundTree, SoundMap, SoundNode};

pub struct BadSoundError {
    pub path: PathBuf,
    pub error: FromFileError,
}

pub fn parse(entry_point: impl Into<PathBuf>, separator: char) -> Result<(SoundTree, Vec<BadSoundError>)> {
    let mut bad_sounds = Vec::new();
    parse_recursive(&entry_point.into(), separator, &mut bad_sounds)
        .map(|root| (SoundTree::new(root), bad_sounds))
}

fn parse_recursive(path: &Path, separator: char, bad_sounds: &mut Vec<BadSoundError>) -> Result<SoundMap> {
    let mut map = SoundMap::new();

    let dir_entries = fs::read_dir(path)?;
    for dir_entry in dir_entries {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        let file_type = dir_entry.file_type()?;

        if file_type.is_file() {
            let (code, name) = file_path_to_code_and_name(&path, separator)?;
            match StaticSoundData::from_file(&path, StaticSoundSettings::default()) {
                Ok(sound_data) => {
                    map.entry(code)
                        .and_modify(|sounds| sounds.insert_sound(name.clone(), sound_data.clone()))
                        .or_insert_with(|| SoundNode::leaf([(name, sound_data)]));
                },
                Err(error) => {
                    bad_sounds.push(BadSoundError {
                        path,
                        error,
                    });
                },
            }
        } else if file_type.is_dir() {
            let (code, name) = file_path_to_code_and_name(&path, separator)?;
            map.insert(code, SoundNode::branch(name, parse_recursive(&path, separator, bad_sounds)?));
        }
    }

    Ok(map)
}

fn custom_str_to_code(str: &str) -> Option<Code> {
    const NUM: &'static str = "num";
    if str.len() == 1 {
        let ch = str.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            Code::from_str(&format!("Key{}", ch.to_ascii_uppercase())).ok()
        } else if ch.is_ascii_digit() {
            Code::from_str(&format!("Digit{ch}")).ok()
        } else {
            None
        }
    } else if str.starts_with(NUM) {
        str[NUM.len()..]
            .chars()
            .next()
            .filter(|ch| ch.is_ascii_digit())
            .and_then(|ch| Code::from_str(&format!("Numpad{ch}")).ok())
    } else {
        None
    }
}

fn file_path_to_code_and_name(path: &Path, separator: char) -> Result<(Code, String)> {
    let file_name = path.file_stem()
        .context("Could not get file name")?
        .to_str()
        .context("Invalid file name")?;

    let split_index = file_name.find(separator);
    let code_str = &file_name[..split_index.unwrap_or(file_name.len())];
    let code = custom_str_to_code(code_str)
        .or_else(|| Code::from_str(code_str).ok())
        .context(format!("Could not convert '{code_str}' to key code"))?;

    let name = match split_index {
        None => String::new(),
        Some(index) => file_name[index + separator.len_utf8()..].to_owned(),
    };

    Ok((code, name))
}
