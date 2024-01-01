use poem_openapi::Object;
use serde::{Serialize, Deserialize};

#[derive(Object, Serialize, Deserialize)]
pub struct Wizard {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub age: i32,
}

#[derive(Object, Serialize, Deserialize)]
pub struct CreateWizard {
    pub name: String,
    pub title: String,
    pub age: i32,
}