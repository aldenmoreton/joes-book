use crate::chapter::create::AddEventType;

pub fn markup(ty: AddEventType) -> maud::Markup {
    maud::html! {
        @match ty {
            AddEventType::SpreadGroup => {
                fieldset form="submit-events" name="events" me-insert="array" class="self-center justify-center p-2 m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
                    h3 class="font-semibold" { "Spread Group" }
                    input type="hidden" name="type" value="spread-group";
                    (spread_group_spread())
                    button hx-get="add?type=spread-group-append" hx-swap="beforebegin" class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                        "Add Spread"
                    }
                }
            },
            AddEventType::SpreadGroupAppend => (spread_group_spread()),
            AddEventType::UserInput => {
                fieldset form="submit-events" name="events" me-insert="array" class="self-center justify-center m-3 bg-white border border-gray-300 rounded-lg shadow-md w-fit" {
                    h3 class="font-semibold" { "User Input" }
                    input type="hidden" name="type" value="user-input";
                    input type="text" name="title" placeholder="Title" form="submit-events" required class="text-center";
                    br;
                    input type="text" name="description" placeholder="Description (optional)" form="submit-events" class="text-center";
                    br;
                    input type="number" name="points" placeholder="Number of Points" value="1" required class="text-center";
                }
            }
        }
    }
}

fn spread_group_spread() -> maud::Markup {
    maud::html! {
        fieldset name="spreads" me-insert="array" class="border border-black rounded-md" {
            p { "Spread Team 1 (Home)" }
            div title="team-1" hx-target="this" {
                input type="search" name="name" placeholder="Team 1" autocomplete="off" hx-get="/team-search" hx-trigger="input changed delay:500ms, search" hx-vals=r#"{"location":"home-id"}"# hx-target="next ul" ;
                ul {}
                input form="submit-events" type="text" name="team-1-validate" style="opacity: 0; width: 0;" oninvalid="this.setCustomValidity('Missing Spread Team 1')" required;
            }

            p { "Spread Team 2 (Away)" }
            div title="team-2" hx-target="this" {
                input type="search" name="name" placeholder="Team 2" autocomplete="off" hx-get="/team-search" hx-trigger="input changed delay:500ms, search" hx-vals=r#"{"location":"away-id"}"# hx-target="next ul" ;
                ul {}
                input form="submit-events" type="text" name="team-2-validate" style="opacity: 0; width: 0;" oninvalid="this.setCustomValidity('Missing Spread Team 2')" required;
            }

            div title="spread-amount" {
                input type="number" name="home-spread" placeholder="Home Spread Amount" form="submit-events" step="0.5" required class="m-1";
            }
        }
    }
}
