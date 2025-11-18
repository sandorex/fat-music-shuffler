//! Contains text files

pub const README: &str = r#"Filesystem managed by fat32-music-shuffler

DO NOT EDIT THE FILESYSTEM IF YOU DONT KNOW WHAT YOU ARE DOING

If the file "DO_NOT_MODIFY" is present it means there are hardlinks in place and any modifications to them will corrupt the filesystem

To add or/and remove files use `f32ms import` or `f32ms clean`!
"#;
