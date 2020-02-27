// Bring in external crates fs_extra and twox_hash
// More will probably be needed as I disocver which are necessary
extern crate fs_extra;
extern crate twox_hash;
extern crate chrono;
extern crate regex;
extern crate glob;
extern crate systemstat;
// Bring in standard components
#[allow(unused_imports)]
use std::{io, collections::HashMap, path::{Path, PathBuf}};
use glob::glob;

// Bring in fs_extra components
use fs_extra::{dir::*, error::*};

// Bring in chrono components
use chrono::{DateTime, Local};

// Bring in twox_hash components
#[allow(unused_imports)]
use twox_hash::XxHash64;

// Bring in regex components
use regex::Regex;

// Bring in systemstat
use systemstat::{System, Filesystem, Platform};
use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::File;
use std::io::{BufReader, Read};

/* Test hashing function
fn hash_test() {
    
} */
#[allow(dead_code)]
struct FolderChoice {
    source: String,
    dest: String,
    date_time: String,
}

// Get current date and time, local to host machine
// Formatted to be safe for use as folder name as: YEAR-MONTH-DAYTHOURSMINUTES
// ex: 2020-02-06T1600 (Feb 6th 2020, 4PM)
fn current_date_time() -> std::string::String {
    let now: DateTime<Local> = Local::now();
    let now = now.format("%Y-%m-%dT%H%M").to_string();
    now
}

// This is super kludgey, I want this to error out if the match fails
// Currently matches file/folder structure for windows and linux
fn folder_picker() -> std::string::String {
    let mut folder = String::new();
    
    let fail = "FAILED";

    io::stdin()
            .read_line(&mut folder)
            .expect("Failed to read line");

    let trimmed = folder.trim();

    let test: bool = check_folder_valid(trimmed);

    if test {
        //println!("DEBUG Folder choice: {}", trimmed.to_string());
        return trimmed.to_string();
    } else {
        //println!("DEBUG Folder choice: {} is invalid", trimmed.to_string());
        return fail.to_string();
    }
}

// Should probably do the error handling for checking here, still don't know how lol
fn check_folder_valid(folder_input: &str) -> bool {
    let windows_re = Regex::new(r"^[a-zA-Z]:\\[\\\S|*\S]?.*$").unwrap();
    let linux_re = Regex::new(r"^(/[^/ ]*)+/?$").unwrap();
    let test: bool = (windows_re.is_match(folder_input)) || (linux_re.is_match(folder_input));
    test
}

fn copy_files(source_folder: &str, dest_folder: &str, date_time: &str) -> Result<()> {
    let source = Path::new(source_folder);
    let dest = Path::new(dest_folder);
    let mut options = CopyOptions::new();
    options.buffer_size = 128_000_000; //128MB

    assert!(source.exists());
    assert!(dest.exists());

    let sub = dest.join(date_time);
    create_all(&sub, true).expect("Couldn't create files");

    assert!(sub.exists());

    let handle = |process_info: TransitProcess|  {
        println!("{}", process_info.total_bytes);
        fs_extra::dir::TransitProcessResult::ContinueOrAbort
    };
        copy_with_progress(&source, &sub, &options, handle).unwrap();

    Ok(())

}

// This currently only works on windows
fn search_for_files_win(folder: &str, filetype: String) -> Vec<String> {
    //Create new vec of strings
    let mut files = Vec::new();
    //Create new path and push func args together: "DriveRoot\**\*.filetype"
    let mut win_path = PathBuf::new();
    win_path.push(folder);
    win_path.push(r"**\*");
    win_path.set_extension(filetype);

    for entry in glob(&win_path.to_string_lossy()).expect("Failed to read glob") {
        match entry {
            Ok(win_path) => files.push(win_path.display().to_string()),
            Err(e) => println!("{:?}", e),
        }
    }
    files
}

//This currently only works for Linux
fn search_for_files_nux(folder: &str, filetype: String) -> Vec<String> {
    //Create new vec of string
    let mut files = Vec::new();
    //Create new path and push func args together: "/mnt/driveroot/**/*.filetype"
    let mut nux_path = PathBuf::new();
    nux_path.push(folder);
    nux_path.push("**/*");
    nux_path.set_extension(filetype);

    for entry in glob(&nux_path.to_string_lossy()).expect("Failed to read glob") {
        match entry {
            Ok(nux_path) => files.push(nux_path.display().to_string()),
            Err(e) => println!("{:?}", e),
        }
    }
    files
}

// Accepts a vector of UNWRAPPED filesystem mounts, returns string of mount with largest size
fn get_largest_disk(disks: &Vec<Filesystem>) -> Filesystem {
    // get number of elements in vector
    let mount_points = disks.len();

    //initialize to temp value of first element in vector
    let mut largest: Filesystem = disks[0].clone();

    // need to use for loop like this because iter not implemented... or something, idk
    for x in 0..mount_points-1 {
        let current = disks[x].clone();
        let next = disks[x+1].clone();
        largest = if current.total > next.total {
            current
        } else {
            next
        };
    }

    largest
}

// Accepts reference to a System, parses and only returns valid filesystems
// Println about mounts and sizes could probably be commented out
fn enumerate_disks(sys: &systemstat::System) -> Vec<systemstat::Filesystem> {
    let mut return_disks = Vec::new();
    match sys.mounts() {
        Ok(mounts) => {
            for mount in mounts.iter() {
                println!("Mounted on: {}, Total space: {}, Avail space: {}",
                    mount.fs_mounted_on, mount.total, mount.avail);
                return_disks.push(mount.clone());
            }
        }
        Err(x) => println!("\nMounts: error: {}", x)
    }
    return_disks
}

// Super kludgey function to get os type (linux or windows)
fn determine_os(disk: &systemstat::Filesystem) -> String {
    let operating_system: String;
    let windows_re = Regex::new(r"^[a-zA-Z]:\\[\\\S|*\S]?.*$").unwrap();
    if windows_re.is_match(&disk.fs_mounted_on) {
        operating_system = String::from("Windows")
    } else {
        operating_system = String::from("Linux")
    }
    operating_system
}

// Iterates over vector of strings, hashes them and outputs hashmap where
// key = file and value = hash
fn hash_files(files: Vec<String>) -> HashMap<String, String>{
    let mut hashed_files = HashMap::new();
    //let mut file = String::from("");
    //let mut path: String = String::new();
    for file in files.iter(){
        let path = file.clone();
        let input = File::open(&path);
        let reader = BufReader::new(input.unwrap());
        let digest = sha256_digest(reader).unwrap();
        hashed_files.insert(path, HEXUPPER.encode(digest.as_ref()));
    }
    hashed_files
}

// Helper to hash_files
fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn main() {

    let time = current_date_time();

    println!("---DEBUG--- Current date-time stamp for dest folder: {}", time);
    let system = System::new();
    let disks = enumerate_disks(&system);
    let largest = get_largest_disk(&disks);
    let operating_system = determine_os(&largest);
    //println!("OS is: {}", operating_system);
    println!("Recommended target based on disk size is: {}", largest.fs_mounted_on);
    println!("Input a source folder: ");
    let mut source = folder_picker();
    //println!("Input a destination folder: ");
    //let destination = folder_picker();
    let files_to_move: Vec<String>;
    let hashed_files: HashMap<String, String>;
    if operating_system == "Windows"{
        println!("Operating system is: {}", operating_system);
        files_to_move = search_for_files_win(&source, "exe".to_string());
        hashed_files = hash_files(files_to_move);
        for (key, value) in &hashed_files {
            println!("{}: {}", key, value);
        }

    } else {
        println!("Operating system is: {}", operating_system);
        files_to_move = search_for_files_nux(&source, "exe".to_string());
        hashed_files = hash_files(files_to_move);
        for (key, value) in &hashed_files {
            println!("{}: {}", key, value);
        }
    }

    //let mut tempe: String = String::from(r"D:\temp\");
    //let temp = search_for_files_win(&mut tempe, "exe".to_string());
    //let file_hash = hash_files(temp);
    //for (key, value) in &file_hash {
    //    println!("{}: {}", key, value);
    //}

    

    //if source == "FAILED" || destination == "FAILED" {
    //    println!("Invalid source or destination")
    //} else {
    //    copy_files(&source, &destination, &time).expect("Couldn't copy files");
    //}
}