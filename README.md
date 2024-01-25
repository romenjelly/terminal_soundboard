# Terminal Soundboard

A soundboard that runs in your terminal with straightforward configuration.

Uses the `kira` audio [crate](https://github.com/tesselode/kira) to load and play sounds, supports: `.mp3`, `.ogg`, `.flac`, `.wav`.

## Be Careful!

This soundboard intercepts keys **in a destructive manner**!

All keys bound to the current playable sounds will be "consumed" by the soundboard, and thus not propagated to your currently focused program.
This means that it may introduce **spelling mistakes when typing**, and **block crucial actions when playing games**.

An option to toggle the soundboard on/off is available, and the [`Escape`] key is bound to "return to the root of the soundboard" by default (also configurable).

Moreover, improper termination might result in hotkeys still being intercepted despite not seeing the soundboard.  
To mitigate this, find and terminate the soundboard in your OS's process manager.

Lastly, repeated improper termination may trip anti-malware software into flagging, and potentially deleting the soundboard executable. If it is randomly gone, it has likely been deleted by anti-malware software.

This codebase is very hastily put together and runs on shaky control flow. Functions may seem nonsensical and randomly strum around the modules; yet *it works on my machine*.  
This repository exists purely by "popular demand", and shouldn't be treated as anything more than haphazardly thrown-together crates to make a computer play funny sounds.

## Configuring the Soundboard

Configuration is done two-fold: using the file system, and parameters passed to the executable.

### Configuring Sounds

This soundboard uses the file system for configuring sounds.

From the specified directory, a tree is built on the following principles:

- Each node has a key
- Each node has a name
- Sound file nodes can have multiple variations (collapsed via key)
- Sound folder nodes lead to more (sound file or folder) nodes

The key and the name is separated by a configurable separator, default being '`_`'.

The key is either a:

- Single character (case-insensitive)
- Single digit
- "num" followed by a single digit (numpad binding)
- Hotkey code if not covered by the above cases

Hotkey codes are parsed using the `FromStr` trait implementation of `keyboard_types::Code`.  
Their full "list" can be found [here](https://github.com/pyfisch/keyboard-types/blob/main/src/code.rs#L690).

All keys will also be intercepted regardless of the [`Shift`] and [`Ctrl`] modifiers.  
The [`Alt`] modifier is not taken into account.

*Note:* This decision has been made in favor of parsing using `global_hotkeys::hotkey::Hotkey::try_from(&str)` for the two following reasons:

- This soundboard was developed first and foremost for games, meaning interception with a default set of captured modifiers.
- Simplicity of configuration being a core principle, hence the dual-possibility parsing. This helps avoid renaming fatigue.

### Configuring Parameters

A description of usable parameters is available by launching the executable with the `-h` or `--help` flag.

Only one parameter is required when launching a soundboard, being the path to the root of the soundboard tree: `-b <PATH>` or `--board <PATH>`.  
If the path is not absolute, it will be relative **to the executable**.

| Short | Long           | Description                                            | Default Value | Example                |
|-------|----------------|--------------------------------------------------------|:-------------:|------------------------|
| `-b`  | `--board`      | Path the the soundboard                                |               | `-b memes`             |
| `-v`  | `--volume`     | Main output volume                                     |      1.0      | `-v 0.5`               |
| `-s`  | `--separator`  | Separator character                                    |     '`_`'     | `-s " "`               |
| `-o`  | `--overlap`    | Overlap behavior (either: prevent, overlap, replace)   |    prevent    | `-o overlap`           |
| `-m`  | `--monitor`    | Monitor device and its (optional) volume               |               | `-m 0.25`              |
|       | `--key-stop`   | Key to interrupt all sound playback                    |               | `--key-stop End`       |
|       | `--key-toggle` | Key to toggle the soundboard on and off                |               | `--key-toggle Numpad0` |
|       | `--key-root`   | Key to navigate to the root of the soundboard          |   [`Escape`]  | `--key-root PageUp`    |
|       | `--no-clear`   | Whether to prevent clearing the screen upon navigation |               | `--no-clear`           |

### Example Configurations

It is recommended to make executable scripts to launch the soundboard, such as `.sh` on Unix systems, and `.bat` on Windows.

#### Example: Replicating Mordhau

In the game [Mordhau](https://store.steampowered.com/app/629760/MORDHAU/), voice lines can be pressed by pressing the [`C`] key. A different selection appears if the key is pressed more than once.

Here is how the file structure for that game's (incomplete) soundboard would look on a Windows machine:

```
soundboard.exe
mordhau_eager.bat
mordhau_foppish.bat
mordhau/
├── foppish/
│   ├── c_Voicelines/
│   │   ├── 1_Yes.wav
│   │   ├── 1_Yes2.wav
│   │   ├── 2_No.wav
│   │   ├── 2_No2.wav
│   │   ├── 3_Help.wav
│   │   ├── 3_Help2.wav
│   │   ├── 4_Insult.wav
│   │   ├── 4_Insult2.wav
│   │   └── c_More Voicelines/
│   │       ├── 1_Sorry.wav
│   │       ├── 1_Sorry2.wav
│   │       ├── 2_Laugh.wav
│   │       ├── 2_Laugh2.wav
│   │       └── c_Even More Voicelines/
│   │           ├── 1_Hold1.wav
│   │           ├── 1_Hold2.wav
│   │           ├── 2_Hello.wav
│   │           └── 2_Hello2.wav
│   ├── v_Battlecry.wav
│   ├── v_Battlecry2.wav
│   └── v_Battlecry3.wav
└── eager/
    └── (same structure as foppish)
```

Here's what the `mordhau+foppish.bat` might may look like:

```bat
./soundboard.exe -b mordhau/foppish -m 0.25 --key-toggle Home --key-stop End
```
