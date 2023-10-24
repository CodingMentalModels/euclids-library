use serde::{Deserialize, Serialize};

// Components

// End Components

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dialog {
    NPCDialog(Box<NPCDialog>),
    PlayerDialog(Box<Vec<(String, Option<Dialog>)>>),
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

    pub fn update_leaf_at(
        &mut self,
        player_choices: Vec<usize>,
        new_dialog: Dialog,
    ) -> Result<(), DialogError> {
        match self {
            Self::PlayerDialog(options) => {
                if player_choices.len() == 0 {
                    return Err(DialogError::MoreChoicesExist(
                        options
                            .iter()
                            .map(|(prompt, _dialog)| prompt.clone())
                            .collect(),
                    ));
                }
                let choice_index = player_choices[0];
                let new_choices = player_choices[1..].to_vec();
                if choice_index >= options.len() {
                    return Err(DialogError::OutOfBounds);
                }
                let maybe_choice = &mut options[choice_index].1;
                match maybe_choice.take() {
                    None => {
                        if new_choices.len() > 0 {
                            return Err(DialogError::EncounteredTooFewChoices);
                        }
                        *maybe_choice = Some(new_dialog);
                    }
                    Some(mut dialog) => {
                        dialog.update_leaf_at(new_choices, new_dialog)?;
                        *maybe_choice = Some(dialog);
                    }
                }
            }
            Self::NPCDialog(npc_dialog) => match &mut npc_dialog.next {
                None => {
                    if player_choices.len() > 0 {
                        return Err(DialogError::EncounteredTooFewChoices);
                    }
                    npc_dialog.next = Some(new_dialog);
                }
                Some(dialog) => {
                    dialog.update_leaf_at(player_choices, new_dialog)?;
                }
            },
        }
        return Ok(());
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

    pub fn get_speaker(&self) -> String {
        self.speaker.clone()
    }

    pub fn get_contents(&self) -> String {
        self.contents.clone()
    }

    pub fn get_next(&self) -> Option<Dialog> {
        self.next.clone()
    }
}

pub enum DialogError {
    OutOfBounds,
    MoreChoicesExist(Vec<String>),
    EncounteredTooFewChoices,
}
