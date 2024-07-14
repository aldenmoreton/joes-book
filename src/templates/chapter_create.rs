use crate::templates::authenticated;

pub fn markup(username: &str) -> maud::Markup {
    authenticated(
        username,
        Some("Create Chapter"),
        Some(maud::html! {
            script src="/public/js/my-enc.js" {}
            script src="/public/js/json-enc.js" {}
        }),
        None,
        Some(maud::html! {
            input type="text" name="chapter-name" placeholder="Chapter Name" form="submit-events" required;
            div id="event-space" {}

            button hx-get="add?type=spread-group" hx-target="#event-spaces" hx-swap="beforeend" hx-on--xhr-loadend="this.remove()" {
                "Add Spread Group"
            }
            button hx-get="add?type=user-input" hx-target="#event-spaces" hx-swap="beforeend" {
                "Add User Input"
            }

            form id="submit-events" hx-post="." hx-ext="my-enc" {
                button type="submit" { "Submit" }
            }
        }),
        None,
    )
}
