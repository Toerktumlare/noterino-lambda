use nanoserde::{DeJson, SerJson};

pub mod document_controller;

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
    pub notes: Vec<NoteReq>,
}

#[derive(Clone, SerJson, DeJson)]
pub struct NoteReq {
    pub title: String,

    #[nserde(default)]
    pub description: String,

    #[nserde(default)]
    pub created: i64,

    #[nserde(default)]
    pub created_by: String,

    #[nserde(default)]
    pub updated_by: String,
}
