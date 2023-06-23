# Celeste
<a href="https://flathub.org/apps/details/com.hunterwittenborn.Celeste"><img width="150" src="https://flathub.org/assets/badges/flathub-badge-i-en.svg" /></a>
<a href="https://snapcraft.io/celeste"><img width="150" src="https://snapcraft.io/static/images/badges/en/snap-store-black.svg" /></a>

Celeste is a GUI file synchronization client that can connect to virtually any cloud provider.

- Backed by [rclone](https://rclone.org/), giving you a reliable and battle-tested way to sync your files anywhere
- Written with GTK4 and Libadwaita, giving Celeste a native look and feel on your desktop
- Written in Rust, making Celeste ***blazingly fast*** to use

![](/assets/main-window.png)

> **NOTE:**
> Celeste is currently alpha software, and you should likewise ensure you have a backup of your data before you decide on trying it. *Any file loss incurred is at your own risk*.

## Features
- Two-way sync
- Asking what to do when a local and remote file have both been updated since last sync
- Ability to exclude files/folders from sync
- Connecting to multiple cloud providers at the same time

## Supported cloud providers
Celeste can currently connect to the following cloud providers:
- Dropbox
- Google Drive
- Nextcloud
- Owncloud
- pCloud
- WebDAV

## Installation
Celeste can be installed via the methods listed below:

### Flatpak
Celeste is available on [Flathub](https://flathub.org/apps/details/com.hunterwittenborn.Celeste). First make sure you have [set up Flatpak](https://flatpak.org/setup/) on your system, and then run the following:

```sh
flatpak install flathub com.hunterwittenborn.Celeste
```

### Snap
Celeste is available on the [Snap Store](https://snapcraft.io/celeste), which can be installed on any system that has Snap installed.

```sh
snap install celeste
```

### Prebuilt-MPR (Debian/Ubuntu)
If you're on Ubuntu 22.10 or later, you can install Celeste from the Prebuilt-MPR. First make sure [the Prebuilt-MPR is set up](https://docs.makedeb.org/prebuilt-mpr/getting-started/) on your system, and then run the following:

```sh
sudo apt install celeste
```

## Support
For anything that isn't an issue in Celeste's functionality, visit the project's [Discord server](https://discord.gg/FtNhPepvj7), where you can obtain support and chat with fellow users. If you have a bug/feature request for Celeste, then make a new issue in the project's [issue tracker](https://github.com/hwittenborn/celeste/issues).

## Contributing
Instructions still largely need to be written up - if you'd like to help with that, feel free to submit a PR!

### Translating
Celeste uses [Weblate](https://weblate.org) to manage translations. See <https://hosted.weblate.org/projects/celeste/celeste> if you'd like to assist in translating.
