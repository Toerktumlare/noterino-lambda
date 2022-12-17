use std::collections::HashMap;

use aws_sdk_dynamodb::{
    model::{AttributeValue, Get, ItemResponse, TransactGetItem},
    Client,
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
}
