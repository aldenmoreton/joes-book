// use anyhow::Ok;
use leptos::*;
use leptos_router::ActionForm;

use crate::server::Signup;

#[component]
pub fn Signup(
    cx: Scope
) -> impl IntoView {
    let signup = create_server_action::<Signup>(cx);

    view! {
        cx,
        <div class="flex flex-col items-center justify-center pt-10">
            <div class="w-full max-w-xs">
                <ActionForm action=signup class="bg-white shadow-md rounded px-8 pt-6 pb-8 mb-4">
                    <div class="mb-4">
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="username">"Username"</label>
                        <input class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline" id="username" name="username" type="text" placeholder="Username"/>
                    </div>
                    <div class="mb-6">
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="password">"Password"</label>
                        <input class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 mb-3 leading-tight focus:outline-none focus:shadow-outline" id="password" name="password" type="password" placeholder="Password"/>
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="password_confirmation">"Confirm Password"</label>
                        <input class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 mb-3 leading-tight focus:outline-none focus:shadow-outline" id="password_confirmation" name="password_confirmation" type="password" placeholder="Password"/>
                        <label>
                            "Remember me?"
                            <input type="checkbox" name="remember" class="auth-input" />
                        </label>
                    </div>
                    <button class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline" type="submit">
                        "Sign Up"
                    </button>
                    <div class="font-bold text-sm pt-3">
                        <p>"Already have an account?"</p>
                        <a class="text-green-500 hover:text-green-800" href="/login">"Sign In"</a>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}