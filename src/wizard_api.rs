use crate::wizard_responses::ResponseObject;
use crate::wizard_service::{self, Wizard, WizardError};
use poem::{web::Data, Error};
use poem_openapi::types::multipart::Upload;
use poem_openapi::Multipart;
use poem_openapi::{
    param::Path,
    payload::Json,
    types::{ParseFromJSON, ToJSON},
    ApiResponse, Object, OpenApi,
};
use sqlx::PgPool;
use tracing::info;

#[derive(ApiResponse)]
#[oai(bad_request_handler = "wizard_bad_request_handler")]
enum WizardResponse<T: ParseFromJSON + ToJSON + Send + Sync> {
    #[oai(status = 200)]
    Ok(Json<ResponseObject<T>>),
    #[oai(status = 400)]
    BadRequest(Json<ResponseObject<T>>),
}

#[derive(ApiResponse)]
enum WizardResponseError<T: ParseFromJSON + ToJSON + Send + Sync> {
    #[oai(status = 404)]
    NotFound(Json<ResponseObject<T>>),
    #[oai(status = 500)]
    InternalServerError(Json<ResponseObject<T>>),
}

fn wizard_bad_request_handler<T: ParseFromJSON + ToJSON + Send + Sync>(
    _err: Error,
) -> WizardResponse<T> {
    WizardResponse::BadRequest(Json(ResponseObject::bad_request(None)))
}

#[derive(Debug, Object)]
pub struct CreateWizardRequest {
    pub name: String,
    pub title: String,
    pub age: i32,
}

#[derive(Debug, Multipart)]
pub struct UploadPayload {
    pub file: Upload,
}

pub struct WizardApi;

#[OpenApi]
impl WizardApi {
    /// Add a new wizard
    #[oai(path = "/wizards", method = "post")]
    async fn add(
        &self,
        conn: Data<&PgPool>,
        request: Json<CreateWizardRequest>,
    ) -> Result<WizardResponse<Wizard>, WizardResponseError<Wizard>> {
        let wizard = wizard_service::create(request.0, conn.0).await;

        match wizard {
            Ok(wizard) => Ok(WizardResponse::Ok(Json(ResponseObject::ok(wizard)))),
            Err(err) => {
                tracing::error!("POST /wizards ERROR: {}", &err.to_string());
                Err(WizardResponseError::InternalServerError(Json(
                    ResponseObject::internal_server_error(err.to_string()),
                )))
            }
        }
    }

    /// Get all wizards
    #[oai(path = "/wizards", method = "get")]
    async fn get_all(
        &self,
        conn: Data<&PgPool>,
    ) -> Result<WizardResponse<Vec<Wizard>>, WizardResponseError<Vec<Wizard>>> {
        let wizards = wizard_service::get_all(conn.0).await;

        match wizards {
            Ok(wizards) => Ok(WizardResponse::Ok(Json(ResponseObject::ok(wizards)))),
            Err(err) => {
                tracing::error!("GET /wizards error: {}", err.to_string());
                Err(WizardResponseError::InternalServerError(Json(
                    ResponseObject::internal_server_error(err.to_string()),
                )))
            }
        }
    }

    /// Get a wizard by id
    #[oai(path = "/wizards/:id", method = "get")]
    async fn get_by_id(
        &self,
        conn: Data<&PgPool>,
        id: Path<i32>,
    ) -> Result<WizardResponse<Wizard>, WizardResponseError<Wizard>> {
        let wizard = wizard_service::get_by_id(id.0, conn.0).await;

        match wizard {
            Ok(wizard) => Ok(WizardResponse::Ok(Json(ResponseObject::ok(wizard)))),
            Err(err) => match err {
                WizardError::NotFoundError => Err(WizardResponseError::NotFound(Json(
                    ResponseObject::not_found(),
                ))),
                WizardError::ExtError(err) => {
                    tracing::error!("GET /wizards/{} error: {}", id.0, err.to_string());
                    Err(WizardResponseError::InternalServerError(Json(
                        ResponseObject::internal_server_error(err.to_string()),
                    )))
                }
            },
        }
    }

    /// Update a wizard by id
    #[oai(path = "/wizards/:id", method = "put")]
    async fn update_wizard(
        &self,
        conn: Data<&PgPool>,
        id: Path<i32>,
        request: Json<CreateWizardRequest>,
    ) -> Result<WizardResponse<Wizard>, WizardResponseError<Wizard>> {
        let wizard = wizard_service::update_by_id(id.0, request.0, conn.0).await;

        match wizard {
            Ok(wizard) => Ok(WizardResponse::Ok(Json(ResponseObject::ok(wizard)))),
            Err(err) => match err {
                WizardError::NotFoundError => Err(WizardResponseError::NotFound(Json(
                    ResponseObject::not_found(),
                ))),
                WizardError::ExtError(err) => {
                    tracing::error!("PUT /wizards/{} error: {}", id.0, err.to_string());
                    Err(WizardResponseError::InternalServerError(Json(
                        ResponseObject::internal_server_error(err.to_string()),
                    )))
                }
            },
        }
    }

    /// Upload wizard image
    #[oai(path = "/wizards/:id/image", method = "post")]
    async fn upload_image(
        &self,
        conn: Data<&PgPool>,
        id: Path<i32>,
        request: UploadPayload,
    ) -> Result<WizardResponse<String>, WizardResponseError<String>> {
        let result = wizard_service::save_image(id.0, conn.0, request.file).await;

        match result {
            Ok(msg) => Ok(WizardResponse::Ok(Json(ResponseObject::ok(msg)))),
            Err(err) => match err {
                WizardError::NotFoundError => Err(WizardResponseError::NotFound(Json(
                    ResponseObject::not_found(),
                ))),
                WizardError::ExtError(err) => {
                    tracing::error!("POST /wizards/{}/image error: {}", id.0, err.to_string());
                    Err(WizardResponseError::InternalServerError(Json(
                        ResponseObject::internal_server_error(err.to_string()),
                    )))
                }
            },
        }
    }

    /// Delete a wizard by id
    #[oai(path = "/wizards/:id", method = "delete")]
    async fn delete(
        &self,
        conn: Data<&PgPool>,
        id: Path<i32>,
    ) -> Result<WizardResponse<Wizard>, WizardResponseError<Wizard>> {
        let result = wizard_service::delete_by_id(id.0, conn.0).await;

        match result {
            Ok(()) => Ok(WizardResponse::Ok(Json(ResponseObject::message(
                "Delete successful".to_string(),
            )))),
            Err(err) => match err {
                WizardError::NotFoundError => Err(WizardResponseError::NotFound(Json(
                    ResponseObject::not_found(),
                ))),
                WizardError::ExtError(err) => {
                    tracing::error!("DELETE /wizards/{} error: {}", id.0, err.to_string());
                    Err(WizardResponseError::InternalServerError(Json(
                        ResponseObject::internal_server_error(err.to_string()),
                    )))
                }
            },
        }
    }
}
