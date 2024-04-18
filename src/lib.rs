pub mod auth;

pub mod routes {
    pub mod admin;
    pub mod book;
    pub mod home;
    pub mod login;
    pub mod signup;
}

pub mod components {
    pub mod book_list;
    pub mod nav;
}

pub mod objects {
    pub mod book;
    pub mod chapter;
    pub mod event;
    pub mod spread;
    pub mod user_input;
}
