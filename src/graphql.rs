use crate::models::NewUserEvent;
use crate::models::User;
use crate::models::UserEvent;
use crate::models::{DbEvent, Event, NewEvent};
use crate::schema::events::dsl::*;
use crate::schema::events::id;
use crate::schema::user_events::dsl::user_events;
use crate::schema::user_events::event_id;
use crate::schema::users::dsl::*;
use crate::Pool;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl};
use log::{error, info};
use uuid::Uuid;

///Definition of QueryRoot for GraphQL
pub struct QueryRoot;

///Definition of MutationRoot for GraphQL
pub struct MutationRoot;

pub type Schema = juniper::RootNode<'static, QueryRoot, MutationRoot>;

/// GraphQLContext contains pool of DB connections
pub struct GraphQLContext {
    pub pool: Pool,
    pub user_id: Option<String>,
}
impl juniper::Context for GraphQLContext {}

#[juniper::object(Context = GraphQLContext)]
impl QueryRoot {
    ///Used for GET events
    ///
    ///Returns list of events
    pub fn events(context: &GraphQLContext) -> juniper::FieldResult<Vec<Event>> {
        let conn = context.pool.get().unwrap();
        //Error handling after Menelu will finish his task.
        let items = events.load::<Event>(&conn)?;
        match &context.user_id {
            Some(u_id) => {
                info!("User uuid: {} accessed GraphQL", u_id);
            }
            None => {}
        }
        Ok(items)
    }

    /// Returns list of events connected to logged in User
    pub fn user_events(context: &GraphQLContext) -> juniper::FieldResult<Vec<Event>> {
        let conn = context.pool.get().unwrap();
        let mut events_lits: Vec<Event>;
        match &context.user_id {
            Some(u_id) => {
                info!("User uuid: {} accessed user_events", &u_id);
                let user = users.find(u_id).get_result::<User>(&conn)?;
                let events_ids = UserEvent::belonging_to(&user).select(event_id);
                events_lits = events.filter(id.eq(any(events_ids))).load::<Event>(&conn)?;
            }
            None => {
                error!("Not possible due to design, REPORT BUG");
                //Should not happen, just for compiler
                events_lits = Vec::new();
            }
        }
        Ok(events_lits)
    }
}

#[juniper::object(Context = GraphQLContext)]
impl MutationRoot {
    ///Used for POST Event
    ///
    ///Return created Event
    pub fn create_event(context: &GraphQLContext, input: NewEvent) -> juniper::FieldResult<Event> {
        //Also incorporate error handling.
        let conn = context.pool.get().unwrap();
        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let new_event = DbEvent {
            id: &uuid,
            name: &input.name,
            description: &input.description.unwrap(),
            longitude: &input.longitude,
            latitude: &input.latitude,
            date_time: &input.date_time,
        };
        let res = diesel::dsl::insert_into(events)
            .values(&new_event)
            .get_result(&conn)?;
        match &context.user_id {
            Some(other_user_id) => {
                info!("User uuid: {} accessed create_event", other_user_id);
                let new_user_event = NewUserEvent {
                    event_id: &uuid,
                    user_id: &other_user_id,
                    owner: true,
                };
                let res2 = diesel::dsl::insert_into(user_events)
                    .values(&new_user_event)
                    .execute(&conn)?;
            }
            None => {
                error!("Not possible due to design, REPORT BUG");
            }
        }
        Ok(res)
    }
}
