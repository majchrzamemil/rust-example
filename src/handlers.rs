use crate::auth::{generate_token, verify_token};
use crate::error::ServiceError;
use crate::graphql::GraphQLContext;
use crate::graphql::Schema;
use crate::handlers::ServiceError::BadRequest;
use crate::models::{DbUser, NewUser, User};
use crate::schema::users::dsl::*;
use crate::utils::{hash, verify};
use crate::Pool;
use actix_web::FromRequest;
use actix_web::{dev::ServiceRequest, web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::expression_methods::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use futures_util::future::{err, ok, Ready};
use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

///Validates token.
///TODO: Move to auth
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let auth = credentials.token();
    match verify_token(&auth.to_string()) {
        Ok(uuid) => {
            info!("User uuid: {} accessed API", uuid);
            Ok(req)
        }
        Err(e) => Err(e.into()),
    }
}

/// Called for route /register HTTP request type: POST, used for registering new user.
///
/// Provided input JSON should be serialized to model [NewUser](../models/struct.NewUser.html)
///
/// Password len >= 8, email must contain '@', returns Created for success.
pub async fn register_user(
    db: web::Data<Pool>,
    item: web::Json<NewUser>,
) -> Result<HttpResponse, Error> {
    const PASSWORD_LEN: usize = 8;
    if item.password.len() < PASSWORD_LEN || !item.email.contains("@") {
        return Ok(HttpResponse::NotFound().json("Not proper user name or password"));
    }
    Ok(web::block(move || add_single_user(db, item))
        .await
        .map(|_| HttpResponse::Created().finish())
        .map_err(|_| HttpResponse::InternalServerError().json("Email in use"))?)
}

/// Called for route /login HTTP request type: POST, used for loging in.
///
/// Provided input JSON should be serialized to model [NewUser](../models/struct.NewUser.html)
///
/// Returns Ok with json: {"token":"user_token"} for success.
pub async fn login(db: web::Data<Pool>, user: web::Json<NewUser>) -> Result<HttpResponse, Error> {
    let result = web::block(move || login_user(db, user)).await;
    match result {
        Ok(token) => {
            let mut json = serde_json::Map::new();
            json.insert("token".to_string(), serde_json::Value::String(token));
            Ok(HttpResponse::Ok().json(json))
        }
        Err(err) => match err {
            actix_rt::blocking::BlockingError::Error(e) => {
                error!("Encountered error: {}", e);
                Err(e.into())
            }
            actix_rt::blocking::BlockingError::Canceled => {
                Ok(HttpResponse::InternalServerError().json("ERROR"))
            }
        },
    }
}

/// The GraphQL Playground route.
pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(juniper::http::playground::playground_source("/graphql"))
}

/// Struct used for FromRequest trate implementation,
/// needed to extract User ID from authentication header.
pub struct IdWrapper {
    id: String,
}

impl FromRequest for IdWrapper {
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();
    type Error = Error;
    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> <Self as actix_web::FromRequest>::Future {
        let headers = req.headers();
        let auth_header = headers.get("authorization");
        match auth_header {
            Some(value) => {
                //Security vulnerability find better way to extrac/validate auth!!
                error!("auth header: {}", value.to_str().unwrap());
                let token: Vec<&str> = value.to_str().unwrap().split(" ").collect();
                let u_id = verify_token(&String::from(token[1])).unwrap();
                ok(IdWrapper {
                    id: String::from(u_id),
                })
            }
            None => err(BadRequest(String::from("No auth")).into()),
        }
    }
}
/// The core handler that provides all GraphQL functionality.
pub async fn graphql(
    pool: web::Data<Pool>,
    schema: web::Data<Arc<Schema>>,
    data: web::Json<juniper::http::GraphQLRequest>,
    id: IdWrapper,
) -> Result<HttpResponse, Error> {
    let ctx = GraphQLContext {
        user_id: Some(id.id),
        pool: pool.get_ref().to_owned(),
    };

    let res = web::block(move || {
        let res = data.execute(&schema, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await
    .map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(res))
}

fn login_user(db: web::Data<Pool>, user: web::Json<NewUser>) -> Result<String, ServiceError> {
    let conn = db.get().unwrap();
    let db_user = users.filter(email.eq(&user.email)).first::<User>(&conn);
    match db_user {
        Ok(db_user) => {
            let pass = db_user.password;
            if verify(pass.as_str(), user.password.as_bytes()) {
                match db_user.token {
                    Some(token) => Ok(token),
                    None => Err(ServiceError::InternalServerError),
                }
            } else {
                Err(ServiceError::InternalServerError)
            }
        }
        Err(_) => Err(ServiceError::InternalServerError),
    }
}

fn add_single_user(
    db: web::Data<Pool>,
    item: web::Json<NewUser>,
) -> Result<User, diesel::result::Error> {
    let conn = db.get().unwrap();
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let token = generate_token(&uuid).unwrap();
    let new_user = DbUser {
        user_id: &uuid,
        email: &item.email,
        password: &hash(item.password.as_bytes()),
        jwt: &token.as_str(),
    };
    let res = diesel::dsl::insert_into(users)
        .values(&new_user)
        .get_result(&conn)?;
    Ok(res)
}
