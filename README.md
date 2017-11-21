# eidolon
A conversion of steam_suite to rust with additional features.
Provides a single TUI-based registry for drm-free, wine and steam games on linux, accessed through a rofi launch menu. Simple, fast and lightweight.

## Installation
You'll need [rofi](https://github.com/DaveDavenport/rofi) and [cargo](https://github.com/rust-lang/cargo) installed. Run:

`git clone https://github.com/nicohman/eidolon.git && cd eidolon`

`cargo build --release`

`sudo cp targets/release/eidolon /usr/bin/eidolon`

## Usage
`eidolon help` for list of commands:
```Commands:

update : updates registry with installed steam games

add [name] [file] : adds game to registry

rm [name] : removes game from registry

menu : shows game menu

import [dir] : attempts to import in game directory just from name of location.

imports [dir] : imports in all game directories within given directory

help : show this screen 
```
