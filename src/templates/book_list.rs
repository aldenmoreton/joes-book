use crate::db::book::BookSubscription;

pub fn markup(books: Vec<BookSubscription>) -> maud::Markup {
    maud::html! {
        div class="flex flex-col items-center justify-center" {
            ul class="items-center self-center justify-center" {
                @if books.is_empty() {
                    li class="p-3 h-30 w-60" {
                        p { "No Books Yet!" }
                    }
                }
                @for book in books {
                    li class="p-3 h-30 w-60" {
                        div class="justify-center object-fill max-w-sm overflow-hidden bg-white rounded-lg shadow-lg" {
                            a href={"/book/"(book.book_id)"/"} class="object-fill" {
                                p { (book.name) }
                                p { (format!("{:?}", book.role)) }
                            }
                        }
                    }
                }
            }
        }
    }
}
