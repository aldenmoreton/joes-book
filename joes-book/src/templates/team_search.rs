use crate::db::team::Team;

pub fn markup(teams: Vec<Team>, location: &str) -> maud::Markup {
    maud::html! {
        @for team in teams {
            li class="border border-black" {
                @let vals = format!(r#""location": "{location}", "name": "{}", "id": {}, "logo": "{}""#, team.name, team.id, team.logo.clone().unwrap_or_default());
                button hx-post="team-select" hx-ext="json-enc" hx-vals=(vals) {
                    @if let Some(logo) = team.logo {
                        img src=(logo) class="object-scale-down w-5 h-5" alt="";
                    }
                    (team.name)
                }
            }
        }
    }
}
