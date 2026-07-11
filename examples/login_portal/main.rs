use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bcrypt::{DEFAULT_COST, hash, verify};

use getrandom::Error;
use rust_localserver::{
    file_server::FileServer,
    future::{AsyncTcpStream, AsyncWrite},
    http::{self, Cookie, Headers, Request, SameSite, SetCookie, Status},
};

struct AppState {
    /// username -> bcrypt hash of the password
    users: RefCell<HashMap<String, String>>,
    /// session id -> username
    sessions: RefCell<HashMap<String, String>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            users: RefCell::new(HashMap::new()),
            sessions: RefCell::new(HashMap::new()),
        }
    }
}
fn generate_session_id() -> Result<String, Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)?;
    Ok(hex::encode(bytes))
}

/// Reads and parses the `application/x-www-form-urlencoded` request body,
/// returning the `username`/`password` fields if both were present and
/// non-empty.
async fn read_credentials(req: &mut Request<&mut AsyncTcpStream>) -> Option<(String, String)> {
    let body = req.body().await.ok()?;
    let mut fields: HashMap<String, String> = form_urlencoded::parse(&body).into_owned().collect();

    let username = fields.remove("username")?;
    let password = fields.remove("password")?;
    (!username.is_empty() && !password.is_empty()).then_some((username, password))
}

async fn respond(req: Request<&mut AsyncTcpStream>, status: Status, body: &str) {
    respond_with(req, status, Headers::new(), body).await;
}

async fn respond_with(req: Request<&mut AsyncTcpStream>, status: Status, headers: Headers, body: &str) {
    _ = req
        .response()
        .status(status)
        .await
        .headers(headers)
        .await
        .write(body.as_bytes())
        .await;
}

async fn register(mut req: Request<&mut AsyncTcpStream>, state: &AppState) {
    let Some((username, password)) = read_credentials(&mut req).await else {
        return respond(req, Status::BadRequest, "username and password are required").await;
    };

    if state.users.borrow().contains_key(&username) {
        return respond(req, Status::BadRequest, "username is already taken").await;
    }

    let Some(hashed) = hash(password, DEFAULT_COST).ok() else {
        return respond(req, Status::InternalError, "could not create account").await;
    };

    state.users.borrow_mut().insert(username, hashed);
    respond(req, Status::OK, "account created").await;
}

async fn login(mut req: Request<&mut AsyncTcpStream>, state: &AppState) {
    let Some((username, password)) = read_credentials(&mut req).await else {
        return respond(req, Status::BadRequest, "username and password are required").await;
    };

    let is_valid = state
        .users
        .borrow()
        .get(&username)
        .is_some_and(|hashed| verify(password, hashed).unwrap_or(false));

    if !is_valid {
        return respond(req, Status::BadRequest, "invalid username or password").await;
    }

    let Ok(session_id) = generate_session_id() else {
        return respond(req, Status::InternalError, "could not start session").await;
    };
    state.sessions.borrow_mut().insert(session_id.clone(), username);

    let cookie = Cookie::new("session_id", &session_id)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    let mut headers = Headers::new();
    headers.set_cookie(cookie);
    respond_with(req, Status::OK, headers, "logged in").await;
}

async fn me(req: Request<&mut AsyncTcpStream>, state: &AppState) {
    let username = req
        .cookie("session_id")
        .and_then(|session_id| state.sessions.borrow().get(&session_id).cloned());

    match username {
        Some(username) => respond(req, Status::OK, &username).await,
        None => respond(req, Status::Unauthorized, "not logged in").await,
    }
}

async fn handle(req: Request<&mut AsyncTcpStream>, state: &AppState) {
    let file_server = FileServer::new("examples/login_portal", "index.html", false);

    let method: &str = &req.meta.method;
    let request_target: &str = &req.meta.request_target;
    match (method, request_target) {
        ("POST", "/register") => register(req, state).await,
        ("POST", "/login") => login(req, state).await,
        ("GET", "/me") => me(req, state).await,
        _ => file_server.serve(req).await,
    }
}

fn main() -> std::io::Result<()> {
    let state = Rc::new(AppState::new());
    let server = http::Server::bind("0.0.0.0:8080")?;

    let handler = async move |req: Request<&mut AsyncTcpStream>| handle(req, &state).await;
    server.serve_and_block(&handler);

    Ok(())
}
