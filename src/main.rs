// Bring in external crates fs_extra and twox_hash
// More will probably be needed as I disocver which are necessary

extern crate fs_extra;
extern crate twox_hash;
extern crate chrono;

// Bring in standard components
use std::path::Path;
use std::{thread, time};
use std::sync::mpsc::{self, TryRecvError};
use chrono::{DateTime, Local};

// Bring in fs_extra components
use fs_extra::dir::*;
use fs_extra::error::*;


fn example_copy() -> Result<()> {

    let path_from = Path::new("D:\\test");
    let path_to = Path::new("D:\\out");
    let test_folder = path_from.join("test_folder");
    let dir = test_folder.join("dir");
    let sub = dir.join("sub");
    let file1 = dir.join("file1.txt");
    let file2 = sub.join("file2.txt");

    create_all(&sub, true)?;
    create_all(&path_to, true)?;
    fs_extra::file::write_all(&file1, "content1")?;
    fs_extra::file::write_all(&file2, "content2")?;

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

fn current_date_time() -> std::string::String {
    let now: DateTime<Local> = Local::now();
    let now = now.format("%Y-%m-%dT%H%M").to_string();
    return now;
}

fn main() {
    example_copy();
    println!("{}", current_date_time());
}
