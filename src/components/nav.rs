use askama::Template;

use crate::auth::AuthSession;

#[derive(Template)]
#[template(path = "components/nav.html")]
pub struct Nav {
    username: String,
}

pub async fn user(auth_session: AuthSession) -> Nav {
    let username = auth_session.user.unwrap().username;

    Nav { username }
}
