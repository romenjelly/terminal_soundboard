use std::collections::HashMap;
use global_hotkey::hotkey::Code;
use kira::sound::static_sound::StaticSoundData;

pub mod parser;

pub type SoundMap = HashMap<Code, SoundNode>;

pub enum SoundNode {
    Leaf(Vec<(String, StaticSoundData)>),
    Branch((String, SoundMap)),
}

impl SoundNode {
    pub fn leaf(sounds: impl IntoIterator<Item = (String, StaticSoundData)>) -> Self {
        Self::Leaf(sounds.into_iter().collect())
    }

    pub fn insert_sound(&mut self, name: impl Into<String>, sound: StaticSoundData) {
        match self {
            Self::Leaf(sounds) => sounds.push((name.into(), sound)),
            Self::Branch(_) => { },
        }
    }

    pub fn branch(name: String, map: SoundMap) -> Self {
        Self::Branch((name, map))
    }
}

pub struct SoundTree {
    root: SoundMap,
}

impl SoundTree {
    pub fn new(tree: SoundMap) -> Self {
        Self {
            root: tree,
        }
    }

    pub fn root(&self) -> &SoundMap {
        &self.root
    }
}
