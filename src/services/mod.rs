use std::collections::HashMap;

use aws_sdk_dynamodb::model::{AttributeValue, ItemResponse};
use serde::Serialize;

use crate::repositories::DatabaseRepository;

#[derive(Serialize)]
pub struct Document {
    title: String,
    description: String,
    created: u32,
    #[serde(rename(serialize = "updatedBy"))]
    updated_by: String,
    groups: Vec<Group>,
}

#[derive(Serialize)]
pub struct Group {
    title: String,
    created: u32,
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

        let filtered_documents: Vec<&HashMap<String, AttributeValue>> = items
            .iter()
            .filter(|item| item["PK"].as_s().unwrap() == "document")
            .collect();

        let groups: Vec<&HashMap<String, AttributeValue>> = items
            .iter()
            .filter(|item| item["PK"].as_s().unwrap() == "group")
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
                        let title = group["title"].as_s().unwrap().clone();
                        let created: u32 =
                            group["created"].as_n().unwrap().clone().parse().unwrap();
                        Group { title, created }
                    })
                    .collect();
                let title = DocumentService::get_string(&item["title"]);
                let description = DocumentService::get_string(&item["description"]);
                let updated_by = DocumentService::get_string(&item["updatedBy"]);
                let created = DocumentService::get_number(&item["created"]);
                Document {
                    title,
                    description,
                    updated_by,
                    created,
                    groups,
                }
            })
            .collect();

        documents
    }

    fn get_string(value: &AttributeValue) -> String {
        value.as_s().unwrap().to_owned()
    }

    fn get_number(v: &AttributeValue) -> u32 {
        let number: u32 = v.as_n().unwrap().parse().unwrap();
        number
    }
}
