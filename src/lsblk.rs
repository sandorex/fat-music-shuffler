use crate::prelude::*;
use crate::util::BlockDevice;
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

    pub label: Option<String>,

    #[serde(rename = "rm")]
    pub removable: bool,

    pub model: Option<String>,

    #[serde(rename = "type")]
    pub dev_type: BlockDeviceType,

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

impl Into<BlockDevice> for &BlockDeviceInfo {
    fn into(self) -> BlockDevice {
        BlockDevice {
            path: self.path.clone(),
            removable: self.removable,
            is_partition: self.is_partition(),
            repr: format!("{}", self),
            partitions: self
                .children
                .as_ref()
                .map(|x| x.iter().map(|y| y.into()).collect()),
        }
    }
}

impl Into<BlockDevice> for BlockDeviceInfo {
    fn into(self) -> BlockDevice {
        Into::<BlockDevice>::into(&self)
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

pub fn query_block_device(path: &str) -> Result<BlockDevice> {
    if !std::fs::exists(path).unwrap_or(false) {
        bail!("Block device {path:?} does not exist");
    }

    query(Some(path))?
        .first()
        .map(|x| x.into())
        .with_context(|| anyhow!("lsblk returned no devices"))
}

pub fn query_all_block_devices() -> Result<Vec<BlockDevice>> {
    query(None).map(|x| {
        x.iter()
            .map(|y| Into::<BlockDevice>::into(y))
            .collect::<Vec<_>>()
    })
}
