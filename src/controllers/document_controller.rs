use lambda_http::{http::Result, Body, Response};

use crate::{repositories::document_repository::DatabaseRepository, services::Documents};

use super::DocumentReq;

pub struct DocumentController<'a> {
    database_repository: &'a DatabaseRepository,
}

impl<'a> DocumentController<'a> {
    pub fn new(database_repository: &'a DatabaseRepository) -> Self {
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

    pub(crate) async fn save(&self, document: &DocumentReq) -> Result<Response<Body>> {
        self.database_repository.save(document).await.unwrap();
        let response = Response::builder().status(200).body(Body::Empty).unwrap();
        Ok(response)
    }
}
