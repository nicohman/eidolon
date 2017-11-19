use std::process::Command;
use std::fs;
use std::io::Read;
fn main() {
    let results = search_games(String::from("Oxenfree"));
	println!("{}{}{}", results.0, results.1, results.2);
}
fn search_games(rawname: String) -> (String, String, String){
	let entries = fs::read_dir("/home/nicohman/steam_games/steamapps/steamapps").expect("cant read");
    for entry in entries {
        let entry = entry.expect("unable to get entry").path();
        let new_entry = entry.into_os_string().into_string().expect("turn to string");
        //println!("{:?}", new_entry);
        if new_entry.find("appmanifest").is_some() {
            //println!("APP"); 
            let mut f = fs::File::open(&new_entry).expect("no file");
            let mut contents = String::new();
            f.read_to_string(&mut contents).expect("wrong");
            //println!("Text:\n{} {}", contents, new_entry);
            unsafe {
                let outname = contents.slice_unchecked(contents.find("installdir").expect("pos1") + 14, contents.find("LastUpdated").unwrap() - 4);
                let appid = contents.slice_unchecked(contents.find("appid").unwrap()+9, contents.find("Universe").unwrap()-4);
                let name = contents.slice_unchecked(contents.find("name").unwrap()+8, contents.find("StateFlags").unwrap()-4);
                //println!("OUTNAME:{} APPID:{} NAME:{}", &outname, &appid, &name);
                if outname == rawname {
                	return (String::from(appid),String::from(name), String::from(outname));
                }
            }
        }
    }
   return (String::from("appid"),String::from("name"),String::from("outname")) 
}
