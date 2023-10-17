use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Link rel="icon" type_="image/ico" href="/favicon.ico"/>
        <Link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png"/>
        <Link rel="icon" type_="image/png" sizes="32x32" href="/favicon-32x32.png"/>
        <Link rel="icon" type_="image/png" sizes="16x16" href="/favicon-16x16.png"/>
        <Link rel="manifest" href="/manifest.json"/>
        <Stylesheet id="leptos" href="/pkg/joes_book.css"/>
        <Body class="justify-center text-center bg-green-50"/>
        <Router>
            <Routes>
                <Route path="" view=Header>
                    <Route path="" view=Books/>
                    <Route path="logout" view=Logout/>
                    <Route path="books" view=Books/>
                    <Route path="books/:book_id" view=Book/>
                    <Route path="books/:book_id/new" view=NewChapter/>
                    <Route path="books/:book_id/chapters/:chapter_id" view=Chapter/>
                    <Route path="books/:book_id/chapters/:chapter_id/grade" view=GradeChapter/>
                    <Route path="books/:book_id/chapters/:chapter_id/table" view=ChapterTable/>
                    <Route path="admin" view=Admin/>
                </Route>
                <Route path="signup" view=Signup/>
                <Route path="login" view=Login/>
            </Routes>
        </Router>
    }
}
