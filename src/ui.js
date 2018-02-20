"use strict";

// Functions to be called from rust
function invalidproject() {
	external.invoke("invalidproject")
}

function newproject() {
	external.invoke("newproject")
	//if (!confirm("Create a new laspad project here?")) {
	if (true) {
		alert("test")
		external.invoke("init")
		return
	}
	return existingproject()
}

function existingproject() {
	external.invoke("existingproject")
}
