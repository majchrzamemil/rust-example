use crate::schema::{events, user_events, users};
use chrono::NaiveDateTime;
use juniper::{GraphQLInputObject, GraphQLObject};
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Debug, Serialize, Deserialize, Queryable, Associations)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct DbUser<'a> {
    pub user_id: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub jwt: &'a str,
}

#[derive(Insertable, Debug)]
#[table_name = "events"]
pub struct DbEvent<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub longitude: &'a f64,
    pub latitude: &'a f64,
    pub date_time: &'a NaiveDateTime,
}

///Struct used for creation of new Event.
#[derive(Debug, GraphQLInputObject)]
pub struct NewEvent {
    pub name: String,
    pub description: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    /// Date and time represented as float (unix timestamp),
    /// for more information look into [Unix time](https://en.wikipedia.org/wiki/Unix_time)
    pub date_time: NaiveDateTime,
}

///Struct representing Event model.
#[derive(Identifiable, Debug, Queryable, GraphQLObject, Associations)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub date_time: NaiveDateTime,
}

///User-Event connection.
#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Event)]
#[belongs_to(User)]
pub struct UserEvent {
    pub id: i32,
    pub event_id: String,
    pub user_id: String,
    pub owner: bool,
}

///Used for creating User-Event connection
#[derive(Insertable, Debug)]
#[table_name = "user_events"]
pub struct NewUserEvent<'a> {
    pub event_id: &'a str,
    pub user_id: &'a str,
    pub owner: bool,
}
