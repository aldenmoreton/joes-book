use crate::{
    db::{
        book::{BookRole, BookSubscription},
        chapter::Chapter,
    },
    templates::chapter_list,
};

pub fn markup(
    username: &str,
    book_subscription: BookSubscription,
    chapters: Vec<Chapter>,
) -> maud::Markup {
    crate::templates::authenticated(
        username,
        Some(&book_subscription.name),
        None,
        None,
        Some(maud::html! {
            @if book_subscription.role == BookRole::Admin {
                a href="chapter/create/" { "Create New Chapter" }
                a href="admin/" { "Admin" }
            }

            (chapter_list::markup(book_subscription.book_id, chapters))
        }),
        None,
    )
}
