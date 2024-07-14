use maud::{html, Markup, DOCTYPE};

pub mod chapter_admin;
pub mod chapter_open;

pub fn base(
    page_title: Option<&str>,
    head: Option<Markup>,
    header: Option<Markup>,
    main: Option<Markup>,
    footer: Option<Markup>,
) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title {(page_title.unwrap_or("Joe's Book"))}
                link rel="icon" type="image/ico" href="/public/favicon.ico";
                link rel="mainfest" href="/public/manifest.json";
                link rel="stylesheet" id="tailwind" href="/public/output.css";
                script src="/public/js/htmx.js" {}
                (head.unwrap_or_default())
            }
            body class="justify-center text-center bg-green-50" {
                script {"0"}
                @if let Some(header_markup) = header {
                    header { (header_markup) }
                }

                @if let Some(header_markup) = main {
                    main { (header_markup) }
                }

                @if let Some(header_markup) = footer {
                    footer { (header_markup) }
                }
            }
        }
    }
}

pub fn authenticated<'a>(
    username: &'a str,
    page_title: Option<&'a str>,
    head: Option<Markup>,
    header: Option<Markup>,
    main: Option<Markup>,
    footer: Option<Markup>,
) -> Markup {
    base(
        page_title,
        head,
        Some(html! {
            nav {
                a href="/" { p { "Home" } }
                button hx-post="/logout" { "Logout" }
                p { (username) }
                hr;
            }
            (header.unwrap_or_default())
        }),
        main,
        footer,
    )
}
