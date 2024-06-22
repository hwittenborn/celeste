# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.3] - 2024-06-22
### Fixed
- Fix parsing error in MetaInfo file.

## [0.8.2] - 2024-06-21
### Fixed
- Update dependencies to fix build error.

## [0.8.1] - 2023-11-18
### Changed
- Added more keywords to Celeste's desktop entry.

### Fixed
- Fixed Google Drive authentication window not showing when using older rclone versions.

## [0.8.0] - 2023-10-23
### Changed
- Added functionality to only run sync checks when files are changed.

## [0.7.0] - 2023-10-07
### Changed
- Remove reliance on GTK3 and libappindicator.
- Make tray functionality part of main application, instead of being a separate app.
- Combine all application functionality into a singular crate.

### Fixed
- Fixed incorrect number of errors being reported in tray.
- Fixed bug where tray icons would never change.

## [0.6.0] - 2023-10-05
### Added
- Added Proton Drive support.

### Fixed
- Fixed missing `description` tags in metainfo's `releases` section.

## [0.5.8] - 2023-09-16
### Added
- Added release notes to about page.

## [0.5.7] - 2023-09-15
### Fixed
- Removed duplicate releases in AppStream metadata file.

## [0.5.6] - 2023-09-14
### Changed
- Changed to new application icons.
- Added credits to about page.

## [0.5.5] - 2023-08-09
### Fixed
- Update `Cargo.toml` and `Cargo.lock` to fix `arm64` compile error.

## [0.5.4] - 2023-07-24
### Fixed
- Updated `Cargo.lock` to fix compile error.

## [0.5.3] - 2023-06-07
### Fixed
- Updated `Cargo.lock` to fix compile error.

## [0.5.2] - 2023-03-27
### Fixed
- Fixed more issues in automated Flathub packaging.

## [0.5.1] - 2023-03-27
### Fixed
- Fixed automated Flathub packaging.

## [0.5.0] - 2023-03-27
### Added
- Added support for translations.

### Changed
- Fixed loading times when adding new sync directories.
- Made autocompletions in sync directory additions more dynamic.

### Fixed
- Fixed loading times when adding remotes with high storage usage.
- Fixed freeze when ports needed by `rclone authorize` are already being used.

## [0.4.6] - 2023-02-24
### Fixed
- Fixed HTML elements in metainfo release descriptions.

## [0.4.5] - 2023-02-24
### Fixed
- Fixed location of metainfo file.

## [0.4.4] - 2023-02-23
### Fixed
- Add `com.hunterwittenborn.Celeste.metainfo.xml` to packaging.
- Fixed Google Drive authentication for Google's branding requirements.

## [0.4.3] - 2023-02-23
### Fixed
- Removed commented line in `justfile`.

## [0.4.2] - 2023-02-22
### Changed
- Fixed panic when previously synced local dir no longer exists.
- Fixed extra padding in certain windows.
- Fixed recorded remote items incorrectly starting with `/` in Celeste's database.
- Fixed long windows when multiple errors are present.

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
