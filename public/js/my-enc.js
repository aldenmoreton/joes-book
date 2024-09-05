const myenc = {
	config: {
		insertMode: "last"
	},
	parseForm: (fieldLevel) => {
		const nestedParameters = {};

		parseLevel(fieldLevel, nestedParameters);
		return nestedParameters;
	}
};

function parseLevel(fieldLevel, obj) {
	const elements = Array.from(fieldLevel.elements);

	const inputs = elements.filter(currElement => {
		const tagName = currElement.tagName.toLowerCase();
		return tagName !== 'fieldset';
	});

	const fieldsets = elements.filter(currElement => {
		const tagName = currElement.tagName.toLowerCase();
		return tagName === 'fieldset';
	});

	inputs
		.filter(currElement => !fieldsets.some(currFieldset => currFieldset.contains(currElement)))
		.forEach(field => {
			const checkable = ((field.type.toLowerCase() != 'checkbox' && field.type.toLowerCase() != 'radio') || field.checked !== false);
			if (field.name && checkable) {
				let fieldName = field.name.includes('[') ? field.name.split('[')[0] : field.name;
				if (obj[fieldName]) {
					switch (typeof obj[fieldName]) {
						case "string":
							obj[fieldName] = field.value;
							break;
						case "object":
							obj[fieldName].push(field.value);
							break;
					}
				} else {
					const insertMode = field.getAttribute("me-insert") || myenc.config.insertMode;
					switch (insertMode.toLowerCase()) {
						case "last":
							obj[fieldName] = field.value;
							break;
						case "array":
							obj[fieldName] = [field.value];
							break;
					}
				}
			}
		});

	const nextLevelFieldsets = fieldsets.filter(currFieldset =>
		!fieldsets.some(parentFieldset => currFieldset != parentFieldset && parentFieldset.contains(currFieldset))
	);
	nextLevelFieldsets.forEach(nestedFieldset => {
		var nestedName = nestedFieldset.name;
		if (!nestedName) {
			nestedName = "fieldset"
		}

		if (obj[nestedName]) {
			switch (typeof obj[nestedName]) {
				case "string":
					obj[nestedName] = {};
					parseLevel(nestedFieldset, obj[nestedName]);
					break;
				case "object":
					const nestedObj = {};
					obj[nestedName].push(nestedObj);
					parseLevel(nestedFieldset, nestedObj);
			}
		} else {
			const insertMode = nestedFieldset.getAttribute("me-insert") || myenc.config.insertMode;
			switch (insertMode.toLowerCase()) {
				case "last":
					obj[nestedName] = {};
					parseLevel(nestedFieldset, obj[nestedName]);
					break;
				case "array":
					const nestedObj = {};
					obj[nestedName] = [nestedObj];
					parseLevel(nestedFieldset, nestedObj);
					break;
			}
		}
	});
}

htmx.defineExtension('my-enc', {
	onEvent: function(name, evt) {
		if (name === 'htmx:configRequest') {
			evt.detail.headers['Content-Type'] = 'application/json';
		}
	},
	encodeParameters: function(xhr, parameters, elt) {
		xhr.overrideMimeType('text/json');

		return JSON.stringify(myenc.parseForm(elt));
	}
});
