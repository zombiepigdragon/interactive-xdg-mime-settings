# interactive-xdg-mime-settings

This is a tool to quickly set [XDG MIME](https://wiki.archlinux.org/index.php/XDG_MIME_Applications) associations
interactively. It should be noted that there are *many* reasons why you probably don't want to use this: See [Problems](#Problems)
below.

## Features
- Calls `xdg-mime default` for all of the MIME types that any installed application provides a way to open.
- Asks for a resolution if there is more than one application for a given MIME type, but automagically assigns the
  application for any types are only provided by one application.

## Problems
- Doesn't actually read the `.desktop` files in detail, it just spits out a single key from the file.
- Doesn't use the Shared MIME database, so it can easily miss programs (which manifests in setting defaults to programs
  that you probably don't want.)
- A thin wrapper around a command line utility which can break desktop environments.

## Installation
Building or installing this tool requires `cargo` to be installed alongside a recent Rust toolchain.
1. Clone or checkout this repository
2. `cd` into the directory the repository is unpacked into.
3. Run `cargo install --path .` to place this into the local path.

## License
This is provided under the terms of the MIT License, and any contributions you make will be licensed likewise. See
[LICENSE](/LICENSE) for more information.
