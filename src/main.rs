// Bring in external crates fs_extra and twox_hash
// More will probably be needed as I disocver which are necessary

extern crate fs_extra;
extern crate twox_hash;
extern crate chrono;
extern crate regex;

// Bring in standard components
use std::path::Path;
use std::{thread, time};
use std::sync::mpsc::{self, TryRecvError};
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::io;

// Bring in fs_extra components
use fs_extra::dir::*;
use fs_extra::error::*;

// Bring in chrono components
use chrono::{DateTime, Local};

// Bring in twox_hash components
use twox_hash::XxHash64;

// Bring in regex components
use regex::Regex;


fn example_copy() -> Result<()> {
    // Source and destination folders, creates if doesn't exist
    let path_from = Path::new("D:\\test");
    let path_to = Path::new("D:\\out");
    // Creates 3 folders under source folder: path_from\test_folder\dir\sub
    let test_folder = path_from.join("test_folder");
    let dir = test_folder.join("dir");
    let sub = dir.join("sub");
    // Creates file under dir: path_from\test_folder\dir\file1.txt
    let file1 = dir.join("file1.txt");
    // Creates file under sub: path_from\test_folder\dir\sub\file2.txt
    let file2 = sub.join("file2.txt");

    // Recursively creates source folder structure and destination folder
    create_all(&sub, true)?;
    create_all(&path_to, true)?;
    // Writes string into new files (file1 and file 2)
    fs_extra::file::write_all(&file1, "content1")?;
    fs_extra::file::write_all(&file2, "content2")?;

    // Checks if files and folders created successfully, panics if false
    assert!(dir.exists());
    assert!(sub.exists());
    assert!(file1.exists());
    assert!(file2.exists());

    let mut options = CopyOptions::new();
    options.buffer_size = 536870912;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let handler = |process_info: TransitProcess| {
            tx.send(process_info).unwrap();
            thread::sleep(time::Duration::from_millis(100));
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        };
        copy_with_progress(&test_folder, &path_to, &options, handler).unwrap();
    });

    loop {
        match rx.try_recv() {
            Ok(process_info) => {
                println!("{} of {} bytes",
                         process_info.copied_bytes,
                         process_info.total_bytes);
            }
            Err(TryRecvError::Disconnected) => {
                println!("finished");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }
    Ok(())

}

/* Test hashing function
fn hash_test() {
    
} */

// Get current date and time, local to host machine
// Formatted to be safe for use as folder name as: YEAR-MONTH-DAYTHOURSMINUTES
// ex: 2020-02-06T1600 (Feb 6th 2020, 4PM)
fn current_date_time() -> std::string::String {
    let now: DateTime<Local> = Local::now();
    let now = now.format("%Y-%m-%dT%H%M").to_string();
    now
}

// This is super kludgey, I want this to error out if the match fails
// Currently only matches file/folder structure for windows
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

fn check_folder_valid(folder_input: &str) -> bool {
    let windows_re = Regex::new(r"^[a-zA-Z]:\\[\\\S|*\S]?.*$").unwrap();
    let linux_re = Regex::new(r"^(/[^/ ]*)+/?$").unwrap();
    let test: bool = (windows_re.is_match(folder_input)) || (linux_re.is_match(folder_input));
    test
}

fn main() {
    //example_copy();
    println!("{}", current_date_time());
    println!("Input a source folder: ");
    let source = folder_picker();
    println!("Input a destination folder: ");
    let destination = folder_picker();

    println!("Source is: {}", source);
    println!("Destination is: {}", destination);
    //hash_test();
}