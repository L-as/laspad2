#About

This is a tool to publish and manage NS2 mods.

Check help.txt for help.

Thanks to @GhoulofGSG9 for explaining the undocumented steam api!

#Installing on Linux
Run install.sh.
Set INSTALL_DIR to the installation directory.
It is by default /usr/local.

#Running
**Steam must be running!**

libsteam_api.so must be available somewhere, where your linker can
access it. This could be /usr/lib, something like it, or just a custom
path in $LD_LIBRARY_PATH.
