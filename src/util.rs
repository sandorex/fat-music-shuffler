use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct BlockDevice {
    /// Path to open the device
    pub path: String,

    /// Is the device removable (SD Card, external SSD/HDD, etc..)
    pub removable: bool,

    /// Is the block device a partition or a disk
    pub is_partition: bool,

    /// Static human representation of the device
    pub repr: String,

    /// Partitions of the disk (if it is a disk)
    pub partitions: Option<Vec<Self>>,
}

// just print the representation
impl Display for BlockDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.repr)
    }
}

impl BlockDevice {
    pub fn open(&self, readonly: bool) -> std::io::Result<std::fs::File> {
        std::fs::OpenOptions::new()
            .read(true)
            .write(!readonly)
            .open(&self.path)
    }
}
