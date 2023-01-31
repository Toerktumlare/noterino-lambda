use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use aws_sdk_dynamodb::model::AttributeValue;

use multimap::MultiMap;
use nanoserde::{DeJson, SerJson};

pub const PK: &str = "PK";
pub const SK: &str = "SK";
pub const TITLE: &str = "title";
pub const CREATED: &str = "created";
pub const PARENT: &str = "parent";
pub const UPDATED_BY: &str = "updatedBy";
pub const LAST_UPDATED: &str = "lastUpdated";
pub const DESCRIPTION: &str = "description";

pub mod documentService;

#[derive(Clone, SerJson, DeJson)]
pub struct Document {
    pub pk: String,
    pub sk: String,
    pub title: String,
    description: String,
    created: i64,
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
        created: i64,
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

impl From<HashMap<String, AttributeValue>> for Document {
    fn from(group_entity: HashMap<String, AttributeValue>) -> Self {
        let pk = group_entity[PK].as_s().unwrap().to_owned();
        let sk = group_entity[SK].as_s().unwrap().to_owned();
        let title = group_entity[TITLE].as_s().unwrap().to_owned();
        let updated_by = group_entity[UPDATED_BY].as_s().unwrap().to_owned();
        let description = group_entity[DESCRIPTION].as_s().unwrap().to_owned();
        let created: i64 = group_entity[CREATED]
            .as_n()
            .unwrap()
            .clone()
            .parse()
            .unwrap();
        Document::new(pk, sk, title, description, created, updated_by)
    }
}

#[derive(SerJson, DeJson, Clone)]
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

#[derive(SerJson, DeJson, Clone)]
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

#[derive(SerJson, Clone)]
pub struct Documents(Vec<Document>);

impl Documents {
    fn new() -> Self {
        Documents(Vec::new())
    }
}

impl Deref for Documents {
    type Target = Vec<Document>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Documents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<HashMap<String, AttributeValue>>> for Documents {
    fn from(items: Vec<HashMap<String, AttributeValue>>) -> Self {
        let mut lookup = MultiMap::new();
        let mut documents = Documents::new();
        for item in items {
            if item.contains_key(PARENT) {
                let value = item[PARENT].as_s().unwrap();
                lookup.insert(value.clone(), item);
            } else {
                let document = Document::from(item);
                documents.push(document);
            }
        }

        for document in documents.iter_mut() {
            let groups = lookup.get_vec(&document.sk).unwrap();

            for group in groups {
                let mut group = Group::from(group);
                if let Some(notes) = lookup.get_vec(&group.sk) {
                    let notes: Vec<Note> = notes.iter().map(Note::from).collect();
                    group.set_notes(notes);
                }
                document.add_group(group);
            }
        }
        documents
    }
}
