use crate::prelude::*;
use crate::util::{Disk, DiskOrPartition, Partition};
use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Clone, Deserialize)]
enum BlockDeviceType {
    #[serde(rename = "part")]
    Partition,

    #[serde(rename = "disk")]
    Disk,
}

#[derive(Debug, Clone, Deserialize)]
struct BlockDeviceInfo {
    pub path: String,

    #[serde(rename = "rm")]
    pub removable: bool,

    pub model: Option<String>,

    pub serial: Option<String>,

    pub label: Option<String>,
    pub partlabel: Option<String>,

    #[serde(rename = "type")]
    pub dev_type: BlockDeviceType,

    pub mountpoints: Vec<String>,

    pub size: String,

    pub children: Option<Vec<Self>>,
}

impl BlockDeviceInfo {
    fn parse(input: &str) -> Result<Vec<Self>> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            blockdevices: Vec<BlockDeviceInfo>,
        }

        let data = serde_json::from_str::<Data>(input)
            .with_context(|| anyhow!("Error parsing block devices from lsblk"))?;

        Ok(data.blockdevices)
    }

    fn is_partition(&self) -> bool {
        match self.dev_type {
            BlockDeviceType::Partition => true,
            BlockDeviceType::Disk => false,
        }
    }

    fn parse_disk(&self) -> Disk {
        Disk {
            path: self.path.clone(),
            model: self.model.as_ref().map(|x| x.trim().to_string()),
            serial: self.serial.as_ref().map(|x| x.trim().to_string()),
            removeable: self.removable,
            size: self.size.clone(),
            partitions: self.children
                .as_ref()
                .map(|x| x.into_iter().map(|y| y.parse_partition()).collect())
                .unwrap_or_default(),
        }
    }

    fn parse_partition(&self) -> Partition {
        Partition {
            path: self.path.clone(),
            removeable: self.removable,
            size: self.size.clone(),
            mounted: !self.mountpoints.is_empty(),
            label: self.label.as_ref().or(self.partlabel.as_ref()).cloned(),
        }
    }
}

impl Display for BlockDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;

        // NOTE: label and note probably wont be availabe at the same time
        if let Some(label) = self.label.as_ref() {
            write!(f, " {:?}", label.trim())?;
        }

        if let Some(model) = self.model.as_ref() {
            write!(f, " {:?}", model.trim())?;
        }

        write!(f, " {}", self.size)?;

        Ok(())
    }
}

impl Into<DiskOrPartition> for &BlockDeviceInfo {
    fn into(self) -> DiskOrPartition {
        if self.is_partition() {
            DiskOrPartition::Partition(self.parse_partition())
        } else {
            DiskOrPartition::Disk(self.parse_disk())
        }
    }
}

impl Into<DiskOrPartition> for BlockDeviceInfo {
    fn into(self) -> DiskOrPartition {
        Into::<DiskOrPartition>::into(&self)
    }
}

fn query(path: Option<&str>) -> Result<Vec<BlockDeviceInfo>> {
    let mut cmd = std::process::Command::new("lsblk");
    // -O => output all columns
    // -A => skip empty devices (like empty sdcard slots)
    cmd.args(["-O", "-A", "--json"]);

    if let Some(path) = path {
        cmd.arg(path);
    }

    let cmd = cmd
        .output()
        .with_context(|| anyhow!("Could not run lsblk"))?;

    if !cmd.status.success() {
        bail!("lsblk exited with code {:?}", cmd.status.code())
    }

    let stdout = String::from_utf8(cmd.stdout)?;
    Ok(BlockDeviceInfo::parse(&stdout)?)
}

pub fn query_block_device(path: &str) -> Result<DiskOrPartition> {
    if !std::fs::exists(path).unwrap_or(false) {
        bail!("Block device {path:?} does not exist");
    }

    query(Some(path))?
        .first()
        .map(Into::<DiskOrPartition>::into)
        .with_context(|| anyhow!("lsblk returned no devices"))
}

pub fn query_all_disks() -> Result<Vec<Disk>> {
    query(None).map(|x| {
        x.iter()
            .map(|y| y.parse_disk())
            .collect::<Vec<_>>()
    })
}

/// Returns both partitions and disks but disks go first
pub fn query_all_block_devices() -> Result<Vec<DiskOrPartition>> {
    Ok(query_all_disks()?
        .into_iter()
        .fold(vec![], |mut acc: Vec<DiskOrPartition>, mut x| {
            let partitions = std::mem::take(&mut x.partitions);
            acc.push(x.into());
            acc.extend(partitions.into_iter().map(|part| part.into()));
            acc
        }))
}
