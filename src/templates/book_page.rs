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
                a href="chapter/create/" {
                    button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                        "Create New Chapter"
                    }
                }
                br;
                a href="admin/" {
                    button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                        "Admin"
                    }
                }
            }

            (chapter_list::markup(book_subscription.book_id, chapters))
        }),
        None,
    )
}
