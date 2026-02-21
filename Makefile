.PHONY: all dev build helper icons clean lint fmt

all: helper icons
	cd src-tauri && cargo build

dev: helper icons
	npm run tauri dev

build: helper icons
	npm run tauri build

# Compile the Swift speech helper
TRIPLE := $(shell rustc -vV | grep host | awk '{print $$2}')

helper:
	@mkdir -p src-tauri/binaries
	swiftc speech-helper/main.swift \
		-o src-tauri/binaries/koe-speech-helper-$(TRIPLE) \
		-framework Speech \
		-framework AVFoundation \
		-framework Foundation \
		-O
	@# Also copy to legacy location for dev mode fallback
	@cp src-tauri/binaries/koe-speech-helper-$(TRIPLE) src-tauri/koe-speech-helper

# Generate placeholder icons (replace with real ones later)
icons:
	@mkdir -p src-tauri/icons
	@if [ ! -f src-tauri/icons/icon.icns ]; then \
		sips -z 32 32 -s format png /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns --out src-tauri/icons/32x32.png 2>/dev/null || true; \
		sips -z 128 128 -s format png /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns --out src-tauri/icons/128x128.png 2>/dev/null || true; \
		sips -z 256 256 -s format png /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns --out src-tauri/icons/128x128@2x.png 2>/dev/null || true; \
		cp /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns src-tauri/icons/icon.icns 2>/dev/null || true; \
		sips -z 256 256 -s format png /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns --out src-tauri/icons/icon.png 2>/dev/null || true; \
		cp src-tauri/icons/32x32.png src-tauri/icons/tray.png 2>/dev/null || true; \
	fi

lint:
	cd src-tauri && cargo clippy -- -D warnings
	npx tsc --noEmit

fmt:
	cd src-tauri && cargo fmt --check
	@echo "Rust formatting OK"

clean:
	rm -f src-tauri/koe-speech-helper
	cd src-tauri && cargo clean
	rm -rf dist
