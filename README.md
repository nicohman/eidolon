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

wine_add [name] [.exe] : adds windows exe to be run under wine to the registry

help : show this screen

```
## Configuration
Right now, only two config options exist: menu_command and steam_dirs.

`menu_command` : The command to be run to display the eidolon menu. Will be given an alphabetical list of names through STDIN, and a name is expected back through STDOUT.

`steam_dirs` : a |-seperated list of steam install directories, with $HOME replacing the home directory.

#### Default config file:
```
steam_dirs: |$HOME/.local/share/steam/steamapps|

menu_command: | rofi -theme sidebar -mesg 'eidolon game:' -p '> ' -dmenu |
```
## Todo

+ Convert procname to use regex
+ Add in support for importation of wine games from lutris, preferably without actually using lutris launch links
+ Please suggest any other features you want as an issue!

