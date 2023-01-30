use std::collections::HashMap;

use aws_sdk_dynamodb::{model::AttributeValue, Client, Error};
use chrono::Utc;

use crate::services::{PK, SK, TITLE, DocumentReq, UPDATED_BY, CREATED, DESCRIPTION};

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
        let timestamp = Utc::now().timestamp().to_string();
        let mut sk = String::from("DOCUMENT#");
        sk.push_str(&timestamp);

        let pk = AttributeValue::S("document".to_string());
        let sk = AttributeValue::S(sk);
        let title = AttributeValue::S(document.title.clone());
        let updated_by = AttributeValue::S(timestamp.clone());
        let created_by = AttributeValue::S(timestamp.clone());
        let description = AttributeValue::S(String::from(""));

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item(PK, pk)
            .item(SK, sk)
            .item(TITLE, title)
            .item(UPDATED_BY, updated_by)
            .item(CREATED, created_by)
            .item(DESCRIPTION, description)
            .send()
            .await
            .unwrap();
        Ok(())
    }
}
