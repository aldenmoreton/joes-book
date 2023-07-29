use crate::{
    // error_template::ErrorTemplate,
    components::{
        todo::Todos,
        signup::Signup,
        login::Login,
        logout::Logout,
        header::Header
    }
};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="leptos" href="/pkg/joes_book.css"/>
        <Router>
            <Routes>
                <Route path="" view=Header>
                    <Route path="" view=Todos/>
                    <Route path="logout" view=Logout/>
                </Route>
                <Route path="signup" view=Signup/>
                <Route path="login" view=Login/>
            </Routes>
        </Router>
    }
}
