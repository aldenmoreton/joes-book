use crate::{
    error_template::ErrorTemplate,
    components::{
        todo::Todos,
        signup::Signup,
        login::Login,
        logout::Logout
    }
};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // let login = create_server_action::<Login>(cx);
    // let logout = create_server_action::<Logout>(cx);
    // let signup = create_server_action::<Signup>(cx);

    // let user = create_resource(
    //     cx,
    //     move || {
    //         (
    //             login.version().get(),
    //             signup.version().get(),
    //             logout.version().get(),
    //         )
    //     },
    //     move |_| get_user(cx),
    // );
    provide_meta_context(cx);

    // let logged_in = move || match user.read(cx) {
    //     Some(Ok(Some(_))) => true,
    //     _ => false
    // };

    view! {
        cx,
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="leptos" href="/pkg/joes_book.css"/>
        <Router>
            // <header>
            //     <A href="/"><h1>"My Tasks"</h1></A>
            //     <Transition
            //         fallback=move || view! {cx, <span>"Loading..."</span>}
            //     >
            //     {move || {
            //         user.read(cx).map(|user| match user {
            //             Err(e) => view! {cx,
            //                 <A href="/signup">"Signup"</A>", "
            //                 <A href="/login">"Login"</A>", "
            //                 <span>{format!("Login error: {}", e)}</span>
            //             }.into_view(cx),
            //             Ok(None) => view! {cx,
            //                 <A href="/signup">"Signup"</A>", "
            //                 <A href="/login">"Login"</A>", "
            //                 <span>"Logged out."</span>
            //             }.into_view(cx),
            //             Ok(Some(user)) => view! {cx,
            //                 <A href="/settings">"Settings"</A>", "
            //                 <span>{format!("Logged in as: {} ({})", user.username, user.id)}</span>
            //             }.into_view(cx)
            //         })
            //     }}
            //     </Transition>
            // </header>
            <hr/>
            <main>
                <Routes>
                    <Route
                        path=""
                        // redirect_path="foo"
                        // condition={move |_| logged_in()}
                        view=|cx| view! {
                            cx,
                            <ErrorBoundary fallback=|cx, errors| view!{cx, <ErrorTemplate errors=errors/>}>
                                <Todos/>
                            </ErrorBoundary>
                        }/> //Route
                    <Route path="foo" view=move |cx| view! {
                        cx,
                        <h1>bar</h1>
                    }/>
                    <Route path="signup" view=move |cx| view! {
                        cx,
                        <Signup/>
                    }/>
                    <Route path="login" view=move |cx| view! {
                        cx,
                        <Login/>
                    }/>
                    <Route path="settings" view=move |cx| view! {
                        cx,
                        <h1>"Settings"</h1>
                        <Logout/>
                    }/>
                </Routes>
            </main>
        </Router>
    }
}
