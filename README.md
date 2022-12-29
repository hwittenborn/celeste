# Celeste
Celeste is a GUI file synchronization client that can connect to virtually any cloud provider.

- Backed by [rclone](https://rclone.org/), giving you a reliable and battle-tested way to sync your files anywhere
- Written with GTK4 and Libadwaita, giving Celeste a native look and feel on your desktop
- Written in Rust, making Celeste ***blazingly fast*** to use

> **NOTE:**
> Celeste is currently alpha software, and you should likewise ensure you have a backup of your data before you decide on trying it. *Any file loss incurred is at your own risk*.

## Features
- Two-way sync
- Ability to exclude files/folders from sync
- Connecting to multiple cloud providers at the same time

## Supported cloud providers
Celeste can currently connect to the following cloud providers:
- Dropbox
- Nextcloud
- WebDAV

## Installation
Celeste can be installed via the methods listed below:

### MPR Source Package (Debian/Ubuntu)
If you're on Ubuntu 22.10 or later, you can install Celeste from source on the [MPR](https://mpr.makedeb.org/packages/celeste). You'll need to have [makedeb](https://docs.makedeb.org/installing/apt-repository/) and [Mist](https://docs.makedeb.org/using-the-mpr/mist-the-mpr-cli/) installed before you do so.

```sh
mist install celeste
```