extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate butlerd;
extern crate dirs;
use butlerd::Butler;
use config::*;
extern crate serde_json;
use std::fs::{DirEntry, OpenOptions};
use std::io::{prelude::*, Read};
use std::process::Command;
use std::{env, fmt, fs, io};
/// Represents a game registered in eidolon
#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub command: String,
    pub pname: String,
    pub name: String,
    pub typeg: games::GameType,
}

/// Module for working directly with the game registry
pub mod games {
    use self::GameType::*;
    use crate::{helper::*, *};
    /// An Enum for the different types of games Eidolon can support
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum GameType {
        Itch,
        Steam,
        Lutris,
        Exe,
        Dolphin,
        WyvernGOG,
    }
    impl fmt::Display for GameType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
    /// Checks game registry for old-formatted games, and attempts to convert them
    pub fn check_games() {
        let games = get_games();
        for game in games {
            let m = fs::metadata(gd() + &game);
            if m.is_ok() {
                if m.unwrap().is_dir() {
                    let mut command = String::new();
                    fs::File::open(gd() + &game + "/start")
                        .unwrap()
                        .read_to_string(&mut command)
                        .unwrap();
                    let mut commandl = command.lines();
                    commandl.next().unwrap();
                    let mut command = commandl.next().unwrap().to_string();
                    let mut typeg = Exe;
                    if command.contains("steam://rungameid") {
                        typeg = Steam;
                    } else if command.contains("lutris:rungameid") {
                        typeg = Lutris;
                    }
                    let mut games = Game {
                        name: game.clone(),
                        pname: game.clone(),
                        command: command,
                        typeg: typeg,
                    };
                    add_game(games);
                    println!("Converted {}", game);
                    fs::remove_dir_all(gd() + &game).unwrap();
                }
            }
        }
    }
    /// Adds a given and configured game to registry
    pub fn add_game(game: Game) {
        if fs::metadata(gd() + &game.name + ".json").is_ok() {
            println!("  Already made a shortcut for {}", game.pname);
        } else {
            let mut ok = true;
            let blocked = get_config().blocked;
            for block in blocked {
                if game.name == block {
                    ok = false;
                }
            }
            if ok {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(gd() + &game.name + ".json")
                    .unwrap()
                    .write_all(serde_json::to_string(&game).unwrap().as_bytes())
                    .unwrap();
                println!("  Made shortcut for {}", game.pname);
            } else {
                println!("  {} is in your blocked list", game.pname);
            }
        }
    }
    /// Loads vec of all installed games
    pub fn get_games() -> Vec<String> {
        fs::read_dir(gd())
            .expect("Can't read in games")
            .collect::<Vec<io::Result<DirEntry>>>()
            .into_iter()
            .map(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .replace(".json", "")
            })
            .collect::<Vec<String>>()
    }
    /// Prints currently installed games
    pub fn list_games() {
        println!("Currently registered games:");
        let entries = get_games();
        println!("Name - Procname - Type");
        for entry in entries {
            let game = read_game(entry).unwrap();
            println!("{} - {} - {}", game.pname, game.name, game.typeg);
        }
    }
    /// Runs a registered game, given name
    pub fn run_game<N>(name: N) -> Result<String, String>
    where
        N: Into<String>,
    {
        let proced = create_procname(name.into());
        let g = read_game(proced);
        if g.is_ok() {
            let g = g.unwrap();
            match g.typeg {
                Itch => {
                    let butler = Butler::new().expect("Has butler been uninstalled?");
                    butler.launch_game(&g.command);
                    return Ok("Launched through butler".to_string());
                }
                Dolphin => {
                    let result = Command::new("dolphin-emu-cli")
                        .arg(g.command)
                        .output()
                        .expect("Couldn't run dolphin game");
                    if !result.status.success() {
                        return Err(String::from_utf8_lossy(&result.stderr)
                            .into_owned()
                            .to_string());
                    } else {
                        return Ok(String::from_utf8_lossy(&result.stdout)
                            .into_owned()
                            .to_string());
                    }
                }
                WyvernGOG => {
                    let path = std::path::PathBuf::from(&g.command);
                    let start = path.join(std::path::PathBuf::from("start.sh"));
                    let result = Command::new(start.to_str().unwrap())
                        .output()
                        .expect("Couldn't run GOG game!");
                    if !result.status.success() {
                        return Err(String::from_utf8_lossy(&result.stderr)
                            .into_owned()
                            .to_string());
                    } else {
                        return Ok(String::from_utf8_lossy(&result.stdout)
                            .into_owned()
                            .to_string());
                    }
                }
                _ => {
                    let result = Command::new("sh")
                        .arg("-c")
                        .arg(g.command)
                        .output()
                        .expect("Couldn't run selected game!");
                    if !result.status.success() {
                        return Err(String::from_utf8_lossy(&result.stderr)
                            .into_owned()
                            .to_string());
                    } else {
                        return Ok(String::from_utf8_lossy(&result.stdout)
                            .into_owned()
                            .to_string());
                    }
                }
            }
        } else {
            println!("Couldn't find that game installed. Maybe you misspelled something?");
            Err("Nonexistent".to_string())
        }
    }
    /// Removes folder of named game
    pub fn rm_game<N>(name: N)
    where
        N: Into<String>,
    {
        let res = fs::remove_file(String::from(gd() + create_procname(name).as_ref()) + ".json");
        if res.is_ok() {
            println!("Game removed!");
        } else {
            println!("Game did not exist. So, removed?");
        }
    }
    /// Registers executable file as game with given name. Wine argguement indicates whether or not to run this game under wine
    pub fn add_game_p<N>(name: N, exe: N, wine: bool)
    where
        N: Into<String>,
    {
        let (name, exe) = (name.into(), exe.into());
        let mut path = env::current_dir().unwrap();
        path.push(exe.clone());
        //Adds pwd to exe path
        let name = create_procname(name.as_str());
        let pname = name.clone();
        if fs::metadata(gd() + &name + ".json").is_ok() {
            println!("A shortcut has already been made for {}", pname);
        } else {
            println!("Creating shortcut for {:?} with a name of {}", path, name);
            let mut start = String::from("");
            if wine {
                let mut winestr = String::from("wine ");
                if exe.to_lowercase().contains(".lnk") {
                    winestr = winestr + "start ";
                }
                start.push_str(&winestr);
            }
            let command = String::from(
                start
                    + &(path
                        .into_os_string()
                        .into_string()
                        .unwrap()
                        .replace(" ", "\\ ")),
            );
            let game = Game {
                pname: pname.to_string(),
                name: name,
                command: command,
                typeg: Exe,
            };
            add_game(game);
        }
    }

    /// Reads in a game's info from a name
    pub fn read_game<N>(name: N) -> Result<Game, String>
    where
        N: Into<String>,
    {
        let name = name.into();
        if fs::metadata(gd() + &name + ".json").is_ok() {
            let mut stri = String::new();
            fs::File::open(gd() + &name + ".json")
                .unwrap()
                .read_to_string(&mut stri)
                .unwrap();
            let g: Game = serde_json::from_str(&stri).unwrap();
            return Ok(g);
        }
        return Err("No such game".to_string());
    }
}

/// Functions related to automatic scanning and updating of game registry
pub mod auto {
    use self::GameType::*;
    use crate::{games::*, helper::*, *};
    /// A result from searching for steam games
    pub struct SearchResult {
        pub appid: String,
        pub name: String,
        pub outname: String,
    }
    /// Fetches lutris wine games and returns a vec of names and lutris ids as tuples
    pub fn get_lutris() -> Result<Vec<(String, String)>, String> {
        let games = Command::new("lutris").arg("-l").output();
        if games.is_ok() {
            let games = games.unwrap();
            if games.status.success() {
                let games_list = String::from_utf8_lossy(&games.stdout);
                return Ok(games_list
                    .lines()
                    .filter(|x| x.find("wine").is_some())
                    .map(|x| {
                        let n = x.split("|").collect::<Vec<&str>>();
                        (String::from(n[0].trim()), String::from(n[1].trim()))
                    })
                    .collect::<Vec<(String, String)>>());
            } else {
                return Err("Lutris not installed".to_string());
            }
        } else {
            return Err("Lutris not installed".to_string());
        }
    }

    /// Searches itch.io games and adds them to game registry
    pub fn update_itch() {
        if fs::metadata(get_home() + "/.config/itch").is_ok() {
            let btest = Butler::new();
            if btest.is_ok() {
                let mut already = get_games()
                    .iter_mut()
                    .filter(|x| {
                        let read = read_game(x.to_string()).unwrap();
                        read.typeg == Itch
                    })
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                println!(">> Reading in itch.io games");
                let butler = btest.expect("Couldn't start butler daemon");
                let caves = butler.fetchall().expect("Couldn't fetch butler caves");
                for cave in caves {
                    let game = cave.game;
                    let name = game.title;
                    let procname = create_procname(name.as_str());
                    let g = Game {
                        pname: name,
                        name: procname.clone(),
                        command: cave.id,
                        typeg: Itch,
                    };
                    add_game(g);
                    let mut i = 0;
                    while i < already.len() {
                        if already[i] == procname {
                            already.remove(i);
                        }
                        i += 1;
                    }
                }
                for left in already {
                    println!("{} has been uninstalled. Removing from registry.", left);
                    rm_game(left);
                }
            } else {
                println!("Itch.io client not installed!");
            }
        } else {
            println!("Itch.io client not installed!");
        }
    }
    // /Iterates through steam directories for installed steam games and creates registrations for all
    pub fn update_steam(dirs: Vec<String>) {
        let mut already = get_games();
        for x in &dirs {
            println!(">> Reading in steam library {}", &x);
            let name = x.to_owned();
            let entries_try = fs::read_dir(name.clone() + "/common");
            if entries_try.is_ok() {
                let entries = fs::read_dir(x.to_owned() + "/common")
                    .expect("Can't read in games")
                    .into_iter()
                    .map(|x| proc_path(x.unwrap()));
                for entry in entries {
                    //Calls search games to get appid and proper name to make the script
                    let results = search_games(entry, x.to_owned());
                    if results.is_some() {
                        let results = results.unwrap();
                        let procname = create_procname(results.name.as_str());
                        let pname = results.name;
                        let command =
                            String::from("steam 'steam://rungameid/") + &results.appid + "'";
                        let game = Game {
                            name: procname.clone(),
                            pname: pname.clone(),
                            command: command,
                            typeg: Steam,
                        };
                        add_game(game);
                        let mut i = 0;
                        while i < already.len() {
                            if already[i] == procname {
                                already.remove(i);
                            }
                            i += 1;
                        }
                    }
                }
            } else {
                println!(
                    "Directory {} does not exist or is not a valid steam library",
                    name
                );
            }
        }
        for al in already {
            let typeg = read_game(al.clone()).unwrap().typeg;
            if typeg == Steam {
                println!("{} has been uninstalled. Removing game from registry.", al);
                rm_game(al);
            }
        }
    }
    /// Adds lutris wine games from get_lutris
    pub fn update_lutris() {
        let lut = get_lutris();
        if lut.is_err() {
            println!(">> No wine games found in lutris, or lutris not installed");
        } else {
            println!(">> Reading in lutris wine games");
            for game in lut.unwrap() {
                let procname = create_procname(game.1.as_str());
                let pname = game.1.clone();
                let command = String::from("lutris lutris:rungameid/") + &game.0;
                let g = Game {
                    pname: pname,
                    name: procname,
                    command: command,
                    typeg: Lutris,
                };
                add_game(g);
            }
        }
    }
    /// Searches given steam game directory for installed game with a directory name of [rawname]
    pub fn search_games<N>(rawname: N, steamdir: N) -> Option<SearchResult>
    where
        N: Into<String>,
    {
        let (rawname, steamdir) = (rawname.into(), steamdir.into());
        let entries = fs::read_dir(steamdir).expect("Can't read installed steam games");
        for entry in entries {
            let entry = entry.unwrap().path();
            let new_entry = entry.into_os_string().into_string().unwrap();
            if new_entry.find("appmanifest").is_some() {
                let mut f = fs::File::open(&new_entry).expect("Couldn't open appmanifest");
                let mut contents = String::new();
                f.read_to_string(&mut contents)
                    .expect("Could not read appmanifest");
                if contents.find("installdir").is_some() {
                    //Slices out the game data from the appmanifest. Will break the instant steam changes appmanifest formats
                    let outname = contents
                        .get(
                            contents
                                .find("installdir")
                                .expect("Couldn't find install dir")
                                + 14
                                ..contents.find("LastUpdated").unwrap() - 4,
                        )
                        .unwrap();
                    if outname == rawname {
                        let appid = contents
                            .get(
                                contents.find("appid").unwrap() + 9
                                    ..contents.find("Universe").unwrap() - 4,
                            )
                            .unwrap();
                        let name = contents
                            .get(
                                contents.find("name").unwrap() + 8
                                    ..contents.find("StateFlags").unwrap() - 4,
                            )
                            .unwrap();
                        return Some(SearchResult {
                            appid: String::from(appid),
                            name: String::from(name),
                            outname: String::from(outname),
                        });
                    }
                }
            }
        }
        //Return none if nothing can be found
        return None;
    }
    /// Iterates through directory and imports each child directory
    pub fn imports<N>(dir: N)
    where
        N: Into<String>,
    {
        let dir = dir.into();
        let entries = fs::read_dir(&dir).expect("Can't read in folder of games");
        println!("Reading in directory: {}", dir);
        for entry in entries {
            let entry = proc_path(entry.unwrap());
            println!("Attempting import on {}", &entry);
            import(entry.as_str());
            println!("Finished attempted import on {}", &entry);
        }
    }
    /// Scans a directory for common game formats and adds them.
    pub fn import<N>(dir: N)
    where
        N: Into<String>,
    {
        let dir = dir.into();
        let mut path = env::current_dir().unwrap();
        let entry_format = &dir.split("/").collect::<Vec<&str>>();
        let real_dir: String = String::from(entry_format[entry_format.len() - 1]);
        let procname = create_procname(real_dir);
        println!("Creating game registry named {}.", procname);
        path.push(dir.clone());
        let entries = fs::read_dir(&path).expect("Can't read in game folder");
        let mut found_game = String::new();
        for entry in entries {
            let entry = proc_path(entry.unwrap());
            let mut found = true;
            if entry.find(".x86_64").is_some() {
                println!("Found a unity exe. Assuming it is game");
            } else if entry.find("start.sh").is_some() {
                println!("Found a GOG game. Assuming it is game");
            } else if entry.find("start").is_some() {
                println!("Found older nicohman game exe. Assuming it is game");
            } else {
                found = false;
            }
            if found == true {
                found_game = entry.to_owned();
            }
        }
        if found_game.len() > 0 {
            add_game_p(
                procname,
                path.into_os_string().into_string().unwrap() + "/" + &found_game,
                false,
            );
        } else {
            println!(
                "Could not find recognizable game exe. You will have to manually specify using eidolon add [name] [exe]"
            );
        }
    }
}
/// Functions for working with the config file/formats
pub mod config {
    use crate::{helper::*, *};
    use regex::Regex;
    /// Eidolon's user config
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Config {
        pub steam_dirs: Vec<String>,
        pub menu_command: String,
        pub prefix_command: String,
        #[serde(default = "default_blocked")]
        pub blocked: Vec<String>,
    }
    impl Config {
        /// Default config
        fn default() -> Config {
            Config {
                steam_dirs: vec!["$HOME/.local/share/Steam/steamapps".to_string()],
                menu_command: "rofi -theme sidebar -mesg 'eidolon game:' -p '> ' -dmenu"
                    .to_string(),
                prefix_command: "".to_string(),
                blocked: default_blocked(),
            }
        }
    }
    /// The pre-3.7 config
    pub struct OldConfig {
        pub steam_dirs: Vec<String>,
        pub menu_command: String,
        pub prefix_command: String,
    }
    fn default_blocked() -> Vec<String> {
        vec![
            "steamworks_common_redistributables".to_string(),
            "proton_3.7".to_string(),
            "proton_3.7_beta".to_string(),
        ]
    }
    /// Converts pre-v1.2.7 config to JSON config
    pub fn convert_config() {
        let old = get_config_old();
        let conf = Config {
            steam_dirs: old
                .steam_dirs
                .into_iter()
                .map(|x| String::from(x))
                .collect::<Vec<String>>(),
            menu_command: String::from(old.menu_command),
            prefix_command: String::from(old.prefix_command),
            blocked: default_blocked(),
        };
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(get_home() + "/.config/eidolon/config.json")
            .unwrap()
            .write_all(serde_json::to_string(&conf).unwrap().as_bytes())
            .unwrap();
        fs::remove_file(get_home() + "/.config/eidolon/config").unwrap();
    }
    /// Loads in eidolon config file
    pub fn get_config() -> Config {
        let mut conf_s = String::new();
        fs::File::open(get_home() + "/.config/eidolon/config.json")
            .expect("Couldn't read config")
            .read_to_string(&mut conf_s)
            .unwrap();
        let mut config: Config = serde_json::from_str(&conf_s).unwrap();
        let fixed = config.steam_dirs.into_iter();
        config.steam_dirs = fixed
            .map(|x| {
                String::from(
                    x.as_str()
                        .replace("$HOME", &get_home())
                        .replace("~", &get_home()),
                )
            })
            .collect::<Vec<String>>();
        config
    }
    /// This parses the config format that eidolon used prior to v1.2.7. This is used to convert the old format into the new JSON-based format when it is detected.
    pub fn get_config_old() -> OldConfig {
        let mut conf = String::new();
        fs::File::open(get_home() + "/.config/eidolon/config")
            .expect("Couldn't read config")
            .read_to_string(&mut conf)
            .expect("Couldn't read in config");
        let mut conf = conf.lines();
        let steam_dirs = conf.next().unwrap();
        let steam_vec = Regex::new(r"(?:([^\|\s]+)\|)")
            .expect("Couldn't create regex")
            .captures_iter(steam_dirs)
            .map(|x| String::from(x.get(1).unwrap().as_str().replace("$HOME", &get_home())))
            .collect::<Vec<String>>();
        let menu_command_base = String::from(conf.next().unwrap());
        let prefix_command_bool = conf.next();
        let mut prefix_command: &str;
        if prefix_command_bool.is_some() {
            prefix_command = prefix_command_bool.unwrap();
            prefix_command = prefix_command.split('|').collect::<Vec<&str>>()[1];
        } else {
            prefix_command = " "
        }
        let menu_command = menu_command_base.split('|').collect::<Vec<&str>>()[1];
        OldConfig {
            steam_dirs: steam_vec,
            menu_command: String::from(menu_command),
            prefix_command: String::from(prefix_command),
        }
    }
    /// Intializes basic directories and config for the first use
    pub fn init() {
        println!("Beginning config init");
        if fs::metadata(get_home() + "/.config").is_err() {
            fs::create_dir(get_home() + "/.config").expect("Couldn't create config directory");
        }
        fs::create_dir(get_home() + "/.config/eidolon").expect("Couldn't create eidolon directory");
        fs::create_dir(get_home() + "/.config/eidolon/games")
            .expect("Couldn't create games directory");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(get_home() + "/.config/eidolon/config.json")
            .unwrap();
        file.write_all(
            serde_json::to_string(&Config::default())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        println!("Correctly initialized base config.");
    }
    /// Checks if eidolon has been inited. If it hasn't, tries to init and returns false if that fails.
    pub fn startup() -> bool {
        if check_inited() {
            true
        } else {
            init();
            check_inited()
        }
    }
    /// Check if eidolon has been initialized prior to this run
    pub fn check_inited() -> bool {
        if fs::metadata(get_home() + "/.config/eidolon").is_err() || fs::metadata(gd()).is_err() {
            false
        } else {
            if fs::metadata(get_home() + "/.config/eidolon/config").is_ok() {
                convert_config();
                true
            } else if fs::metadata(get_home() + "/.config/eidolon/config.json").is_ok() {
                true
            } else {
                false
            }
        }
    }
    /// Returns the eidolon game directory
    pub fn gd() -> String {
        return get_home() + "/.config/eidolon/games/";
    }
}
/// A set of helper functions commonly used by eidolon
pub mod helper {
    use regex::Regex;
    use std::env;
    use std::fs::DirEntry;
    /// Formats game name into nice-looking underscored name for continuity with other names
    pub fn create_procname<N>(rawname: N) -> String
    where
        N: Into<String>,
    {
        let mut basename = String::from(rawname.into()).to_lowercase();
        basename = String::from(basename.trim());
        let reg_white = Regex::new(r"-|\s").unwrap();
        let reg_special = Regex::new(r"'|™|:").unwrap();
        let white_formatted = reg_white.replace_all(&basename, "_");
        let total_formatted = reg_special.replace_all(&white_formatted, "");
        return String::from(total_formatted);
    }

    /// Converts DirEntry into a fully processed file/directory name
    pub fn proc_path(path: DirEntry) -> String {
        let base = path.file_name().into_string().unwrap();
        return base;
    }
    /// Gets current user's home directory
    pub fn get_home() -> String {
        return String::from(dirs::home_dir().unwrap().to_str().unwrap());
    }
}
