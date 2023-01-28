use aws_sdk_dynamodb::Client;
use lambda_http::{Response, Body, http::{request::Parts, Method}};
use matchit::{Router, Match};
use nanoserde::SerJson;

use crate::{repositories::DatabaseRepository, services::DocumentService};

pub enum HttpRoute {
    Documents,
    Document
}

pub struct RouterDelegate {
    router: Router<HttpRoute>,
    document_service: DocumentService,
}

impl RouterDelegate {
    pub(crate) fn new(client: Client) -> Self {
        let database = DatabaseRepository::from_client(client);
        let document_service = DocumentService::new(database);
        let mut router = Router::new();
        router.insert("/api/notes/documents", HttpRoute::Documents).unwrap();
        router.insert("/api/notes/documents/:id", HttpRoute::Document).unwrap();
        Self { router, document_service }
    }

    pub(crate) async fn handle(&self, parts: &Parts) -> Response<Body> {
        dbg!(parts.uri.path());
        let match_res = self.router.at(parts.uri.path());
        match match_res {
            Ok(m) => self.resolve(m, &parts.method).await,
            Err(_) => todo!(),
        }
    }

    async fn resolve<'a>(&self, m: Match<'a, 'a, &HttpRoute>, method: &Method) -> Response<Body> {
        let value = m.value;
        let response = match value {
            HttpRoute::Documents => {
                match method {
                    &Method::GET => {
                        let documents = self.document_service.list_all().await;
                        let body = SerJson::serialize_json(&documents.to_vec());
                        Response::builder()
                            .status(200)
                            .header("content-type", "application/json")
                            .body(body.into())
                            .unwrap() 
                    }
                    _ => todo!()
                }
            },
            HttpRoute::Document => {
                match method {
                    &Method::GET => {
                        let id = m.params.get("id").unwrap();
                        let documents = self.document_service.fetch_by_id(id).await;
                        let body = SerJson::serialize_json(&documents);
                        Response::builder()
                            .status(200)
                            .header("content-type", "application/json")
                            .body(body.into())
                            .unwrap() 
                    }
                    _ => todo!()
                }
            },
        };
        response
    }
}
