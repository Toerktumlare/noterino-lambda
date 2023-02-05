use chrono::Utc;
use lambda_http::Error;
use nanoserde::{DeJson, SerJson};

use crate::{controllers::NoteReq, repositories::DatabaseRepository};

use super::SK;

pub struct NotesService<'a> {
    database_repository: &'a DatabaseRepository,
}

#[derive(SerJson, DeJson, Clone)]
pub struct Note {
    pub title: String,
    pub description: String,
    pub created: i64,
    pub created_by: String,
    pub updated_by: String,
    pub parent: String,
}

impl From<&NoteReq> for Note {
    fn from(value: &NoteReq) -> Self {
        let timestamp = Utc::now().timestamp();
        Note {
            title: value.title.clone(),
            description: value.description.clone(),
            created: timestamp,
            created_by: value.created_by.clone(),
            updated_by: value.updated_by.clone(),
            parent: String::from(""),
        }
    }
}

impl<'a> NotesService<'a> {
    pub fn new(database_repository: &'a DatabaseRepository) -> Self {
        Self {
            database_repository,
        }
    }

    pub(crate) async fn save(
        &self,
        doc_id: i32,
        group_id: i32,
        note_req: &NoteReq,
    ) -> Result<(), Error> {
        let documents = self
            .database_repository
            .fetch_document_by_id(&doc_id.to_string())
            .await;

        if documents.is_empty() {
            todo!("Throw a cool error here");
        }
        let groups = self.database_repository.fetch_group_by_id(group_id).await;
        if groups.is_empty() {
            todo!("Throw a cool error here");
        }

        // Check that group and document actually belong together

        let mut note = Note::from(note_req);
        note.parent = groups
            .first()
            .unwrap()
            .get(SK)
            .unwrap()
            .as_s()
            .unwrap()
            .to_string();
        self.database_repository.save_note(&note).await.unwrap();
        Ok(())
    }
}
