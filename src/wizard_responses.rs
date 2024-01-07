use poem_openapi::{
    types::{ParseFromJSON, ToJSON},
    Object,
};

#[derive(Object)]
pub struct ResponseObject<T: ParseFromJSON + ToJSON + Send + Sync> {
    code: i32,
    msg: String,
    data: Option<T>,
}

impl<T: ParseFromJSON + ToJSON + Send + Sync> ResponseObject<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 200,
            msg: "OK".to_string(),
            data: Some(data),
        }
    }

    pub fn message(msg: String) -> Self {
        Self {
            code: 200,
            msg,
            data: None,
        }
    }

    pub fn bad_request(message: Option<String>) -> Self {
        Self {
            code: 400,
            msg: match message {
                Some(msg) => msg,
                None => "Bad Request".to_string(),
            },
            data: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            code: 404,
            msg: "Not Found".to_string(),
            data: None,
        }
    }

    pub fn internal_server_error(msg: String) -> Self {
        Self {
            code: 500,
            msg,
            data: None,
        }
    }
}
