pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod wallpaper_handlers;

pub use auth::{AccountInfo, AdminService, DEFAULT_PASSWORD, DEFAULT_USERNAME};
