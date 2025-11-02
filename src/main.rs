use std::fs::OpenOptions;
use std::io::{self, prelude::*};

use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    dbg!(&args);

    if args.len() < 2 {
        panic!("Please provide a path to the file");
    }

    let img_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(args.get(1).unwrap())
    {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open image!");
            return Err(err);
        }
    };
    let buf_stream = BufStream::new(img_file);
    let options = FsOptions::new().update_accessed_date(true);
    let fs = FileSystem::new(buf_stream, options)?;
    let root = fs.root_dir();
    root.create_hardlink("10.mp3", "6.mp3")?;
    root.create_hardlink("11.mp3", "9.mp3")?;
    root.create_hardlink("12.mp3", "6.mp3")?;
    root.create_hardlink("13.mp3", "9.mp3")?;
    root.create_hardlink("14.mp3", "6.mp3")?;
    root.create_hardlink("15.mp3", "9.mp3")?;
    // for i in root.iter().flatten() {
    //     println!("{i:?}");
    // }
    // let mut file = fs.root_dir().create_file("hello.txt")?;
    // file.write_all(b"Hello World!")?;
    Ok(())
}
