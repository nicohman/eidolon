# eidolon
A conversion of steam\_suite to rust with additional features.
Provides a single TUI-based registry for drm-free, wine and steam games on linux, accessed through a rofi launch menu. Simple, fast and lightweight. This is a mirror of the [sr.ht repository](https://git.sr.ht/~nicohman/eidolon). Please file issues for eidolon [here](https://todo.sr.ht/~nicohman/eidolon) and patches [here](https://lists.sr.ht/~nicohman/eidolon), though I will still accept issues and pull requests on github.

### See it in action

![A gif showing eidolon working](https://thumbs.gfycat.com/OrganicGeneralDove-size_restricted.gif)

## Installation
You can now install from [crates.io](https://crates.io/crates/eidolon). Just run `cargo install eidolon` and install [rofi](https://github.com/DaveDavenport/rofi) via your distro's package manager!

You'll need [rofi](https://github.com/DaveDavenport/rofi) and [cargo](https://github.com/rust-lang/cargo) installed. Run:

`git clone https://git.sr.ht/~nicohman/eidolon && cd eidolon`

`cargo install --path . --force`

Alternatively, check [here](https://github.com/nicohman/eidolon/releases) for a possibly out of date binary. In addition, you can download a version built from the latest git commit at [my website](https://demenses.net/downloads)

### Unofficial packages

It appears someone is maintaining a package on the [AUR](https://aur.archlinux.org/packages/eidolon). If anyone else wants to maintain a package somewhere, feel free to! If you tell me about it, I'll even add a link here.

## Usage
`eidolon help` for list of commands:
```
eidolon
nicohman <nicohman@demenses.net>
Provides a single TUI-based registry for drm-free, wine and steam games on linux, accessed through a rofi launch menu.

USAGE:
    eidolon <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add       Adds selected file to registry
    help      Prints this message or the help of the given subcommand(s)
    import    Attempts to import in game directory from dir path
    list      Lists installed games
    menu      Show game menu
    rm        Remove a game from the registry
    run       Runs a game by name
    update    Updates registry with installed steam, lutris wine, and itch games
```

## Configuration
Right now, only three config options exist: menu\_command, prefix\_command and steam\_dirs. The config file is saved in ~/.config/eidolon/config.json, of course in the JSON format.

`menu_command` : The command to be run to display the eidolon menu. Will be given an alphabetical list of names through STDIN, and a name is expected back through STDOUT.

`steam_dirs` : a |-seperated list of steam install directories, with $HOME replacing the home directory.

`prefix_command` : A command that will be run as a prefix to every game command. Good for optirun or steam runtime launching.

#### Default config file:
```
{
steam_dirs: ["$HOME/.local/share/steam/steamapps"],

menu_command: "rofi -theme sidebar -mesg 'eidolon game:' -p '> ' -dmenu",

prefix_command: ""
}
```
## Todo

+ Please suggest any features you want as an issue!
