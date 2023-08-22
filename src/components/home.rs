use leptos::*;

use crate::components::TimezoneDropdown;

#[component]
pub fn Home(
    cx: Scope,
) -> impl IntoView {

    view! {
        cx,
		<h1 class="text-blue-500">"Green!"</h1>
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
		<div class="inline-block fill-red-500  h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]" role="status">
  			<span class="!absolute fill-red-500 !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]">Loading...</span>
		</div>
		<div
  			class="inline-block h-8 w-8 animate-[spinner-grow_0.75s_linear_infinite] rounded-full bg-current align-[-0.125em] text-primary opacity-0 motion-reduce:animate-[spinner-grow_1.5s_linear_infinite]"
  			role="status">
  			<span
    			class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]"
    			>Loading...</span>
		</div>
    }
}