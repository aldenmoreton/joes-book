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
    chapter: Chapter,
    events: Vec<Event>,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    authenticated(
        username,
        Some(&format!("{} - Admin", chapter.title)),
        Some(maud::html! {
            script src="/public/js/my-enc.js" {}
        }),
        None,
        Some(maud::html! {
            a href = ".." {
                button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                            "Return to Chapter"
                }
            }
            div class="flex flex-col items-center justify-center" {
                div class="self-center justify-center p-2 m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
                    (chapter_open_button(chapter.is_open))

                    (chapter_visible_button(chapter.is_visible))
                }

                form class="items-center self-center justify-center" hx-post="." hx-ext="my-enc" hx-swap="afterend" {
                    @for (i, event) in events.into_iter().enumerate() {
                        fieldset name="events" me-insert="array" class="items-center self-center justify-center m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
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

                    button type="submit" class="px-2 py-2 mt-1 mb-5 font-bold text-white bg-green-600 rounded hover:bg-green-700" {
                        "Submit"
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
        @for (i, spread) in spreads.into_iter().enumerate() {
            div class="grid grid-flow-col grid-cols-2 gap-4 p-5" {
                div class="col-span-1" {
                    input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="home" id={(index)"-"(i)"-home"} checked[spread.answer == Some("home".into())] required;
                    label for={(index)"-"(i)"-home"} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                        div {
                            img src=(relevent_teams[&spread.home_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Home Team Logo";
                            p { (relevent_teams[&spread.home_id].0) }
                        }
                    }
                }

                div class="col-span-1" {
                    input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="away" id={(index)"-"(i)"-away"} checked[spread.answer == Some("away".into())] required;
                    label for={(index)"-"(i)"-away"} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                        div {
                            img src=(relevent_teams[&spread.away_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Away Team Logo";
                            p { (relevent_teams[&spread.away_id].0) }
                        }
                    }
                }

            }

            div {
                input type="radio" name={"selections["(index)"-"(i)"]"} me-insert="array" class="absolute opacity-0 peer" value="push" id={(index)"-"(i)"-push"} checked[spread.answer == Some("push".into())] required;
                label for={(index)"-"(i)"-push"} class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-orange-700 peer-checked:bg-orange-500 peer-checked:border-orange-600 hover:bg-orange-100" {
                    p class="px-1 font-semibold" { "Push" }
                }
            }
        }
    }
}

fn user_input(input: UserInput, event_id: i32, _index: usize) -> maud::Markup {
    maud::html! {
        h3 {
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
