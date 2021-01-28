//! Documentation for backend
//!
//! For API definitions go to [handlers](handlers)
//!
//! API:
//!
//! -register user URI:/register [register](handlers/fn.register_user.html)
//!
//! -login user URI:/login [login](handlers/fn.login.html)
//!
//! -graphql URI:/graphql [graphql](handlers/fn.graphql_playground.html)

#[macro_use]
extern crate diesel;

pub mod auth;
pub mod error;
pub mod graphql;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod utils;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
