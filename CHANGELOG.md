# Changelog

## [3.15.2](https://github.com/hairbui76/glazewm/compare/v3.15.1...v3.15.2) (2026-10-05)


### Bug fixes

* **installer:** keep autostart enabled across updates ([14e4fab](https://github.com/hairbui76/glazewm/commit/14e4fabaa88f882abb0c61b97abb56edfd9160c5))
* resolve the latest release without the GitHub API ([62f0294](https://github.com/hairbui76/glazewm/commit/62f0294f9676d61739bf32d6f65720b48541331e))
* start at sign-in through a scheduled task ([c1f777a](https://github.com/hairbui76/glazewm/commit/c1f777a78aea60a8207e2fdb3a95602f42c97b64))

## [3.15.1](https://github.com/hairbui76/glazewm/compare/v3.15.0...v3.15.1) (2026-09-25)


### Bug fixes

* drop empty workspaces when rearranging ([86ee8c1](https://github.com/hairbui76/glazewm/commit/86ee8c1e5d65eb17488c98f88f0c4e2d2123c0ea))

## [3.15.0](https://github.com/hairbui76/glazewm/compare/v3.14.0...v3.15.0) (2026-09-25)


### Features

* show keyboard shortcuts in the tray menu ([8a15803](https://github.com/hairbui76/glazewm/commit/8a1580352e5bedf3c76e54f892c3fac907718a3a))

## [3.14.0](https://github.com/hairbui76/glazewm/compare/v3.13.1...v3.14.0) (2026-09-24)


### Features

* add a command to renumber workspaces without gaps ([14af98b](https://github.com/hairbui76/glazewm/commit/14af98b4be3061336bd160ac4a89661c8e121ba8))

## [3.13.1](https://github.com/hairbui76/glazewm/compare/v3.13.0...v3.13.1) (2026-09-16)


### Bug fixes

* stop bouncing windows that keep putting themselves back ([2cb4c8a](https://github.com/hairbui76/glazewm/commit/2cb4c8a3cbe6796328a20aff0e1abb56cd009411))

## [3.13.0](https://github.com/hairbui76/glazewm/compare/v3.12.0...v3.13.0) (2026-09-16)


### Features

* show the version at the top of the tray menu ([c2861c5](https://github.com/hairbui76/glazewm/commit/c2861c5f47a172fc3618e1a393e42b4488484700))


### Bug fixes

* rebuild crates when the version number changes ([d8732a8](https://github.com/hairbui76/glazewm/commit/d8732a8c8f7ebcc10359f75a49e6ba021b000839))

## [3.12.0](https://github.com/hairbui76/glazewm/compare/v3.11.2...v3.12.0) (2026-09-16)


### Features

* stop bundling Zebar in the installer ([5cd15f6](https://github.com/hairbui76/glazewm/commit/5cd15f6d501097cae83cd9ec7dbdf0c0cf5726b5))


### Bug fixes

* relaunch the installed executable after updating ([7b345a7](https://github.com/hairbui76/glazewm/commit/7b345a73adbbd7425765626f2252a4b61f344a1b))

## [3.11.2](https://github.com/hairbui76/glazewm/compare/v3.11.1...v3.11.2) (2026-09-16)


### Bug fixes

* **ci:** authenticate the Zebar release lookup ([e61d06d](https://github.com/hairbui76/glazewm/commit/e61d06d630f7f15ac120ab52be400424acf93e02))

## [3.11.1](https://github.com/hairbui76/glazewm/compare/v3.11.0...v3.11.1) (2026-09-16)


### Bug fixes

* **ci:** stop requesting UIAccess in release builds ([e2d2518](https://github.com/hairbui76/glazewm/commit/e2d25180f2355e9bbd19b2dee90f862a818e4b9a))

## [3.11.0](https://github.com/hairbui76/glazewm/compare/v3.10.1...v3.11.0) (2026-09-16)


### Features

* add "Check for updates" to the system tray ([51f22d9](https://github.com/hairbui76/glazewm/commit/51f22d970b843e8704c064a64270619b3f8c21dd))
* add ignore rule for CleanShot X to sample config ([#1343](https://github.com/hairbui76/glazewm/issues/1343)) ([9d94e12](https://github.com/hairbui76/glazewm/commit/9d94e125fb7b520559227bf1f4ffa09802d520d0))
* add one-line install script for Windows ([befbe98](https://github.com/hairbui76/glazewm/commit/befbe9820d5fe4e136f617ad971dac27eb6d3de1))
* add reciprocal keybind to sample config for exiting resize mode ([#1344](https://github.com/hairbui76/glazewm/issues/1344)) ([efd0efa](https://github.com/hairbui76/glazewm/commit/efd0efaa8d58d832ac9a7a6ba63968e91d2f9702))
* Minimized windows to workspaces when startup ([aae47df](https://github.com/hairbui76/glazewm/commit/aae47dfaebe2189a91b607022c4fe2e51cd4df85))
* Support toggle for workspaces in single monitor only ([801f698](https://github.com/hairbui76/glazewm/commit/801f6984f0ca131c278719b80c72b2e4f398520c))


### Bug fixes

* capture hide events outside of the visible workspaces on macOS ([#1359](https://github.com/hairbui76/glazewm/issues/1359)) ([dce8d14](https://github.com/hairbui76/glazewm/commit/dce8d143143c4a9268347016a1fdf03552c99b2f))
* Change primary display name to hardwareId filter ([2c32b0c](https://github.com/hairbui76/glazewm/commit/2c32b0cbf307edc55ffadcf98d60060be34b4f55))
* **ci:** publish releases directly and skip signing without secrets ([6263fd1](https://github.com/hairbui76/glazewm/commit/6263fd1bef6ed77c69aa61bcdc6decb165de46ec))
* clean up stale entries from ignored windows list ([#1312](https://github.com/hairbui76/glazewm/issues/1312)) ([ddf7bc1](https://github.com/hairbui76/glazewm/commit/ddf7bc19b6c7c692ff388ea2bdff3526efd22a7e))
* enforce single-monitor workspace placement across all entry points ([1b68c27](https://github.com/hairbui76/glazewm/commit/1b68c2793a1baa73fab70f004e5d80b2a2392373))
* Prevent monitor-to-monitor window flicker on display change ([54654a3](https://github.com/hairbui76/glazewm/commit/54654a3ffb34da713868182e7298e4ec57b59de4))
* Refine window placement on unmanaged monitors ([41e466e](https://github.com/hairbui76/glazewm/commit/41e466ed8c02273d5d7b7f8ff1179604bdfd145b))
* segfault on focus with macOS sonoma 14.5 ([#1340](https://github.com/hairbui76/glazewm/issues/1340)) ([919f4c9](https://github.com/hairbui76/glazewm/commit/919f4c93e32f7c93e66a745f9ab8e434dac968a0))


### Refactors

* remove macOS support ([6ed9dd7](https://github.com/hairbui76/glazewm/commit/6ed9dd7ade09caa63e6c86a74dc470dee84b5e6f))
