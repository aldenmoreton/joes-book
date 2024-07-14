use crate::book::chapter::create::TeamSelect;

pub fn markup(team: TeamSelect) -> maud::Markup {
    maud::html! {
        div {
            p { (team.team.name) }
            img src=(team.team.logo.clone().unwrap_or(String::new())) class="object-scale-down w-20 h-20" alt="";
            input type="hidden" name=(team.location) value=(team.team.id);
        }
    }
}
