use lambda_http::Error;

use crate::{controllers::DocumentReq, repositories::DatabaseRepository};

use super::Documents;

pub struct DocumentService {
    database_repository: DatabaseRepository,
}

impl DocumentService {
    pub fn new(database_repository: DatabaseRepository) -> Self {
        Self {
            database_repository,
        }
    }

    pub(crate) async fn list_all(&self) -> Documents {
        let items = self.database_repository.list_all().await;
        Documents::from(items)
    }

    pub(crate) async fn fetch_by_id(&self, id: &str) -> Documents {
        let item = self.database_repository.fetch_by_id(id).await;
        Documents::from(item)
    }

    pub(crate) async fn save(&self, document: &DocumentReq) -> Result<(), Error> {
        self.database_repository.save(document).await.unwrap();
        Ok(())
    }
}
