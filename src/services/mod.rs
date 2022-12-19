use std::collections::HashMap;

use aws_sdk_dynamodb::model::AttributeValue;

use multimap::MultiMap;
use nanoserde::SerJson;

use crate::repositories::DatabaseRepository;

const PK: &str = "PK";
const SK: &str = "SK";
const TITLE: &str = "title";
const CREATED: &str = "created";
const PARENT: &str = "parent";
const UPDATED_BY: &str = "updatedBy";
const DESCRIPTION: &str = "description";

#[derive(Clone, SerJson)]
pub struct Document {
    pub pk: String,
    pub sk: String,
    title: String,
    description: String,
    created: u32,
    #[nserde(rename = "updatedBy")]
    updated_by: String,
    groups: Vec<Group>,
}

impl Document {
    fn new(
        pk: String,
        sk: String,
        title: String,
        description: String,
        created: u32,
        updated_by: String,
    ) -> Self {
        let groups = Vec::default();
        Document {
            pk,
            sk,
            title,
            description,
            created,
            updated_by,
            groups,
        }
    }

    fn add_group(&mut self, group: Group) {
        self.groups.push(group);
    }
}

impl From<&HashMap<String, AttributeValue>> for Document {
    fn from(group_entity: &HashMap<String, AttributeValue>) -> Self {
        let pk = group_entity[PK].as_s().unwrap().to_owned();
        let sk = group_entity[SK].as_s().unwrap().to_owned();
        let title = group_entity[TITLE].as_s().unwrap().to_owned();
        let updated_by = group_entity[UPDATED_BY].as_s().unwrap().to_owned();
        let description = group_entity[DESCRIPTION].as_s().unwrap().to_owned();
        let created: u32 = group_entity[CREATED]
            .as_n()
            .unwrap()
            .clone()
            .parse()
            .unwrap();
        Document::new(pk, sk, title, description, created, updated_by)
    }
}

#[derive(SerJson, Clone)]
pub struct Group {
    pub sk: String,
    title: String,
    created: u32,
    notes: Vec<Note>,
}

impl Group {
    fn new(sk: impl Into<String>, title: String, created: u32) -> Group {
        let sk = sk.into();
        let notes = Vec::default();
        Group {
            sk,
            title,
            created,
            notes,
        }
    }

    fn set_notes(&mut self, notes: Vec<Note>) {
        self.notes = notes;
    }
}

impl From<&HashMap<String, AttributeValue>> for Group {
    fn from(group_entity: &HashMap<String, AttributeValue>) -> Self {
        let sk = group_entity[SK].as_s().unwrap();
        let title = group_entity[TITLE].as_s().unwrap().clone();
        let created: u32 = group_entity[CREATED]
            .as_n()
            .unwrap()
            .clone()
            .parse()
            .unwrap();
        Group::new(sk, title, created)
    }
}

#[derive(SerJson, Clone)]
pub struct Note {
    title: String,
    created: u32,
}

impl From<&HashMap<String, AttributeValue>> for Note {
    fn from(note_entity: &HashMap<String, AttributeValue>) -> Self {
        let title = note_entity[TITLE].as_s().unwrap().to_owned();
        let created: u32 = note_entity[CREATED].as_n().unwrap().parse().unwrap();
        Note { title, created }
    }
}

pub struct DocumentService {
    database_repository: DatabaseRepository,
}

impl DocumentService {
    pub fn new(database_repository: DatabaseRepository) -> Self {
        Self {
            database_repository,
        }
    }

    pub async fn list_all(&self) -> Vec<Document> {
        let items = self.database_repository.list_all().await;
        let mut lookup = MultiMap::new();
        let mut documents = Vec::new();
        for item in items.iter() {
            if item.contains_key(PARENT) {
                let value = item[PARENT].as_s().unwrap();
                lookup.insert(value, item);
            } else {
                let document = Document::from(item);
                documents.push(document);
            }
        }

        for document in documents.iter_mut() {
            let groups = lookup.get_vec(&document.sk).unwrap();

            for group in groups.iter() {
                let mut group = Group::from(*group);
                if let Some(notes) = lookup.get_vec(&group.sk) {
                    let notes: Vec<Note> = notes.iter().map(|note| Note::from(*note)).collect();
                    group.set_notes(notes);
                }
                document.add_group(group);
            }
        }
        documents
    }

    pub(crate) async fn fetch_by_id(&self, id: &str) -> Document {
        let items = self.database_repository.fetch_by_id(id).await;

        let filtered_documents: Vec<&HashMap<String, AttributeValue>> = items
            .iter()
            .filter(|item| item["PK"].as_s().unwrap() == "document")
            .collect();

        let groups: Vec<&HashMap<String, AttributeValue>> = items
            .iter()
            .filter(|item| item["PK"].as_s().unwrap() == "group")
            .collect();

        let notes: Vec<&HashMap<String, AttributeValue>> = items
            .iter()
            .filter(|item| item["PK"].as_s().unwrap() == "note")
            .collect();

        let documents: Vec<Document> = filtered_documents
            .iter()
            .map(|item| {
                let sk = item["SK"].as_s().unwrap();
                let groups: Vec<Group> = groups
                    .iter()
                    .filter(|group| {
                        let parent = group["parent"].as_s().unwrap();
                        parent == sk
                    })
                    .map(|group| {
                        let group_sk = group[SK].as_s().unwrap();
                        let notes = notes
                            .iter()
                            .filter(|note| {
                                let parent = note["parent"].as_s().unwrap();
                                parent == group_sk
                            })
                            .map(|note| {
                                let title = note["title"].as_s().unwrap().clone();
                                let created: u32 =
                                    note["created"].as_n().unwrap().clone().parse().unwrap();
                                Note { title, created }
                            })
                            .collect();
                        let mut group = Group::from(*group);
                        group.set_notes(notes);
                        group
                    })
                    .collect();
                Document::from(*item)
            })
            .collect();
        let doc = documents.first().unwrap();
        doc.clone()
    }
}
