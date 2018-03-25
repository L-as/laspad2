"use strict";

var active = false;

function log(msg, type) {
	let node = document.createElement("span")
	node.innerText = "\n" + msg
	node.className = type
	let log = document.getElementById("log")
	log.appendChild(node)
	log.scrollTop = log.scrollHeight
}

function set_active(active) {
	let commands = [].slice.call(document.getElementsByClassName("command"));
	let branches = [].slice.call(document.getElementsByClassName("branch"));
	let buttons = commands.concat(branches)
	for (let i = 0; i < buttons.length; ++i) {
		buttons[i].disabled = active
		if(!buttons[i].old_background_color) buttons[i].old_background_color = getComputedStyle(buttons[i]).getPropertyValue("background-color")
		buttons[i].style.backgroundColor = active ? "grey" : buttons[i].old_background_color
	}
}

function post(target, callback) {
	let req = new XMLHttpRequest()
	req.addEventListener("load", callback)
	req.open("POST", target)
	req.overrideMimeType("text/plain")
	req.send()
}

function get_msg(resp, callback) {
	let error    = "ERR" // alert
	let warning  = "WRN" // red line in log
	let info     = "INF" // bold line in log
	let log      = "LOG" // white line in log
	let finished = "FIN" // handled by callback

	let msg = resp.substring(3)
	switch (resp.substring(0, 3)) {
	case error:
		set_active(false)
		window.log(msg, "warning")
		alert("Fatal error: " + msg)
		return
	case warning:
		window.log(msg, "warning")
		break
	case log:
		window.log(msg, "message")
		break
	case info:
		window.log(msg, "information")
		break
	case finished:
		set_active(false)
		return callback ? callback(msg) : undefined
	}
	post("/get_msg", function() {
		return get_msg(this.responseText, callback)
	})
}

function command(target, callback) {
	console.log("command("+target+")")
	set_active(true)
	post(target, function() {
		return get_msg(this.responseText, callback)
	})
}

function update() {
	command("/update")
}

function need() {
	command("/need?" + prompt("Mod ID of dependency to add"))
}

function ns2() {
	command("/ns2")
}

function editor() {
	command("/editor")
}

function get_branches(resp) {
	let branches = resp.split("")
	for(let i = 0; i < branches.length; i++) {
		let branch = branches[i]
		let node = document.createElement("button")
		node.innerText = "Publish " + branch
		node.className = "branch"
		node.addEventListener("click", function() {
			command("/publish?" + branch)
		})
		document.getElementById("publish").appendChild(node)
	}
}

command("/get_branches", get_branches)
