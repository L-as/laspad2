
--[[

A table with possible branches as keys should be
returned from this lua file.
The valid keys for each branch is `name`, `tags`, `description`, and `preview`.
The `name`, `description`, and `preview` have to be strings.
The `tags` should be a table of the tags as strings.

If any field is a function, the function will be called
with the mod id for the branch, if available.
The return value of the function will be used instead of the
function itself.


NB: Unlike the laspad configuration file, the description and preview
fields are not paths, but instead the actual data.

]]

return {
	-- variation name, this is the default
	master = {
		name            = "My mod",
		tags            = {"must be run on server", "gameplay tweak"},
		description     = laspad.get_description("README.md"), -- generates function
		preview         = laspad.read("preview.png"), -- equivalent to `function() return io.read "preview.png" :read "a" end`
	}
}
