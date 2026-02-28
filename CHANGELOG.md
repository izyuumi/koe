# Changelog

## [0.2.0](https://github.com/izyuumi/koe/compare/v0.1.2...v0.2.0) (2026-02-25)


### Features

* fix panics, dynamic mic indicator, duplicate process guard, escape dismiss ([#36](https://github.com/izyuumi/koe/issues/36)) ([d47bb55](https://github.com/izyuumi/koe/commit/d47bb55167a34932ac27f6b758252191d631628f))
* **koe:** auto-copy transcript to clipboard on dictation end ([#45](https://github.com/izyuumi/koe/issues/45)) ([446b458](https://github.com/izyuumi/koe/commit/446b4580f25426ae28e340990cea33c96c5b6208))
* **koe:** add fn/Globe key shortcut to toggle transcription (macOS 15+) ([#47](https://github.com/izyuumi/koe/issues/47)) ([5024f6f](https://github.com/izyuumi/koe/commit/5024f6f174100b9a66f4e1335dd8387742439f58))


### Bug Fixes

* Trim transcript whitespace and add trailing space for chaining ([#37](https://github.com/izyuumi/koe/issues/37)) ([7116a86](https://github.com/izyuumi/koe/commit/7116a863526b9f22f03f5a0ab67499470974b7da))

## [0.1.2](https://github.com/izyuumi/koe/compare/v0.1.1...v0.1.2) (2026-02-15)


### Bug Fixes

* **ci:** skip lipo on sidecar binary (already universal) ([2d5eb69](https://github.com/izyuumi/koe/commit/2d5eb697c192acaa766277463e3c266be28daee1))

## [0.1.1](https://github.com/izyuumi/koe/compare/v0.1.0...v0.1.1) (2026-02-15)


### Bug Fixes

* **ci:** use app bundle instead of dmg for universal build ([70c8067](https://github.com/izyuumi/koe/commit/70c8067471528c9d973b0ff3a890b6bc3e005ea4))

## 0.1.0 (2026-02-13)


### Features

* add app icon with mic and 声 kanji ([cb590dd](https://github.com/izyuumi/koe/commit/cb590dde0a4fe224beb57c6e6b9f3e6e23314b40))
* add language toggle, streaming UX, clipboard safety, watchdog, tray icons ([a167fa4](https://github.com/izyuumi/koe/commit/a167fa4006a30b22af733c6a2dda75efcd6989ee))
* enhance onboarding experience and update styles for Koe dictation HUD ([40660da](https://github.com/izyuumi/koe/commit/40660dabbaff4d0286aec88ba9ea32809605f219))


### Bug Fixes

* bundle speech helper as Tauri sidecar binary ([87d39d4](https://github.com/izyuumi/koe/commit/87d39d417cd65e45c851f4513de81bf9a04fa2d5))
* enable image-png feature, add app icons, remove unused imports ([14e83b2](https://github.com/izyuumi/koe/commit/14e83b2cb06e56a8f3d428626a4ca10d7ffec6a4))
* **tauri:** resolve ACL permission config and macOS transparency setup ([98e2ebd](https://github.com/izyuumi/koe/commit/98e2ebd0a44560a27725db9249762068e55a721a))
* **ux:** improve onboarding and dictation HUD flow ([a127284](https://github.com/izyuumi/koe/commit/a127284be101177411f4b3693be276acc0950fc3))


### Miscellaneous Chores

* release 0.1.0 ([70bcae6](https://github.com/izyuumi/koe/commit/70bcae6b88a27eb7680c57ded4422974c03c8092))
