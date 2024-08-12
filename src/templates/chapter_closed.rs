use super::authenticated;

pub fn markup() -> maud::Markup {
    let is_admin = true;

    authenticated(
        "username",
        Some("Some(&chapter.title)"),
        None,
        None,
        Some(maud::html! {
            @if is_admin {
                a href="admin/" {
                    button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                        "Go to Admin Page"
                    }
                }
            }
        }),
        None,
    )
}
