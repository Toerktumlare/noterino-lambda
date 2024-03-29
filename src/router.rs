use lambda_http::{http::Method, Body, Request, Response};
use matchit::{Match, Router};
use nanoserde::{DeJson, SerJson};

use crate::controllers::document_controller::DocumentController;
use crate::controllers::{DocumentReq, NoteReq};
use crate::repositories::document_repository::DatabaseRepository;
use crate::services::notes_service::NotesService;

pub enum HttpRoute {
    Documents,
    Document,
    Group,
}

pub struct RouterDelegate<'a> {
    router: Router<HttpRoute>,
    document_controller: DocumentController<'a>,
    notes_service: NotesService<'a>,
}

impl<'a> RouterDelegate<'a> {
    pub(crate) fn new(database: &'a DatabaseRepository) -> Self {
        let document_service = DocumentController::new(&database);
        let notes_service = NotesService::new(&database);
        let mut router = Router::new();
        router
            .insert("/api/notes/documents", HttpRoute::Documents)
            .unwrap();
        router
            .insert("/api/notes/documents/:id", HttpRoute::Document)
            .unwrap();
        router
            .insert("/api/notes/documents/:id/groups/:groupId", HttpRoute::Group)
            .unwrap();
        Self {
            router,
            document_controller: document_service,
            notes_service,
        }
    }

    pub(crate) async fn handle(&self, event: Request) -> Response<Body> {
        let (head, body) = event.into_parts();
        let match_res = self.router.at(head.uri.path());
        dbg!(body.clone());
        match match_res {
            Ok(m) => self.resolve(m, &head.method, body).await,
            Err(_) => Response::builder().status(404).body(Body::Empty).unwrap(),
        }
    }

    async fn resolve(
        &self,
        m: Match<'a, 'a, &HttpRoute>,
        method: &Method,
        body: Body,
    ) -> Response<Body> {
        let value = m.value;
        let response = match value {
            HttpRoute::Documents => match *method {
                Method::GET => {
                    let documents = self.document_controller.list_all().await;
                    let body = SerJson::serialize_json(&documents);
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body(body.into())
                        .unwrap()
                }
                Method::POST => match body {
                    Body::Text(body) => {
                        let document: DocumentReq = DeJson::deserialize_json(&body).unwrap();
                        self.document_controller.save(&document).await.unwrap();
                        return Response::builder().status(200).body(Body::Empty).unwrap();
                    }
                    _ => Response::builder().status(400).body(Body::Empty).unwrap(),
                },
                _ => Response::builder().status(405).body(Body::Empty).unwrap(),
            },
            HttpRoute::Document => match method {
                &Method::GET => {
                    let id = m.params.get("id").unwrap();
                    let documents = self.document_controller.fetch_by_id(id).await;
                    let document = documents.first().unwrap();
                    let body = SerJson::serialize_json(document);
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body(body.into())
                        .unwrap()
                }
                _ => Response::builder().status(405).body(Body::Empty).unwrap(),
            },
            HttpRoute::Group => match *method {
                Method::POST => match body {
                    Body::Text(body) => {
                        let doc_id: i32 = m.params.get("id").unwrap().parse().unwrap();
                        let group_id: i32 = m.params.get("groupId").unwrap().parse().unwrap();
                        let note_req: NoteReq = DeJson::deserialize_json(&body).unwrap();
                        self.notes_service
                            .save(doc_id, group_id, &note_req)
                            .await
                            .unwrap()
                    }
                    _ => Response::builder().status(400).body(Body::Empty).unwrap(),
                },
                _ => Response::builder().status(405).body(Body::Empty).unwrap(),
            },
        };
        response
    }
}
