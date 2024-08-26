pub fn markup() -> maud::Markup {
    crate::templates::base(
        Some("Login"),
        Some(maud::html!(
            (crate::templates::alertify())
            script src="https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback" defer {}
            script {
                "window.onloadTurnstileCallback = function () {
                    turnstile.render('#example-container', {
                        sitekey: '1x00000000000000000000AA',
                        callback: function(token) {
                            document.getElementById('submit-button').disabled = false;
                            console.log(`Challenge Success ${token}`);
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
                    form hx-post="/login" {
                        div class="px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md" {
                            h1 { "Log In" }
                            label class="block mb-2 text-sm font-bold text-gray-700" {
                                "Username:"
                                input type="text" placeholder="Username" maxlength="32" name="username" class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                            }

                            label class="block mb-2 text-sm font-bold text-gray-700" {
                                "Password:"
                                input type="password" placeholder="Password" name="password" class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                            }

                            button disabled id="submit-button" type="submit" class="px-4 py-2 font-bold text-white bg-green-500 rounded disabled:cursor-wait disabled:bg-gray-400 hover:bg-green-700 focus:outline-none focus:shadow-outline" {
                                "Log In"
                            }
                        }
                        div id="example-container" {}
                    }
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
