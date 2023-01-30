use aws_sdk_dynamodb::Client;
use lambda_http::{
    http::Method,
    Body, Response, Request,
};
use matchit::{Match, Router};
use nanoserde::{SerJson, DeJson};

use crate::{repositories::DatabaseRepository, services::{DocumentService, DocumentReq}};

pub enum HttpRoute {
    Documents,
    Document,
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
        router
            .insert("/api/notes/documents", HttpRoute::Documents)
            .unwrap();
        router
            .insert("/api/notes/documents/:id", HttpRoute::Document)
            .unwrap();
        Self {
            router,
            document_service,
        }
    }

    pub(crate) async fn handle(&self, event: Request) -> Response<Body> {
        let (head, body) = event.into_parts();
        let match_res = self.router.at(head.uri.path());
        match match_res {
            Ok(m) => self.resolve(m, &head.method, body).await,
            Err(_) => todo!(),
        }
    }

    async fn resolve<'a>(&self, m: Match<'a, 'a, &HttpRoute>, method: &Method, body: Body) -> Response<Body> {
        let value = m.value;
        let response = match value {
            HttpRoute::Documents => match *method {
                Method::GET => {
                    let documents = self.document_service.list_all().await;
                    let body = SerJson::serialize_json(&documents);
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body(body.into())
                        .unwrap()
                },
                Method::POST => {
                    let body_string = match body {
                        Body::Text(v) => v,
                        _ => todo!(),
                    };
                    let document: DocumentReq = DeJson::deserialize_json(&body_string).unwrap();
                    self.document_service.save(&document).await.unwrap();
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body(Body::Empty)
                        .unwrap()

                },
                _ => todo!(),
            },
            HttpRoute::Document => match method {
                &Method::GET => {
                    let id = m.params.get("id").unwrap();
                    let documents = self.document_service.fetch_by_id(id).await;
                    let document = documents.first().unwrap();
                    let body = SerJson::serialize_json(document);
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body(body.into())
                        .unwrap()
                }
                _ => todo!(),
            },
        };
        response
    }
}
