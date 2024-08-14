pub fn markup() -> maud::Markup {
    crate::templates::base(
        Some("Login"),
        Some(maud::html!(
            script src="/public/js/alertify.js" {}
            link rel="stylesheet" href="/public/styles/alertify-main.css";
            link rel="stylesheet" href="/public/styles/alertify-theme.css";
        )),
        None,
        Some(maud::html! {
            div class="flex flex-col items-center justify-center pt-10" {
                div class="w-full max-w-xs" {
                    form hx-post="/login" hx-swap="outerHTML" hx-target="next script" class="px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md" {
                        h1 { "Log In" }
                        label class="block mb-2 text-sm font-bold text-gray-700" {
                            "User ID:"
                            input type="text" placeholder="Username" maxlength="32" name="username" class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                        }
                        // br;
                        label class="block mb-2 text-sm font-bold text-gray-700" {
                            "Password:"
                            input type="password" placeholder="Password" name="password" class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                        }
                        // br;
                        // label {
                        //     "Remember Me?"
                        //     input type="checkbox" name="remember" class="auth-input";
                        // }
                        // br;
                        button type="submit" class="px-4 py-2 font-bold text-white bg-green-500 rounded hover:bg-green-700 focus:outline-none focus:shadow-outline" {
                            "Log In"
                        }
                    }
                    script {}
                    div class="pt-3 text-sm font-bold" {
                        p { "Don't have an account?" }
                        a class="text-green-500 hover:text-green-800" href="/signup" { "Create Account" }
                    }
                }
            }
        }),
        None,
    )
}
