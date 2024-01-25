use std::{borrow::Cow, collections::BTreeMap};

use global_hotkey::hotkey::Code;

use crate::soundtree::{SoundMap, SoundNode};

pub fn print_nav(map: &SoundMap, key_toggle: Option<Code>) {
    let mut display_map = BTreeMap::new();
    for (&code, node) in map {
        display_map.insert(code_to_string(code), node);
    }
    for (display_str, node) in display_map {
        let name = match node {
            SoundNode::Leaf(sounds) => format!("{} ({} variations)", sounds[0].0.as_str(), sounds.len()),
            SoundNode::Branch((name, sub_map)) => format!("{name} ({} key binds)", sub_map.len()),
        };
        println!("[{display_str}] {name}");
    }
    if let Some(key_toggle) = key_toggle {
        println!("\nPress [{key_toggle}] to toggle soundboard off.")
    }
}

pub fn code_to_string<'a>(code: Code) -> Cow<'a, str> {
    match code {
        Code::Digit0 => "0".into(),
        Code::Digit1 => "1".into(),
        Code::Digit2 => "2".into(),
        Code::Digit3 => "3".into(),
        Code::Digit4 => "4".into(),
        Code::Digit5 => "5".into(),
        Code::Digit6 => "6".into(),
        Code::Digit7 => "7".into(),
        Code::Digit8 => "8".into(),
        Code::Digit9 => "9".into(),
        Code::Numpad0 => "Num 0".into(),
        Code::Numpad1 => "Num 1".into(),
        Code::Numpad2 => "Num 2".into(),
        Code::Numpad3 => "Num 3".into(),
        Code::Numpad4 => "Num 4".into(),
        Code::Numpad5 => "Num 5".into(),
        Code::Numpad6 => "Num 6".into(),
        Code::Numpad7 => "Num 7".into(),
        Code::Numpad8 => "Num 8".into(),
        Code::Numpad9 => "Num 9".into(),
        Code::KeyA => "A".into(),
        Code::KeyB => "B".into(),
        Code::KeyC => "C".into(),
        Code::KeyD => "D".into(),
        Code::KeyE => "E".into(),
        Code::KeyF => "F".into(),
        Code::KeyG => "G".into(),
        Code::KeyH => "H".into(),
        Code::KeyI => "I".into(),
        Code::KeyJ => "J".into(),
        Code::KeyK => "K".into(),
        Code::KeyL => "L".into(),
        Code::KeyM => "M".into(),
        Code::KeyN => "N".into(),
        Code::KeyO => "O".into(),
        Code::KeyP => "P".into(),
        Code::KeyQ => "Q".into(),
        Code::KeyR => "R".into(),
        Code::KeyS => "S".into(),
        Code::KeyT => "T".into(),
        Code::KeyU => "U".into(),
        Code::KeyV => "V".into(),
        Code::KeyW => "W".into(),
        Code::KeyX => "X".into(),
        Code::KeyY => "Y".into(),
        Code::KeyZ => "Z".into(),
        _ => format!("{code}").into(),
    }
}

pub fn print_soundboard_inactive(toggle_code: Code) {
    println!("Soundboard is off.\nPress [{toggle_code}] to toggle it back on.");
}
