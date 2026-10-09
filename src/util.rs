use std::{fmt::Display, path::PathBuf};

/// Describes block device and holds its partitions
#[derive(Debug, Clone)]
pub struct Disk {
    pub path: String,
    pub removeable: bool,
    pub size: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub partitions: Vec<Partition>,
}

impl Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;

        if let Some(model) = self.model.as_ref() {
            write!(f, " {}", model)?;
        }

        write!(f, " {}", self.size)?;

        if self.removeable {
            write!(f, " RM")?;
        }

        Ok(())
    }
}

impl Disk {
    pub fn open(&self, readonly: bool) -> std::io::Result<std::fs::File> {
        std::fs::OpenOptions::new()
            .read(true)
            .write(!readonly)
            .open(&self.path)
    }
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub path: String,
    pub removeable: bool,
    pub size: String,
    pub mounted: bool,
    pub label: Option<String>,
}

impl Display for Partition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.path, self.size)?;

        if self.removeable {
            write!(f, " RM")?;
        }

        if self.mounted {
            write!(f, " M")?;
        }

        Ok(())
    }
}

impl Partition {
    pub fn open(&self, readonly: bool) -> std::io::Result<std::fs::File> {
        std::fs::OpenOptions::new()
            .read(true)
            .write(!readonly)
            .open(&self.path)
    }
}

#[derive(Debug, Clone)]
pub enum DiskOrPartition {
    Disk(Disk),
    Partition(Partition)
}

impl Display for DiskOrPartition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskOrPartition::Disk(x) => write!(f, "{}", x),
            DiskOrPartition::Partition(x) => write!(f, "{}", x),
        }
    }
}

impl DiskOrPartition {
    pub fn open(&self, readonly: bool) -> std::io::Result<std::fs::File> {
        match self {
            DiskOrPartition::Disk(x) => x.open(readonly),
            DiskOrPartition::Partition(x) => x.open(readonly),
        }
    }

    pub fn is_partition(&self) -> bool {
        match self {
            DiskOrPartition::Partition(_) => true,
            _ => false,
        }
    }

    pub fn model_or_label(&self) -> Option<&str> {
        match self {
            DiskOrPartition::Disk(x) => x.model.as_ref().map(|x| x.as_str()),
            DiskOrPartition::Partition(x) => x.label.as_ref().map(|x| x.as_str()),
        }
    }

    pub fn size(&self) -> &str {
        match self {
            DiskOrPartition::Disk(x) => x.size.as_str(),
            DiskOrPartition::Partition(x) => x.size.as_str(),
        }
    }

    pub fn removeable(&self) -> bool {
        match self {
            DiskOrPartition::Disk(x) => x.removeable,
            DiskOrPartition::Partition(x) => x.removeable,
        }
    }

    pub fn path(&self) -> &str {
        match self {
            DiskOrPartition::Disk(x) => &x.path,
            DiskOrPartition::Partition(x) => &x.path,
        }
    }
}

impl From<Disk> for DiskOrPartition {
    fn from(value: Disk) -> Self {
        Self::Disk(value)
    }
}

impl From<Partition> for DiskOrPartition {
    fn from(value: Partition) -> Self {
        Self::Partition(value)
    }
}

pub fn find_mp3_files(vec: &mut Vec<PathBuf>, path: PathBuf) -> std::io::Result<()> {
    if path.is_dir() {
        let paths = std::fs::read_dir(&path)?;
        for path_result in paths {
            let full_path = path_result?.path();
            find_mp3_files(vec, full_path)?;
        }
    } else {
        // only collect MP3 files
        if let Some(ext) = path.extension() {
            if ext == "mp3" {
                vec.push(path);
            }
        }
    }

    Ok(())
}
