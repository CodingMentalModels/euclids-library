use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Components

#[derive(Component, Clone)]
pub struct DialogComponent(pub Dialog);

// End Components

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

    pub fn story(speaker: String, contents: Vec<String>) -> Self {
        NPCDialog::story(speaker, contents).into()
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

    pub fn story(speaker: String, contents: Vec<String>) -> Self {
        if contents.len() == 0 {
            panic!("A story must have at least one element.");
        }

        if contents.len() == 1 {
            Self::leaf(speaker, contents[0].clone())
        } else {
            let current_contents = contents[0].clone();
            let additional_contents = contents[1..].to_vec();
            Self::new(
                speaker.clone(),
                current_contents,
                Some(Self::story(speaker, additional_contents).into()),
            )
        }
    }
}
