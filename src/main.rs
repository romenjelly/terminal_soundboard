use std::{
    env,
    io,
    path::{PathBuf, Path},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    collections::HashMap,
};

use anyhow::{Result, Context};
use cpal::{
    traits::{HostTrait, DeviceTrait},
    Device,
};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent,
    GlobalHotKeyManager,
    HotKeyState,
};
use winit::event_loop::{
    ControlFlow,
    EventLoopBuilder,
    EventLoopWindowTarget,
};
use kira::{
    manager::{
        AudioManager,
        AudioManagerSettings,
        backend::{DefaultBackend, cpal::CpalBackendSettings},
    },
    track::TrackBuilder,
    tween::Tween,
};
use kira::sound::{
    PlaybackState,
    static_sound::StaticSoundHandle,
};
use rand::Rng;
use clap::Parser;

mod soundtree;
use soundtree::{
    SoundMap,
    SoundNode,
    parser::BadSoundError,
};

mod config;
use config::{Settings, OverlapBehavior};

mod display;

type Nav = Vec<(Code, HotKey)>;
type Handles = Vec<(StaticSoundHandle, Option<StaticSoundHandle>)>;

enum Command {
    /// `(main sound handle, monitor sound handle)`
    Play((StaticSoundHandle, Option<StaticSoundHandle>)),
    Reset,
    NoChange,
}

fn main() -> Result<()> {
    let settings = Settings::parse();

    let audio = AudioManager::<DefaultBackend>::new(AudioManagerSettings {
        backend_settings: CpalBackendSettings {
            device: select_output_device("main output device")?,
            ..Default::default()
        },
        main_track_builder: TrackBuilder::new().volume(settings.volume),
        ..Default::default()
    })?;

    let monitor = if let Some(volume) = settings.monitor {
        AudioManager::<DefaultBackend>::new(AudioManagerSettings {
            backend_settings: CpalBackendSettings {
                device: select_output_device("monitor output device")?,
                ..Default::default()
            },
            main_track_builder: TrackBuilder::new().volume(volume.unwrap_or(1.0_f64)),
            ..Default::default()
        }).ok()
    } else {
        None
    };

    run(
        audio,
        monitor,
        &settings,
        make_absolute_path(&settings.board),
    )
}

fn run(
    mut audio: AudioManager,
    mut monitor: Option<AudioManager>,
    settings: &Settings,
    path: impl Into<PathBuf>,
) -> Result<()> {
    let (tree, bad_sounds) = soundtree::parser::parse(path, settings.separator)?;

    if bad_sounds.len() > 0 {
        println!("Found {} errors whilst loading sounds:\n", bad_sounds.len());
        for BadSoundError { path, error } in bad_sounds {
            println!("Path: {}\nError: {error}\n", path.display());
        }
        println!("Press Enter to continue, or CTRL + C to terminate");
        io::stdin().read_line(&mut String::new()).unwrap();
    }

    let event_loop = EventLoopBuilder::new().build()?;
    let hotkey_manager = GlobalHotKeyManager::new()?;
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();

    let root_hotkey_storage = hotkey_variations(settings.key_root)
        .into_iter()
        .map(|hotkey| (hotkey.id(), hotkey))
        .collect::<HashMap<_, _>>();
    let root_hotkeys = &root_hotkey_storage;
    let stop_hotkey_storage = register_optional_hotkey_map(&hotkey_manager, settings.key_stop)?;
    let stop_hotkeys = stop_hotkey_storage.as_ref();
    let toggle_hotkey_storage = register_optional_hotkey_map(&hotkey_manager, settings.key_toggle)?;
    let toggle_hotkeys = toggle_hotkey_storage.as_ref();

    let should_terminate = Arc::new(AtomicBool::new(false));
    {
        let should_terminate = should_terminate.clone();
        ctrlc::set_handler(move || {
            should_terminate.store(true, Ordering::Relaxed);
        })?;
    }

    let mut soundboard_active = true;
    let mut nav_storage = Nav::new();
    let nav = &mut nav_storage;
    register_nav(&hotkey_manager, tree.root(), nav)?;
    let root = tree.root();
    let mut walker = tree.root();
    let mut sound_index_map = HashMap::new();
    let mut handles = Handles::new();
    display::print_nav(walker, settings.key_toggle);
    event_loop.run(move |_event, event_loop: &EventLoopWindowTarget<()>| {
        event_loop.set_control_flow(ControlFlow::Poll);

        if should_terminate.load(Ordering::Relaxed) {
            event_loop.exit();
            return;
        }

        let Ok(event) = global_hotkey_channel.try_recv() else {
            std::thread::sleep(std::time::Duration::from_millis(16));
            return;
        };
        if event.state() != HotKeyState::Pressed { return };

        handles.retain(|(handle, _)| matches!(handle.state(), PlaybackState::Playing));
        let command = (|| -> Result<Command> {
            if root_hotkeys.contains_key(&event.id()) {
                return Ok(Command::Reset);
            }
            if stop_hotkeys.map(|hotkeys| hotkeys.contains_key(&event.id())).unwrap_or(false) {
                stop_playback(&mut handles)?;
                return Ok(Command::NoChange);
            }
            if toggle_hotkeys.map(|hotkeys| hotkeys.contains_key(&event.id())).unwrap_or(false) {
                if soundboard_active {
                    unregister_nav(&hotkey_manager, nav)?;
                } else {
                    register_nav(&hotkey_manager, root, nav)?;
                }
                stop_playback(&mut handles)?;
                soundboard_active =! soundboard_active;
                return Ok(Command::Reset);
            }
            if !soundboard_active {
                return Ok(Command::NoChange);
            }
            let found_code = nav.iter()
                .find(|(_, hotkey)| hotkey.id() == event.id())
                .map(|(code, _)| *code);
            let Some(code) = found_code else {
                println!("Pressing keys too fast, a hotkey wasn't properly handled!");
                return Ok(Command::NoChange);
            };
            match walker.get(&code).context("Failed to get node")? {
                SoundNode::Leaf(sounds) => {
                    if settings.overlap == OverlapBehavior::Prevent {
                        if !handles.is_empty() { return Ok(Command::Reset) };
                    }
                    let index = *sound_index_map.entry(sound_id(code, sounds[0].0.as_str())).and_modify(|index| {
                        *index += if sounds.len() > 2 {
                            rand::thread_rng().gen_range(1..sounds.len())
                        } else {
                            1
                        };
                        *index %= sounds.len();
                    }).or_insert(0);
                    let (_, sound) = sounds.get(index).context("No sounds to choose from")?;
                    let new_handle = audio.play(sound.clone())?;
                    let monitor_handle = if let Some(monitor) = monitor.as_mut() {
                        Some(monitor.play(sound.clone())?)
                    } else { None };
                    return Ok(Command::Play((new_handle, monitor_handle)));
                },
                SoundNode::Branch((_, sub_map)) => {
                    for root_hotkey in root_hotkeys.values().copied() {
                        let _ = hotkey_manager.register(root_hotkey);
                    }
                    replace_nav(&hotkey_manager, sub_map, nav)?;
                    walker = sub_map;
                },
            }
            Ok(Command::NoChange)
        })().expect("Fatal soundboard failure");

        let reset_to_root = match command {
            Command::Play(sound) => {
                if settings.overlap == OverlapBehavior::Replace {
                    stop_playback(&mut handles).expect("Failed to stop playback");
                }
                handles.push(sound);
                true
            },
            Command::Reset => true,
            Command::NoChange => false,
        };
        if reset_to_root {
            unregister_hotkeys_fallible(&hotkey_manager, root_hotkeys.values().copied());
            if soundboard_active {
                replace_nav(&hotkey_manager, root, nav).expect("Failed to reset navigation");
            }
            if let Some(stop_hotkeys) = stop_hotkeys {
                if soundboard_active {
                    for &hotkey in stop_hotkeys.values() {
                        let _ = hotkey_manager.register(hotkey);
                    }
                } else {
                    unregister_hotkeys_fallible(&hotkey_manager, stop_hotkeys.values().copied());
                }
            }
            walker = root;
        }

        if settings.no_clear {
            println!()
        } else {
            clearscreen::clear().expect("Failed to clear screen");
        }
        if soundboard_active {
            display::print_nav(walker, settings.key_toggle);
        } else {
            let key_toggle = settings.key_toggle.expect("This state should be unreachable without a set key");
            display::print_soundboard_inactive(key_toggle);
        }
    })?;

    println!("Stopping soundboard");
    if let Ok(hotkey_manager) = GlobalHotKeyManager::new() {
        unregister_hotkeys_fallible(&hotkey_manager, root_hotkey_storage.into_values());
        if let Some(stop_hotkeys) = stop_hotkey_storage {
            unregister_hotkeys_fallible(&hotkey_manager, stop_hotkeys.into_values());
        }
        if let Some(toggle_hotkeys) = toggle_hotkey_storage {
            unregister_hotkeys_fallible(&hotkey_manager, toggle_hotkeys.into_values());
        }
        let _ = unregister_nav(&hotkey_manager, &mut nav_storage);
    }

    Ok(())
}

fn make_absolute_path(relative_or_absolute_path: &str) -> PathBuf {
    let input_path = Path::new(relative_or_absolute_path);

    if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        exe_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(input_path)
    }
}

fn select_output_device(selection_prompt: &str) -> Result<Option<Device>> {
    let host = cpal::default_host();
    let devices = host.output_devices()?.collect::<Vec<_>>();
    println!("Select {selection_prompt} (press enter to select default)");
    for (index, device) in devices.iter().enumerate() {
        println!("{}. {}", index + 1, device.name()?);
    }
    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    if selection.trim().len() == 0 {
        Ok(None)
    } else {
        let selected: usize = selection.trim().parse()?;
        let device = devices.into_iter()
            .nth(selected - 1)
            .context("Was unable to acquire device")?;
        Ok(Some(device))
    }
}

fn hotkey_variations(code: Code) -> [HotKey; 4] {
    [
        Modifiers::empty(),
        Modifiers::CONTROL,
        Modifiers::SHIFT,
        Modifiers::CONTROL | Modifiers::SHIFT,
    ].map(|modifier| HotKey::new(Some(modifier), code))
}

fn register_hotkey(hotkey_manager: &GlobalHotKeyManager, code: Code, mut post_process: impl FnMut(HotKey)) -> Result<()> {
    for hotkey in hotkey_variations(code) {
        hotkey_manager.register(hotkey)?;
        post_process(hotkey);
    }
    Ok(())
}

fn register_nav(hotkey_manager: &GlobalHotKeyManager, map: &SoundMap, nav: &mut Nav) -> Result<()> {
    for &code in map.keys() {
        register_hotkey(hotkey_manager, code, |hotkey| {
            nav.push((code, hotkey));
        })?;
    }
    Ok(())
}

fn unregister_nav(hotkey_manager: &GlobalHotKeyManager, nav: &mut Nav) -> Result<()> {
    for (_, hotkey) in nav.iter() {
        hotkey_manager.unregister(*hotkey)?;
    }
    nav.clear();
    Ok(())
}

fn replace_nav(hotkey_manager: &GlobalHotKeyManager, map: &SoundMap, nav: &mut Nav) -> Result<()> {
    unregister_nav(hotkey_manager, nav)?;
    register_nav(hotkey_manager, map, nav)?;
    Ok(())
}

fn register_optional_hotkey_map(hotkey_manager: &GlobalHotKeyManager, code: Option<Code>) -> Result<Option<HashMap<u32, HotKey>>> {
    if let Some(code) = code {
        let mut map = HashMap::new();
        register_hotkey(hotkey_manager, code, |hotkey| {
            map.insert(hotkey.id(), hotkey);
        })?;
        Ok(Some(map))
    } else { Ok(None) }
}

fn unregister_hotkeys_fallible(hotkey_manager: &GlobalHotKeyManager, hotkeys: impl Iterator<Item = HotKey>) {
    for hotkey in hotkeys {
        let _ = hotkey_manager.unregister(hotkey);
    }
}

fn sound_id(code: Code, sound_name: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hasher, Hash};

    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    sound_name.hash(&mut hasher);
    hasher.finish()
}

fn stop_playback(handles: &mut Handles) -> Result<()> {
    for (mut sound_handle, monitor_sound_handle) in handles.drain(..) {
        sound_handle.stop(Tween::default())?;
        if let Some(mut sound_handle) = monitor_sound_handle {
            sound_handle.stop(Tween::default())?;
        }
    }
    Ok(())
}
