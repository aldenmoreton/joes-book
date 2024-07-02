const myenc = {
	config: {
		insertMode: "last"
	}
};

htmx.defineExtension('my-enc', {
	onEvent: function(name, evt) {
		if (name === 'htmx:configRequest') {
			evt.detail.headers['Content-Type'] = 'application/json';
		}
	},
	encodeParameters: function(xhr, parameters, elt) {
		xhr.overrideMimeType('text/json');
		const nestedParameters = {};

		function parseForm(fieldLevel, obj) {
			const elements = Array.from(fieldLevel.elements);

			const inputs = elements.filter(currElement => {
				const tagName = currElement.tagName.toLowerCase();
				return tagName != 'fieldset';
			});

			const fieldsets = elements.filter(currElement => {
				const tagName = currElement.tagName.toLowerCase();
				return tagName === 'fieldset';
			});

			inputs
				.filter(currElement => !fieldsets.some(currFieldset => currFieldset.contains(currElement)))
				.forEach(field => {
					if (field.name && field.checked !== false) {
						if (obj[field.name]) {
							switch (typeof obj[field.name]) {
								case "string":
									obj[field.name] = field.value;
									break;
								case "object":
									obj[field.name].push(field.value);
									break;
							}
						} else {
							const insertMode = field.getAttribute("me-insert") || myenc.config.insertMode;
							switch (insertMode.toLowerCase()) {
								case "last":
									obj[field.name] = field.value;
									break;
								case "array":
									obj[field.name] = [field.value];
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
							parseForm(nestedFieldset, obj[nestedName]);
							break;
						case "object":
							const nestedObj = {};
							obj[nestedName].push(nestedObj);
							parseForm(nestedFieldset, nestedObj);
					}
				} else {
					const insertMode = nestedFieldset.getAttribute("me-insert") || myenc.config.insertMode;
					switch (insertMode.toLowerCase()) {
						case "last":
							obj[nestedName] = {};
							parseForm(nestedFieldset, obj[nestedName]);
							break;
						case "array":
							const nestedObj = {};
							obj[nestedName] = [nestedObj];
							parseForm(nestedFieldset, nestedObj);
							break;
					}
				}
			});
		};

		parseForm(elt, nestedParameters);
		return JSON.stringify(nestedParameters);
	}
});