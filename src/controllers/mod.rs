use nanoserde::{DeJson, SerJson};

use crate::services::Note;

#[derive(Clone, SerJson, DeJson)]
pub struct DocumentReq {
    pub title: String,

    #[nserde(default)]
    pub updated_by: String,

    #[nserde(default)]
    pub description: String,

    #[nserde(default)]
    pub groups: Vec<GroupReq>,
}

#[derive(Clone, SerJson, DeJson)]
pub struct GroupReq {
    pub title: String,

    #[nserde(default)]
    pub description: String,

    #[nserde(default)]
    pub created: i64,

    #[nserde(default)]
    pub created_by: String,

    #[nserde(default)]
    pub updated_by: String,

    #[nserde(default)]
    pub notes: Vec<Note>,
}
