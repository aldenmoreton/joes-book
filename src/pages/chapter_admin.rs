use std::collections::HashMap;

use crate::db::{
    chapter::Chapter,
    event::{Event, EventContent},
    spread::Spread,
    user_input::UserInput,
};

use super::authenticated;

pub fn markup<'a>(
    username: &'a str,
    chapter: Chapter,
    events: Vec<Event>,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    authenticated(
        username,
        Some(&format!("{} - Admin", chapter.title)),
        None,
        None,
        Some(maud::html! {
            p { "Chapter Admin Page" }
            div class="flex flex-col items-center justify-center" {
                div class="self-center justify-center p-2 m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
                    div hx-target="this" {
                        @if chapter.is_open {
                            p { "Chapter Status: Open" }
                            button hx-post="" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Close"
                            }
                        } @else {
                            p { "Chapter Status: Closed" }
                            button hx-post="" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Open"
                            }
                        }
                    }

                    div hx-target="this" {
                        @if chapter.is_visible {
                            p { "Chapter Visibility: Visible" }
                            button hx-post="" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Hide"
                            }
                        } @else {
                            p { "Chapter Visibility: Hidden" }
                            button hx-post="" class="p-0.5 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Show"
                            }
                        }
                    }
                }

                form class="items-center self-center justify-center" {
                    @for (i, event) in events.into_iter().enumerate() {
                        fieldset name="events" me-insert="array" class="self-center justify-center m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
                            input type="hidden" name="type" value="spread-group";
                            input type="hidden" name="event-id" value=(event.id);
                            @match event.contents.0 {
                                EventContent::SpreadGroup(spreads) => (spread_group(spreads, i, &relevent_teams)),
                                EventContent::UserInput(input) => (user_input(input, i))
                            }
                        }
                    }
                    button type="submit" { "Submit" }
                }
            }

        }),
        None,
    )
}

fn spread_group(
    spreads: Vec<Spread>,
    index: usize,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    maud::html! {
        @for (i, spread) in spreads.into_iter().enumerate() {
            fieldset name="spreads" me-insert="array" {
                div class="grid grid-flow-col grid-cols-2 gap-4 p-5" {
                    div class="col-span-1" {
                        input type="radio" name=(format!("selection[{}-{}]", index, i)) class="absolute opacity-0 peer" value="home" id=(format!("{}-{}-home", index, i)) required;
                        label for=(format!("{}-{}-home", index, i)) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                            div {
                                img src=(relevent_teams[&spread.home_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Home Team Logo";
                                p { (relevent_teams[&spread.home_id].0) }
                            }
                        }
                    }

                     div class="col-span-1" {
                        input type="radio" name=(format!("selection[{}-{}]", index, i)) class="absolute opacity-0 peer" value="away" id=(format!("{}-{}-away", index, i)) required;
                        label for=(format!("{}-{}-away", index, i)) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                            div {
                                img src=(relevent_teams[&spread.away_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Away Team Logo";
                                p { (relevent_teams[&spread.away_id].0) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn user_input(input: UserInput, _index: usize) -> maud::Markup {
    maud::html! {
        p { (format!("{input:?}")) }
    }
}
