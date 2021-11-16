use warp::{Filter, Rejection, Reply};

use crate::handlers::{admin, user};
use crate::model::user::Role;
use crate::Application;

mod jwt;

pub fn routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name))
        .or(user_routes(app))
        .or(admin_routes(app))
}

fn user_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("user").and(auth_routes(app))
}

fn auth_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(user::auth::register);
    let login = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(user::auth::login);
    let logout = warp::path("logout")
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![]))
        .and(jwt::refresh_filter())
        .and_then(user::auth::logout);
    let refresh_session = warp::path("refresh-session")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and(jwt::refresh_filter())
        .and_then(user::auth::refresh_session);
    let routes = register.or(login).or(logout).or(refresh_session);
    warp::path("auth").and(routes)
}

fn admin_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("admin").and(prize_routes(app))
}

fn prize_routes(
    app: &'static Application,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let prizes = warp::path("dates")
        .and(warp::get())
        .and(with_app(app))
        .and(jwt::jwt_filter(app, vec![Role::Admin]))
        .and_then(admin::prize::prizes);
    warp::path("queue").and(prizes)
}

pub fn with_app(
    app: &'static Application,
) -> impl Filter<Extract = (&Application,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app)
}
