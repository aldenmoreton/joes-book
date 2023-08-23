use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Link rel="icon" type_="image/ico" href="/favicon.ico"/>
        <Link rel="manifest" href="/manifest.json"/>
        <Stylesheet id="leptos" href="/pkg/joes_book.css"/>
        <body class="bg-green-600 justify-center text-center">
        <Router>
            <Routes>
                <Route path="" view=Header>
                    <Route path="" view=Home/>
                    <Route path="logout" view=Logout/>
                    <Route path="books" view=Books/>
                    <Route path="books/:id" view=Book/>
                    <Route path="books/:id/new" view=NewChapter/>
                    <Route path="admin" view=Admin/>
                </Route>
                <Route path="signup" view=Signup/>
                <Route path="login" view=Login/>
            </Routes>
        </Router>
        </body>
    }
}
