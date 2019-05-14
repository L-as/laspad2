#!/usr/bin/env fish

set -l target $argv[1]

for dep in (patchelf --print-needed $target)
	patchelf --remove-needed $dep $target
end

patchelf --add-needed libcrypto.so.1.1 $target
patchelf --add-needed libc.so.6 $target
patchelf --add-needed libdl.so.2 $target
patchelf --add-needed libgcc_s.so.1 $target
patchelf --add-needed libm.so.6 $target
patchelf --add-needed libpthread.so.0 $target
patchelf --add-needed librt.so.1 $target
patchelf --add-needed libssl.so.1.1 $target
patchelf --add-needed libsteam_api.so $target

patchelf --set-rpath '$ORIGIN' $target
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 $target
