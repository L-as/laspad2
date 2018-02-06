# About

This is a tool to publish and manage NS2 mods.

Check help.txt for help.

Thanks to @GhoulofGSG9 for explaining the undocumented steam api!

# Configuration
Configuration is made in TOML.

There is the concept of "branches", which closely resembles git's branches.
You can e.g. have a beta branch and a master branch.
Each branch has its own separate tags, description, preview, etc.
Each branch corresponds to a separate workshop item.

## README
If you set your description to a .md file, then it will
automatically be converted to steam's BBcode format.
Note, however, that there *are* limits to how well this works!

## Previews
Previews can be in any format, unlike Launch Pad.
You can use PNG, JPG, and have it in 4k, 8k, 32T, or anything like that.
