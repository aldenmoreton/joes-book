pub fn markup(site_key: &str) -> maud::Markup {
    crate::templates::base(
        Some("Login"),
        None,
        Some(maud::html!(
            (crate::templates::alertify())
            script src="https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback" defer {}
            script {
                "window.onloadTurnstileCallback = function () {
                    turnstile.render('#cf-turnstile-container', {
                        sitekey: '"(site_key)"',
                        callback: function(token) {
                            document.getElementById('submit-button').disabled = false;
                        },
                        theme: 'light',
                        action: 'login',
                    });
                };"
            }
        )),
        None,
        Some(maud::html! {
            div class="flex flex-col items-center justify-center pt-10" {
                div class="w-full max-w-xs" {
                    form hx-post="/legacy-login" hx-on--after-on-load="if (event.detail.xhr.status !== 200) {document.getElementById('submit-button').disabled = true;turnstile.reset('#cf-turnstile-container');}" {
                        div class="flex flex-col items-center px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md" {
                            h1 { "Log In" }
                            label class="block mb-2 text-sm font-bold text-gray-700" {
                                "Username:"
                                input required type="text" placeholder="Username" maxlength="32" name="username" class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                            }

                            label class="block mb-2 text-sm font-bold text-gray-700" {
                                "Password:"
                                input required type="password" placeholder="Password" name="password" class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                            }

                            div id="cf-turnstile-container" {}

                            button disabled id="submit-button" type="submit" class="px-4 py-2 font-bold text-white bg-green-500 rounded disabled:cursor-wait disabled:bg-gray-400 hover:bg-green-700 focus:outline-none focus:shadow-outline" {
                                "Log In"
                            }
                        }
                    }
                    div class="pt-3 text-sm font-bold" {
                        p { "Don't have an account?" }
                        a class="text-green-500 hover:text-green-800" href="/login" { "Create Account" }
                    }
                }
            }
        }),
        None,
    )
}
