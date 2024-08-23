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
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a { (book_subscription.name) }
            }
        }),
        Some(maud::html! {
            h1 class="text-4xl font-extrabold" {(book_subscription.name)}
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
                div class="flex justify-center" {
                    fieldset class="w-1/2 border border-black" {
                        legend { "Unpublished Chapters" }
                        (chapter_list::markup(book_subscription.book_id, chapters.iter().filter(|c| !c.is_visible).peekable()))
                    }
                }
            }

            (chapter_list::markup(book_subscription.book_id, chapters.iter().filter(|c| c.is_visible).peekable()))
        }),
        None,
    )
}
