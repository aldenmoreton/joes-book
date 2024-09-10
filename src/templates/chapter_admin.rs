use std::collections::HashMap;

use crate::db::{
    chapter::Chapter,
    event::{Event, EventContent},
    spread::Spread,
    user_input::UserInput,
};

use super::authenticated;

pub fn markup(
    username: &str,
    book_name: &str,
    chapter: Chapter,
    events: Vec<Event>,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    authenticated(
        username,
        Some(&format!("{} - Admin", chapter.title)),
        None,
        Some(maud::html! {
            script src="/public/js/my-enc.js" {}
            (crate::templates::alertify())
        }),
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a href="../../.." class="text-blue-400 hover:underline" { (book_name) } " > "
                a href=".." class="text-blue-400 hover:underline" {(chapter.title)} " > "
                a {"Admin"}
            }
        }),
        Some(maud::html! {
            div class="flex flex-col items-center justify-center" {
                div class="self-center justify-center p-2 m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
                    (chapter_open_button(chapter.is_open))

                    (chapter_visible_button(chapter.is_visible))
                }

                div class="flex items-center justify-center" {
                    details class="flex items-center w-max" hx-target="this" {
                        summary class="p-3 my-1 align-middle bg-green-500 rounded-lg shadow-md select-none" {
                            "Unsubmitted Users"
                        }
                        div hx-get="unsubmitted-users" hx-trigger="load" hx-swap="outerHTML" class="flex items-center" {
                            "Loading..."
                        }
                    }
                }

                form hx-post="." hx-ext="my-enc" hx-swap="afterend" {
                    @for (i, event) in events.into_iter().enumerate() {
                        fieldset name="events" me-insert="array" class="flex items-center justify-center" {
                            input type="hidden" name="event-id" value=(event.id);
                            @match event.contents.0 {
                                EventContent::SpreadGroup(spreads) => {
                                    input type="hidden" name="type" value="spread-group";
                                    (spread_group(spreads, i, &relevent_teams))
                                },
                                EventContent::UserInput(input) => {
                                    input type="hidden" name="type" value="user-input";
                                    (user_input(input, event.id, i))
                                }
                            }
                        }
                    }

                    button type="submit" class="px-2 py-2 mt-1 mb-10 font-bold text-white bg-green-600 rounded hover:bg-green-700" {
                        "Submit"
                    }
                    div {}

                    details class="my-5" {
                        summary {
                            span class="text-red-500" {"Danger Zone"}
                        }
                        button
                            hx-delete="."
                            hx-confirm="Are you sure you wish to delete this chapter and all the picks within FOREVER?"
                            class="p-0.5 font-bold text-white bg-red-600 rounded hover:bg-red-700" {
                            "Delete Chapter"
                        }
                    }
                }
            }

        }),
        None,
    )
}

pub fn chapter_open_button(is_open: bool) -> maud::Markup {
    maud::html! {
        div hx-target="this" {
            @if is_open {
                p { "Chapter Status: Open" }
                button hx-post="open?toggle=false" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                    "Close"
                }
            } @else {
                p { "Chapter Status: Closed" }
                button hx-post="open?toggle=true" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                    "Open"
                }
            }
        }
    }
}

pub fn chapter_visible_button(is_visible: bool) -> maud::Markup {
    maud::html! {
        div hx-target="this" {
            @if is_visible {
                p { "Chapter Visibility: Visible" }
                button hx-post="visible?toggle=false" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                    "Hide"
                }
            } @else {
                p { "Chapter Visibility: Hidden" }
                button hx-post="visible?toggle=true" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                    "Show"
                }
            }
        }
    }
}

fn spread_group(
    spreads: Vec<Spread>,
    index: usize,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    maud::html! {
        div class="m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
            @for (i, spread) in spreads.into_iter().enumerate() {
                div class="grid grid-flow-col grid-cols-2 p-2" {
                    div class="col-span-1 mr-0.5" {
                        input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="home" id={(index)"-"(i)"-home"} checked[spread.answer == Some("home".into())];
                        label for={(index)"-"(i)"-home"} class="inline-grid w-full h-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                            div {
                                p class="font-semibold" { "Home" }
                                img src=(relevent_teams[&spread.home_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Home Team Logo";
                                p { (format!("{:+}", spread.home_spread)) " " (relevent_teams[&spread.home_id].0) }
                            }
                        }
                    }

                    div class="col-span-1 ml-0.5" {
                        input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="away" id={(index)"-"(i)"-away"} checked[spread.answer == Some("away".into())];
                        label for={(index)"-"(i)"-away"} class="inline-grid w-full h-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                            div {
                                h3 class="font-semibold" { "Away" }
                                img src=(relevent_teams[&spread.away_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Away Team Logo";
                                p { (format!("{:+}", -1. * spread.home_spread)) " " (relevent_teams[&spread.away_id].0) }
                            }
                        }
                    }

                }

                div {
                    input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="push" id={(index)"-"(i)"-push"} checked[spread.answer == Some("push".into())];
                    label for={(index)"-"(i)"-push"} class="inline-grid w-10/12 p-5 pt-0 pb-0 mb-1 border border-black rounded-lg cursor-pointer hover:border-orange-700 peer-checked:bg-orange-500 peer-checked:border-orange-600 hover:bg-orange-100" {
                        p class="px-1 font-semibold" { "Push" }
                    }
                }
                div {
                    input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="unpicked" id={(index)"-"(i)"-unpicked"} checked[spread.answer.is_none() || spread.answer == Some("unpicked".into())];
                    label for={(index)"-"(i)"-unpicked"} class="inline-grid w-10/12 p-5 pt-0 pb-0 mb-1 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                        p class="px-1 font-semibold" { "Unpicked" }
                    }
                }
            }
        }
    }
}

fn user_input(input: UserInput, event_id: i32, _index: usize) -> maud::Markup {
    maud::html! {
        div class="p-2 m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
            h3 class="text-lg font-bold" {
                (input.title)
            }
            @if let Some(description) = input.description {
                p { (description) }
            }
            div hx-get={"user-input?event-id="(event_id)} hx-swap="outerHTML" hx-trigger="load" {
                p {
                    "Loading..."
                }
            }
        }
    }
}
