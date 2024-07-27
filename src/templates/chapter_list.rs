use crate::db::chapter::Chapter;

pub fn markup(book_id: i32, chapters: Vec<Chapter>) -> maud::Markup {
    maud::html! {
        div class="flex flex-col items-center justify-center" {
            ul class="items-center self-center justify-center" {
                @if chapters.is_empty() {
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
