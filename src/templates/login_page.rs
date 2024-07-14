pub fn markup() -> maud::Markup {
    crate::templates::base(
        Some("Login"),
        None,
        None,
        Some(maud::html! {
            form action="/login" method="post" {
                h1 { "Log In" }
                label {
                    "User ID:"
                    input type="text" placeholder="Username" maxlength="32" name="username" class="auth-input";
                }
                br;
                label {
                    "Password:"
                    input type="password" placeholder="Password" name="password" class="auth-input";
                }
                br;
                label {
                    "Remember Me?"
                    input type="checkbox" name="remember" class="auth-input";
                }
                br;
                button type="submit" class="button" { "Log In" }
            }
            p { "Don't have an account?" a href="signup" { "Sign Up" } }
        }),
        None,
    )
}
