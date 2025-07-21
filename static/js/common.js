function triggerToast(text, success = false, enable_html = false) {
	const toastTrigger = document.getElementById('liveToastBtn')
	const toastLive = document.getElementById('liveToast')
	const toast = new bootstrap.Toast(toastLive)
	if (success) {
		toastLive.classList.remove('text-bg-danger');
		toastLive.classList.add('text-bg-success');
	} else {
		toastLive.classList.add('text-bg-danger');
		toastLive.classList.remove('text-bg-success');
	}

	if (enable_html)
		document.getElementById('toast_body').innerHTML = text;
	else
		document.getElementById('toast_body').innerText = text;

	toast.show()
}

function mapToObject(map) {
	if (map instanceof Map)
		return Object.fromEntries(map.entries());
	return map;
}

async function api_get(url) {
	let response = await fetch(url);
	let text = await response.text();
	if (response.status !== 200) {
		throw (`${response.status} - ${text}`);
	}
	if (text === "")
		return {};
	return JSON.parse(text);
}

async function api_post(url, body = null) {
	body = mapToObject(body);
	if (body !== null) {
		body = JSON.stringify(body);
	}
	let response = await fetch(url, {
		method: "POST",
		body: body,
		headers: new Headers({ 'content-type': 'application/json' }),
	});
	let text = await response.text();
	if (response.status !== 200) {
		throw (`${response.status} - ${text}`);
	}
	if (text === "")
		return {};
	return JSON.parse(text);
}

async function api_put(url, body = null) {
	body = mapToObject(body);
	if (body !== null) {
		body = JSON.stringify(body);
	}
	let response = await fetch(url, {
		method: "PUT",
		body: body,
		headers: new Headers({ 'content-type': 'application/json' }),
	});
	let text = await response.text();
	if (response.status !== 200) {
		throw (`${response.status} - ${text}`);
	}
	if (text === "")
		return {};
	return JSON.parse(text);
}

async function api_delete(url, body = null) {
	body = mapToObject(body);
	if (body !== null) {
		body = JSON.stringify(body);
	}
	let response = await fetch(url, {
		method: "DELETE",
		body: body,
		headers: new Headers({ 'content-type': 'application/json' }),
	});
	let text = await response.text();
	if (response.status !== 200) {
		throw (`${response.status} - ${text}`);
	}
	if (text === "")
		return {};
	return JSON.parse(text);
}


document.querySelectorAll(".spoiler").forEach(el => {
	el.addEventListener("click", () => {
		el.classList.add("revealed")
	})
})

window.api_post = api_post;
window.api_put = api_put;
window.api_get = api_get;
window.api_delete = api_delete;
window.triggerToast = triggerToast;
