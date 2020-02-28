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
let j = serde_json::to_string(&file_hash);
println!("J: {:?}", j);

let temp5 = folder_validator2(r"D:\");
fn folder_validator2(folder_input: &str) -> std::result::Result<String, io::Error>{
    let mut f = File::open(&folder_input.to_string())?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}