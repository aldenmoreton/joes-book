use leptos::*;

use crate::components::TimezoneDropdown;

#[component]
pub fn Home(
    cx: Scope,
) -> impl IntoView {

    view! {
        cx,
        <form>
			<select name="timezone" id="timezone">
				<option value="Pacific/Kwajalein">Eniwetok, Kwajalein</option>
				<option value="Pacific/Midway">Midway Island, Samoa</option>
				<option value="Pacific/Honolulu">Hawaii</option>
				<option value="Pacific/Marquesas">Taiohae</option>
			</select>
			<input type="datetime-local" id="meeting-time" name="meeting-time"/>
		</form>
		<form>
			<TimezoneDropdown/>
		</form>
    }
}