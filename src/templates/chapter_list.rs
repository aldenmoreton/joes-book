use std::iter::Peekable;

use crate::db::chapter::Chapter;

pub fn markup<'a, I>(book_id: i32, mut chapters: Peekable<I>) -> maud::Markup
where
    I: Iterator<Item = &'a Chapter>,
{
    maud::html! {
        div class="flex flex-col items-center justify-center" {
            ul class="items-center self-center justify-center" {
                @if chapters.peek().is_none() {
                    li class="p-3 h-30 w-60" {
                        p { "No Chapters Yet!" }
                    }
                }
                @for chapter in chapters {
                    li class="p-3 h-30 w-60" {
                        div class="justify-center object-fill max-w-sm overflow-hidden bg-white rounded-lg shadow-lg" {
                            a href={"/book/"(book_id)"/chapter/"(chapter.chapter_id)"/"} class="object-fill" {
                                (chapter.title)
                            }
                        }
                    }
                }
            }
        }
    }
}