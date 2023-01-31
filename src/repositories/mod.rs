use std::collections::HashMap;

use aws_sdk_dynamodb::{model::AttributeValue, Client, Error};
use chrono::Utc;

use crate::{
    controllers::{DocumentReq, GroupReq},
    services::{CREATED, DESCRIPTION, LAST_UPDATED, PARENT, PK, SK, TITLE},
};

pub struct DatabaseRepository {
    client: Client,
    table_name: String,
}

impl DatabaseRepository {
    pub fn from_client(client: Client) -> Self {
        let table_name =
            std::env::var("TABLE_NAME").expect("A TABLE_NAME must be set in this app's Lambda");
        Self { client, table_name }
    }

    pub(crate) async fn list_all(&self) -> Vec<HashMap<String, AttributeValue>> {
        let req = self
            .client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .unwrap();

        let items = req.items().unwrap();
        items.to_vec()
    }

    pub(crate) async fn fetch_by_id(&self, id: &str) -> Vec<HashMap<String, AttributeValue>> {
        let mut sk = String::from("DOCUMENT#");
        let mut notes_sk = String::from("GROUP#");
        sk.push_str(id);
        notes_sk.push_str(id);
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), AttributeValue::S("document".to_string()));
        keys.insert("SK".to_string(), AttributeValue::S(sk.clone()));
        let response = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(keys))
            .consistent_read(true)
            .send()
            .await
            .unwrap();

        let groups = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pk = :pk")
            .filter_expression("contains(#parent, :parentId)")
            .expression_attribute_names("#pk", "PK")
            .expression_attribute_values(":pk", AttributeValue::S("group".to_string()))
            .expression_attribute_values(":parentId", AttributeValue::S(sk))
            .expression_attribute_names("#parent", "parent")
            .send()
            .await
            .unwrap();

        let notes = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pk = :pk")
            .filter_expression("contains(#parent, :parentId)")
            .expression_attribute_names("#pk", "PK")
            .expression_attribute_values(":pk", AttributeValue::S("note".to_string()))
            .expression_attribute_values(":parentId", AttributeValue::S(notes_sk))
            .expression_attribute_names("#parent", "parent")
            .send()
            .await
            .unwrap();

        let response = response.item().unwrap();

        let mut document = groups.items().unwrap().to_vec();
        let mut notes = notes.items().unwrap().to_vec();

        document.append(&mut notes);
        document.push(response.to_owned());

        document
    }

    pub(crate) async fn save(&self, document: &DocumentReq) -> Result<(), Error> {
        let timestamp = Utc::now().timestamp();
        let mut sk = String::from("DOCUMENT#");
        sk.push_str(&timestamp.to_string());

        let pk = AttributeValue::S("document".to_string());
        let sk = AttributeValue::S(sk);
        let title = AttributeValue::S(document.title.clone());
        let created = AttributeValue::N(timestamp.to_string());
        let last_updated = AttributeValue::S(timestamp.to_string());
        let description = AttributeValue::S(document.description.clone());

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item(PK, pk)
            .item(SK, sk)
            .item(TITLE, title)
            .item(CREATED, created)
            .item(LAST_UPDATED, last_updated)
            .item(DESCRIPTION, description)
            .send()
            .await
            .unwrap();

        if !document.groups.is_empty() {
            for mut group in document.groups.iter().cloned() {
                group.created = timestamp;
                self.save_group(&group).await.unwrap();
            }
        }

        Ok(())
    }

    pub(crate) async fn save_group(&self, group: &GroupReq) -> Result<(), Error> {
        let mut sk = String::from("GROUP#");
        sk.push_str(&group.created.to_string());

        let mut parent = String::from("DOCUMENT#");
        parent.push_str(&group.created.to_string());

        let pk = AttributeValue::S("group".to_string());
        let sk = AttributeValue::S(sk);
        let title = AttributeValue::S(group.title.clone());
        let created = AttributeValue::N(group.created.to_string());
        let last_updated = AttributeValue::N(group.created.to_string());
        let description = AttributeValue::S(group.description.clone());
        let parent = AttributeValue::S(parent);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item(PK, pk)
            .item(SK, sk)
            .item(TITLE, title)
            .item(CREATED, created)
            .item(LAST_UPDATED, last_updated)
            .item(DESCRIPTION, description)
            .item(PARENT, parent)
            .send()
            .await
            .unwrap();
        Ok(())
    }
}
