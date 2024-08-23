use crate::templates::authenticated;

pub fn markup(username: &str, book_name: &str) -> maud::Markup {
    authenticated(
        username,
        Some("Create Chapter"),
        Some(maud::html! {
            script src="/public/js/my-enc.js" {}
            script src="/public/js/json-enc.js" {}
        }),
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a href="../.." class="text-blue-400 hover:underline" { (book_name) } " > "
                a {"Create Chapter"}
            }
        }),
        Some(maud::html! {
            input type="text" name="chapter-name" placeholder="Chapter Name" form="submit-events" required;
            div id="event-spaces" {}

            button hx-get="add?type=spread-group" hx-target="#event-spaces" hx-swap="beforeend" hx-on--after-on-load="this.remove()" class="px-2 py-2 mx-1 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                "Add Spread Group"
            }
            button hx-get="add?type=user-input" hx-target="#event-spaces" hx-swap="beforeend" class="px-2 py-2 mx-1 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                "Add User Input"
            }

            form id="submit-events" hx-post="." hx-ext="my-enc" {
                button type="submit" class="p-2 mt-3 text-black bg-white border border-green-500 hover:border-black hover:bg-green-500 hover:text-white rounded-2xl" {
                    "Submit"
                }
            }
        }),
        None,
    )
}
