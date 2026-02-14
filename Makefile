.PHONY: build run clean test fmt lint install deps dist dist-mac dist-linux dist-windows

# Build targets
build:
	cargo build --release

build-debug:
	cargo build

# Run targets
run:
	cargo run --release

run-debug:
	cargo run

# Development targets
clean:
	cargo clean

test:
	cargo test

test-show:
	cargo test -- --show-output

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

# Dependency targets
deps:
	cargo update

# Install (if needed)
install: build
	@echo "Binary available at: target/release/fractal-oxide"

# Distribution targets
DIST_DIR := dist
APP_NAME := FractalOxide
APP_VERSION := 0.1.0

# Clean distribution directory
dist-clean:
	rm -rf $(DIST_DIR)
	mkdir -p $(DIST_DIR)

# macOS .dmg distribution
dist-mac: build dist-clean
	@echo "Building macOS .dmg package..."
	mkdir -p $(DIST_DIR)/mac/$(APP_NAME).app/Contents/MacOS
	mkdir -p $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Resources
	cp target/release/fractal-oxide $(DIST_DIR)/mac/$(APP_NAME).app/Contents/MacOS/$(APP_NAME)
	cp LICENSE $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Resources/
	@echo '<?xml version="1.0" encoding="UTF-8"?>' > $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '<plist version="1.0">' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '<dict>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleName</key>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <string>$(APP_NAME)</string>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleDisplayName</key>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <string>Fractal Oxide</string>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleIdentifier</key>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <string>net.ultrametrics.fractal-oxide</string>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleVersion</key>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <string>$(APP_VERSION)</string>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleExecutable</key>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <string>$(APP_NAME)</string>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <key>LSMinimumSystemVersion</key>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '    <string>10.11</string>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '</dict>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo '</plist>' >> $(DIST_DIR)/mac/$(APP_NAME).app/Contents/Info.plist
	@echo "Creating .dmg file..."
	@which hdiutil > /dev/null 2>&1 && hdiutil create -volname "$(APP_NAME)" -srcfolder $(DIST_DIR)/mac -ov -format UDZO $(DIST_DIR)/$(APP_NAME)-$(APP_VERSION)-macOS.dmg || echo "Note: hdiutil not available (requires macOS). App bundle created in $(DIST_DIR)/mac/"
	@echo "macOS distribution created: $(DIST_DIR)/$(APP_NAME)-$(APP_VERSION)-macOS.dmg"

# Linux .deb distribution for apt
dist-linux: build dist-clean
	@echo "Building Linux .deb package..."
	mkdir -p $(DIST_DIR)/linux/usr/bin
	mkdir -p $(DIST_DIR)/linux/usr/share/applications
	mkdir -p $(DIST_DIR)/linux/usr/share/doc/fractal-oxide
	mkdir -p $(DIST_DIR)/linux/DEBIAN
	cp target/release/fractal-oxide $(DIST_DIR)/linux/usr/bin/
	cp LICENSE $(DIST_DIR)/linux/usr/share/doc/fractal-oxide/copyright
	@echo "[Desktop Entry]" > $(DIST_DIR)/linux/usr/share/applications/fractal-oxide.desktop
	@echo "Name=Fractal Oxide" >> $(DIST_DIR)/linux/usr/share/applications/fractal-oxide.desktop
	@echo "Comment=Interactive fractal oxide" >> $(DIST_DIR)/linux/usr/share/applications/fractal-oxide.desktop
	@echo "Exec=fractal-oxide" >> $(DIST_DIR)/linux/usr/share/applications/fractal-oxide.desktop
	@echo "Type=Application" >> $(DIST_DIR)/linux/usr/share/applications/fractal-oxide.desktop
	@echo "Categories=Graphics;Education;Science;" >> $(DIST_DIR)/linux/usr/share/applications/fractal-oxide.desktop
	@echo "Package: fractal-oxide" > $(DIST_DIR)/linux/DEBIAN/control
	@echo "Version: $(APP_VERSION)" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo "Section: graphics" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo "Priority: optional" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo "Architecture: amd64" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo "Depends: libc6 (>= 2.17), libgcc1 (>= 1:4.1.1)" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo "Maintainer: ultrametrics.net <contact@ultrametrics.net>" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo "Description: Interactive fractal oxide" >> $(DIST_DIR)/linux/DEBIAN/control
	@echo " Built with Rust, supports 12 fractal types with GPU-accelerated rendering." >> $(DIST_DIR)/linux/DEBIAN/control
	@echo " Features include bookmarks, undo/redo, and multiple color palettes." >> $(DIST_DIR)/linux/DEBIAN/control
	@which dpkg-deb > /dev/null 2>&1 && dpkg-deb --build $(DIST_DIR)/linux $(DIST_DIR)/fractal-oxide_$(APP_VERSION)_amd64.deb || echo "Note: dpkg-deb not available. Package structure created in $(DIST_DIR)/linux/"
	@echo "Linux distribution created: $(DIST_DIR)/fractal-oxide_$(APP_VERSION)_amd64.deb"

# Windows installer distribution
dist-windows: build dist-clean
	@echo "Building Windows distribution..."
	mkdir -p $(DIST_DIR)/windows
	cp target/release/fractal-oxide.exe $(DIST_DIR)/windows/FractalOxide.exe
	cp LICENSE $(DIST_DIR)/windows/LICENSE.txt
	cp README.md $(DIST_DIR)/windows/README.txt
	@echo "Creating Windows batch installer script..."
	@echo '@echo off' > $(DIST_DIR)/windows/install.bat
	@echo 'echo Installing Fractal Oxide...' >> $(DIST_DIR)/windows/install.bat
	@echo 'set "INSTALL_DIR=%ProgramFiles%\FractalOxide"' >> $(DIST_DIR)/windows/install.bat
	@echo 'mkdir "%INSTALL_DIR%" 2>nul' >> $(DIST_DIR)/windows/install.bat
	@echo 'copy /Y FractalOxide.exe "%INSTALL_DIR%\"' >> $(DIST_DIR)/windows/install.bat
	@echo 'copy /Y LICENSE.txt "%INSTALL_DIR%\"' >> $(DIST_DIR)/windows/install.bat
	@echo 'copy /Y README.txt "%INSTALL_DIR%\"' >> $(DIST_DIR)/windows/install.bat
	@echo 'echo Installation complete!' >> $(DIST_DIR)/windows/install.bat
	@echo 'echo To run: "%INSTALL_DIR%\FractalOxide.exe"' >> $(DIST_DIR)/windows/install.bat
	@echo 'pause' >> $(DIST_DIR)/windows/install.bat
	@echo "Creating ZIP archive..."
	@which zip > /dev/null 2>&1 && cd $(DIST_DIR)/windows && zip -r ../FractalOxide-$(APP_VERSION)-windows.zip . || echo "Note: zip not available. Files ready in $(DIST_DIR)/windows/"
	@echo "Windows distribution created: $(DIST_DIR)/FractalOxide-$(APP_VERSION)-windows.zip"

# Build all distributions
dist: dist-mac dist-linux dist-windows
	@echo "All distributions created in $(DIST_DIR)/"

# Help
help:
	@echo "Available targets:"
	@echo "  build          - Build release version"
	@echo "  build-debug    - Build debug version"
	@echo "  run            - Run release version"
	@echo "  run-debug      - Run debug version"
	@echo "  clean          - Clean build artifacts"
	@echo "  test           - Run tests"
	@echo "  test-show      - Run tests with output"
	@echo "  fmt            - Format code"
	@echo "  lint           - Run clippy lints"
	@echo "  deps           - Update dependencies"
	@echo "  install        - Build and show binary location"
	@echo "  dist-clean     - Clean distribution directory"
	@echo "  dist-mac       - Create macOS .dmg package"
	@echo "  dist-linux     - Create Linux .deb package"
	@echo "  dist-windows   - Create Windows installer package"
	@echo "  dist           - Create all distribution packages"
	@echo "  help           - Show this help"
