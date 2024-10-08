use std::collections::HashMap;

use itertools::izip;
use maud::{html, Markup};

use crate::db::{
    chapter::Chapter,
    event::{EventContent, Pick, UserPick},
    spread::Spread,
    user_input::UserInput,
};

use super::authenticated;

pub fn markup(
    username: &str,
    book_name: &str,
    chapter: &Chapter,
    user_picks: Vec<UserPick>,
    is_admin: bool,
    relevent_teams: HashMap<i32, (String, Option<String>)>,
) -> Markup {
    authenticated(
        username,
        Some(&chapter.title),
        None,
        Some(html! {
            script src="/public/js/my-enc.js" {}
            (crate::templates::alertify())
        }),
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a href="../.." class="text-blue-400 hover:underline" { (book_name) } " > "
                a {(chapter.title)}
            }
        }),
        Some(html! {
            @if is_admin {
                div class="flex justify-center" {
                    fieldset class="w-1/2 border border-orange-600 max-w-60" {
                        legend class="ml-3" { "Admin Section" }
                        a href="admin/" {
                            button class="p-2 my-1 text-sm font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Admin"
                            }
                        }
                    }
                }
            }
            form id="submit-picks" hx-post="." hx-ext="my-enc" {
                @if user_picks.is_empty() {
                    p { "No Events in this Chapter" }
                }
                @for (i, (event, pick)) in user_picks.into_iter().enumerate() {
                    fieldset name="events" me-insert="array" class="flex items-center justify-center" {
                        input type="hidden" name="event-id" value=(event.id);
                        @match event.contents.0 {
                            EventContent::SpreadGroup(spreads) => (spread_group(spreads, pick, i, &relevent_teams)),
                            EventContent::UserInput(input) => (user_input(input, pick))
                        }
                    }
                }
                button type="submit" class="px-2 py-2 mt-1 mb-10 font-bold text-white bg-green-600 rounded hover:bg-green-700" {
                    "Submit"
                }
            }
            div {}
        }),
        None,
    )
}

fn spread_group(
    spreads: Vec<Spread>,
    picks: Option<Pick>,
    index: usize,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> Markup {
    let (choices, wagers) = match picks {
        Some(Pick {
            choice: serde_json::Value::Array(choice),
            wager: serde_json::Value::Array(wager),
            ..
        }) => (choice, wager),
        _ => (
            vec![serde_json::Value::Null; spreads.len()],
            vec![serde_json::Value::Null; spreads.len()],
        ),
    };

    let num_spreads = spreads.len();

    html! {
        div class="m-3 bg-white border border-gray-300 rounded-lg shadow-md" {
            p class="text-lg font-semibold" { "Spreads" }
            input type="hidden" name="type" value="spread-group";
            @for (i, (spread, choice, wager)) in izip!(spreads, choices, wagers).enumerate() {
                fieldset name="spreads" me-insert="array" {
                    div class="grid grid-flow-col grid-cols-2 gap-4 p-2" {
                        div class="col-span-1" {
                            input type="radio" name=(format!("selection[{}-{}]", index, i)) class="absolute opacity-0 peer" value="home" id=(format!("{}-{}-home", index, i)) required checked[matches!(&choice, serde_json::Value::String(s) if s == "home")];
                            label for=(format!("{}-{}-home", index, i)) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                                div {
                                    h3 class="font-semibold" { "Home" }
                                    img src=(relevent_teams[&spread.home_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Home Team Logo";
                                    p { (relevent_teams[&spread.home_id].0) " " (format!("{:+}", spread.home_spread)) }
                                }
                            }
                        }

                        div class="col-span-1" {
                            input type="radio" name=(format!("selection[{}-{}]", index, i)) class="absolute opacity-0 peer" value="away" id=(format!("{}-{}-away", index, i)) required checked[matches!(&choice, serde_json::Value::String(s) if s == "away")];
                            label for=(format!("{}-{}-away", index, i)) class="inline-grid w-full p-5 pt-0 pb-0 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                                div {
                                    h3 class="font-semibold" { "Away" }
                                    img src=(relevent_teams[&spread.away_id].1.to_owned().unwrap_or_default()) width="150" height="150" alt="Away Team Logo";
                                    p { (relevent_teams[&spread.away_id].0) " " (format!("{:+}", -1. * spread.home_spread)) }
                                }
                            }
                        }
                    }

                    ul {
                        @for j in 1..=num_spreads {
                            li class="inline-flex items-center p-1" {
                                input type="radio" value=(j) name=(format!("num-points[{}-{}]", index, i)) id=(format!("{}-{}-{}", index, i, j)) class="absolute opacity-0 peer" required checked[matches!(&wager, serde_json::Value::Number(n) if n == &serde_json::Number::from(j))];
                                label for=(format!("{}-{}-{}", index, i, j)) class="flex items-center justify-center w-5 h-5 p-5 border border-black rounded-lg cursor-pointer hover:border-green-700 peer-checked:bg-green-500 peer-checked:border-green-600 hover:bg-green-100" {
                                    p class="text-xl font-bold" {
                                        (j)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn user_input(input: UserInput, pick: Option<Pick>) -> Markup {
    html! {
        div class="p-2 m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
            h3 class="text-lg font-semibold" { (input.title) }
            input type="hidden" name="type" value="user-input";
            @if let Some(description) = input.description {
                h4 { (description) }
            }

            label class="block mb-2 text-sm font-medium" {
                @let value = pick.and_then(|p| if let serde_json::Value::String(input) = p.choice {Some(input)} else {None});
                input type="text" name="user-input" placeholder="Make Pick" value=[value] required class="block p-1 ml-1 mr-1 text-sm text-center text-gray-900 border border-green-300 rounded-lg focus:ring-blue-500 focus:border-blue-500";
            }

            @if input.points == 1 {
                p {
                    "(" (input.points) " Point)"
                }
            } @else {
                p {
                    "(" (input.points) " Points)"
                }
            }
        }
    }
}
