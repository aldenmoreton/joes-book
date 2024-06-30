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
		console.log(elt.elements);
		function parseForm(fieldLevel, obj) {
			const elements = Array.from(fieldLevel.elements);

			const inputs = elements.filter(currElement => {
				const tagName = currElement.tagName.toLowerCase();
				return tagName === 'select' || tagName === 'textarea' || tagName === 'input';
			});

			const fieldsets = elements.filter(currElement => {
				const tagName = currElement.tagName.toLowerCase();
				return tagName === 'fieldset';
			});

			inputs
				.filter(currElement => !fieldsets.some(currFieldset => currFieldset.contains(currElement)))
				.forEach(field => {
					if (field.name) {
						obj[field.name] = field.value;
					}
				});

			const nextLevelFieldsets = fieldsets.filter(currFieldset =>
				!fieldsets.some(parentFieldset => currFieldset != parentFieldset && parentFieldset.contains(currFieldset))
			)
			nextLevelFieldsets.forEach(nestedFieldset => {
				var nestedName = nestedFieldset.name;
				const insertMode = nestedFieldset.getAttribute("me-insert") || myenc.config.insertMode;
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