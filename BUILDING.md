# Building Celeste
This document contains the instructions needed to build Celeste from source.

If you're just trying to install Celeste, look at the [installation instructions](/README.md#installation) instead.

## Needed packages
Celeste needs some packages installed in order to build:

- [rustup](https://rustup.rs/)
- [Go](https://go.dev/)
- [just](https://github.com/casey/just)
- Headers for GTK4 and Libadwaita (The best way I've found to find the needed packages is to try building as described below, and then looking at the output to see what the missing header files are. You can then install the needed packages via your package manager. This process is kind of tedious, but I don't know any other methods at the moment).

## Building
To build the project, run the following from the root of the repository:

```sh
just build
```

To install the needed files into a packaging directory (such as how `${pkgdir}` functions in Arch/makedeb PKGBUILDs), run the following from the root of the repository (replacing `{pkgdir}` with the location of your packaging directory):

```sh
DESTDIR='{pkgdir}' just install
```
