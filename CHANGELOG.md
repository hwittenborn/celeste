# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Fixed panic when previously synced local dir no longer exists.

## [0.4.1] - 2023-02-15
### Changed
- Removed no `:` requirement in username and password login fields.

## [0.4.0] - 2023-02-08
### Added
- Added support for pCloud.

## [0.3.6] - 2023-02-08
### Changed
- Fixed slash suffixes in local directory causing a crash in sync directory dialog.
- Fixed remote directories not being placed correctly on the local system.

## [0.3.5] - 2023-02-04
### Changed
- Fixed behavior of tray icon closing when main application crashes inside of Flatpak.
- Fixed wordage on a couple CLI warning messages.

## [0.3.4] - 2023-02-02
### Changed
- Closed tray icon when main application crashes.
- Fixed new servers not showing up after all have been removed.
- Fixed crash caused by incorrect server name being registered for Nextcloud and Owncloud servers.

## [0.3.3] - 2023-02-02
### Changed
- Improved titlebars in main application window.

## [0.3.2] - 2023-02-02
### Changed
- Added better error handing when sending DBus messages.

## [0.3.0] - 2023-02-02
### Added
- Added Owncloud storage type.
- Added ability to start up in the background.

## [0.2.0] - 2023-02-02
### Added
- Added Google Drive storage type.

### Changed
- Fixed DBus connection names in Snapcraft config.

## [0.1.12] - 2023-02-01
### Changed
- Fixed file names in `justfile`.

## [0.1.11] - 2023-02-01
### Changed
- Fixed namespace used for symbolic icons and DBus connections.

## [0.1.10] - 2023-02-01
### Changed
- Fixed location of symbolic icons during installation.

## [0.1.9] - 2023-02-01
### Added
- Added Snap packaging

## [0.1.8] - 2023-01-07
### Added
- Added tray icon.

## [0.1.7] - 2022-12-30
### Changed
- Fixed vertical alignment of text in file/folder exclusion section.
- Improved method for finding running Celeste instances.

## [0.1.6] - 2022-12-30
### Changed
- Fixed missing icon on about page.
- Updated progress text to show individual file checks.
- Fixed missing popover button on main screen.
- Fixed main screen window not closing after being reopened.

## [0.1.5] - 2022-12-30
### Changed
- Fixed panic on launch from missing directory.

## [0.1.4] - 2022-12-29
### Changed
- Added very hacky workaround to finally fix linker usage in AppImage.

## [0.1.3] - 2022-12-29
### Changed
- Fixed more linker usage in AppImage.

## [0.1.2] - 2022-12-29
### Changed
- Fixed linker usage and missing dependencies in AppImage.

## [0.1.1] - 2022-12-29
### Changed
- Fixed MPR packaging.

## [0.1.0] - 2022-12-29
First release! 🥳
