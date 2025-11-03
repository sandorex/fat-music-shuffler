//! Contains text files

pub const README: &str = r#"Please read this file before editing the filesystem

This filesystem is managed by f32ms (fat32-music-shuffler) and it uses hardlinks, so deleting/modifying ANY FILE will corrupt the filesystem

If the file DO_NOT_MODIFY is present that means the filesystem has hardlinks

"#;
