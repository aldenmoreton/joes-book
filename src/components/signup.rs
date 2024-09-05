// use anyhow::Ok;
use leptos::*;
use leptos_router::ActionForm;

use crate::server::Signup;

#[component]
pub fn Signup() -> impl IntoView {
    let signup = create_server_action::<Signup>();

    view! {
        <div class="flex flex-col items-center justify-center pt-10">
            <div class="w-full max-w-xs">
                <ActionForm action=signup class="px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md">
                    <div class="mb-4">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="username">"Username"</label>
                        <input class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="username" name="username" type="text" placeholder="Username"/>
                    </div>
                    <div class="mb-6">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="password">"Password"</label>
                        <input class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="password" name="password" type="password" placeholder="Password"/>
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="password_confirmation">"Confirm Password"</label>
                        <input class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="password_confirmation" name="password_confirmation" type="password" placeholder="Password"/>
                        <label>
                            "Remember me?"
                            <input type="checkbox" name="remember" class="auth-input" />
                        </label>
                    </div>
                    <button class="px-4 py-2 font-bold text-white bg-green-500 rounded hover:bg-green-700 focus:outline-none focus:shadow-outline" type="submit">
                        "Sign Up"
                    </button>
                    <div class="pt-3 text-sm font-bold">
                        <p>"Already have an account?"</p>
                        <a class="text-green-500 hover:text-green-800" href="/login">"Sign In"</a>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
