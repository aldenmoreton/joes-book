use crate::book::chapter::create::AddEventType;

pub fn markup(ty: AddEventType) -> maud::Markup {
    maud::html! {
        @match ty {
            AddEventType::SpreadGroup => {
                fieldset form="submit-event" name="events" me-insert="array" {
                    input type="hidden" name="type" value="spread-group";
                    (spread_group_spread())
                    button hx-get="add?type=spread-group-append" hx-target="previous" hx-swap="beforeend" {
                        "Add Spread"
                    }
                }
            },
            AddEventType::SpreadGroupAppend => (spread_group_spread()),
            AddEventType::UserInput => {
                fieldset form="submit-event" name="events" me-insert="array" {
                    h3 { "User Input" }
                    input type="hidden" name="type" value="user-input";
                    input type="text" name="title" placeholder="Title" form="submit-events" required;
                    input type="text" name="description" placeholder="Description (optional)" form="submit-events";
                    input type="number" name="points" placeholder="Number of Points" value="1" required;
                }
            }
        }
    }
}

fn spread_group_spread() -> maud::Markup {
    maud::html! {
        fieldset name="spreads" me-insert="array" {
            div title="team-1" hx-target="this" {
                p { "Spread Team 1 (Home)" }
                input type="search" name="name" placeholder="Team 1" autocomplete="off" hx-get="/team-search" hx-trigger="input changed delay:500ms, search" hx-vals=r#"{"location":"home-id"}"# hx-target="next ul" ;
                ul {}
                input form="submit-events" type="text" name="team-1-validate" style="opacity: 0; width: 0;" oninvalid="this.setCustomValidity('Missing Spread Team 1')" required;
            }

            div title="team-2" hx-target="this" {
                p { "Spread Team 2 (Away)" }
                input type="search" name="name" placeholder="Team 2" autocomplete="off" hx-get="/team-search" hx-trigger="input changed delay:500ms, search" hx-vals=r#"{"location":"away-id"}"# hx-target="next ul" ;
                ul {}
                input form="submit-events" type="text" name="team-2-validate" style="opacity: 0; width: 0;" oninvalid="this.setCustomValidity('Missing Spread Team 2')" required;
            }

            div title="spread-amount" {
                input type="number" name="home-spread" placeholder="Home Spread Amount" form="submit-events" step="0.5" required;
            }
        }
    }
}
