release:
	cargo build --release
	rm -rf target/appdir
	mkdir  target/appdir
	cp target/release/laspad    target/appdir/AppRun
	cp assets/laspad.desktop    target/appdir/laspad.desktop
	cp assets/icon.png          target/appdir/laspad.png
	cp assets/steam_appid.txt   target/appdir/steam_appid.txt
	cp 3rdparty/libsteam_api.so target/appdir/libsteam_api.so
	appimagetool target/appdir laspad.AppImage
	
debug:
	cargo build
	rm -rf target/appdir
	mkdir  target/appdir
	cp target/debug/laspad      target/appdir/AppRun
	cp assets/laspad.desktop    target/appdir/laspad.desktop
	cp assets/icon.png          target/appdir/laspad.png
	cp assets/steam_appid.txt   target/appdir/steam_appid.txt
	cp 3rdparty/libsteam_api.so target/appdir/libsteam_api.so
	appimagetool target/appdir laspad.AppImage

clean:
	rm -rf target
	rm laspad.AppImage
