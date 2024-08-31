use maud::{html, Markup, DOCTYPE};

pub mod add_event;
pub mod book_list;
pub mod chapter_admin;
pub mod chapter_create;
pub mod chapter_list;
pub mod chapter_open;
pub mod home_page;
pub mod login_page;
pub mod team_search;
pub mod team_select;

pub fn base(
    page_title: Option<&str>,
    open_graph: Option<Markup>,
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
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title {(page_title.unwrap_or("Joe's Book"))}
                link rel="icon" type="image/ico" href="/public/favicon.ico";
                link rel="manifest" href="/public/manifest.json";
                link rel="stylesheet" id="tailwind" href="/public/styles/bookie.css";
                script src="/public/js/htmx.js" {}

                @if let Some(graph) = open_graph {
                    (graph)
                } @else {
                    meta property="og:title" content="Joe's Book";
                    meta property="og:image" content="/public/icon-512x512.png";
                    meta property="og:image:type" content="image/png";
                    meta property="og:image:width" content="512";
                    meta property="og:image:height" content="512";
                    meta property="og:image:alt" content="Site Logo";
                    meta
                        property="og:description"
                        content="Sports Picks";
                }
                meta property="og:site_name" content="Joe's Book";

                (head.unwrap_or_default())
            }
            body class="justify-center text-center bg-green-50" {
                script {"0"}
                @if let Some(header_markup) = header {
                    header { (header_markup) }
                    hr;
                }

                @if let Some(header_markup) = main {
                    main { (header_markup) }
                }

                @if let Some(header_markup) = footer {
                    hr;
                    footer { (header_markup) }
                }
            }
        }
    }
}

pub fn authenticated(
    username: &str,
    page_title: Option<&str>,
    open_graph: Option<Markup>,
    head: Option<Markup>,
    header: Option<Markup>,
    main: Option<Markup>,
    footer: Option<Markup>,
) -> Markup {
    base(
        page_title,
        open_graph,
        head,
        Some(html! {
            nav {
                // a href="/" { p { "Home" } }
                // button hx-post="/logout" { "Logout" }
                p { (username) }
                (header.unwrap_or_default())
            }
        }),
        main,
        footer,
    )
}

pub fn alertify() -> maud::Markup {
    maud::html! {
        script src="/public/js/alertify.js" {}
        link rel="stylesheet" href="/public/styles/alertify-main.css";
        link rel="stylesheet" href="/public/styles/alertify-theme.css";
        script {
            "window.addEventListener('load', function() {
                document.body.addEventListener('htmx:beforeOnLoad', function (evt) {
                    if (evt.detail.xhr.getResponseHeader('content-type') === 'text/html; charset=utf-8') {
                        evt.detail.shouldSwap = true;
                        evt.detail.isError = false;
                    }
                });
            })"
        }
    }
}
