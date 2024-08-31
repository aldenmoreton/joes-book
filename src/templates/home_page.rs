use crate::{db::book::BookSubscription, templates::book_list};

use super::authenticated;

pub fn markup(username: &str, is_admin: bool, books: Vec<BookSubscription>) -> maud::Markup {
    authenticated(
        username,
        Some("Home"),
        None,
        None,
        Some(maud::html! {
             p {
                a {"Home"}
            }
        }),
        Some(maud::html! {
            @if is_admin {
                form action="/book/create" method="post" {
                    label {
                        p {"Create New Book"}
                        input type="text" placeholder="Book Name" maxlength="32" name="book-name";
                    }
                    button type="submit" class="button" { "Create" }
                }
            }
            h3 class="h-3" { b { "Your Books"}}
            (book_list::markup(books))
        }),
        None,
    )
}
