htmx.defineExtension('my-enc', {
	onEvent: function(name, evt) {
		if (name === 'htmx:configRequest') {
			evt.detail.headers['Content-Type'] = 'application/json'
		}
	},
	encodeParameters: function(xhr, parameters, elt) {
		console.log(xhr)
		console.log(parameters)
		console.log(elt)
		console.log(elt.elements)
		xhr.overrideMimeType('text/json')
		const json = {};

		function parseForm(fieldset, obj) {
			const fieldsetName = fieldset.name;
			if (!fieldsetName) {
				fieldsetName = "fieldset"
			};

			if (obj[fieldsetName] == null) {
				obj[fieldsetName] = [];
			};

			const fieldset_values = {};
			const elements = Array.from(fieldset.children).filter(el => el.tagName.toLowerCase() !== 'fieldset');

			elements.forEach(field => {
				if (field.name) {
					fieldset_values[field.name] = field.value;
				}
			});

			const nestedFieldsets = fieldset.querySelectorAll('fieldset');
			nestedFieldsets.forEach(nestedFieldset => {
				parseForm(nestedFieldset, fieldset_values);
			});
			obj[fieldsetName].push(fieldset_values)
		}

		const topLevelFieldsets = Array.from(elt.children).filter(el => {
			return el.tagName.toLowerCase() == 'fieldset';
		});

		topLevelFieldsets.forEach(fieldset => {
			parseForm(fieldset, json);
		});

		const otherElements = Array.from(elt.children).filter(el => {
			return el.tagName.toLowerCase() !== 'fieldset';
		});

		otherElements.forEach(element => {
			if (json[element.name] == null) {
				json[element.name] = [];
			}
			json[element.name].push(element.value);
		});

		return JSON.stringify(json);
	}
})