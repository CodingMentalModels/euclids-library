use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dialog {
    NPCDialog(Box<NPCDialog>),
    PlayerDialog(Box<Vec<(String, Dialog)>>),
}

impl From<NPCDialog> for Dialog {
    fn from(value: NPCDialog) -> Self {
        Self::NPCDialog(Box::new(value))
    }
}

impl Dialog {
    pub fn new_npc(speaker: String, contents: String, next: Option<Dialog>) -> Self {
        Self::NPCDialog(Box::new(NPCDialog::new(speaker, contents, next)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NPCDialog {
    speaker: String,
    contents: String,
    next: Option<Dialog>,
}

impl NPCDialog {
    pub fn new(speaker: String, contents: String, next: Option<Dialog>) -> Self {
        Self {
            speaker,
            contents,
            next,
        }
    }

    pub fn leaf(speaker: String, contents: String) -> Self {
        Self::new(speaker, contents, None)
    }
}
