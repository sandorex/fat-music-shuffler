use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum BlockDeviceType {
    #[serde(rename = "part")]
    Partition,

    #[serde(rename = "disk")]
    Disk,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockDeviceInfo {
    pub path: String,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub dev_type: BlockDeviceType,
    pub size: String,
}

impl BlockDeviceInfo {
    fn parse(input: &str) -> Result<Vec<Self>> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            blockdevices: Vec<BlockDeviceInfo>,
        }

        let data = serde_json::from_str::<Data>(input)
            .with_context(|| anyhow!("Error parsing block devices from blkid"))?;

        Ok(data.blockdevices)
    }

    pub fn is_partition(&self) -> bool {
        match self.dev_type {
            BlockDeviceType::Partition => true,
            _ => false,
        }
    }
}

pub fn query_block_device(path: &str) -> Result<BlockDeviceInfo> {
    if !std::fs::exists(path).unwrap_or(false) {
        bail!("Block device {path:?} does not exist");
    }

    let cmd = std::process::Command::new("lsblk")
        .args(["-O", "--json", path])
        .output()
        .with_context(|| anyhow!("Could not run lsblk"))?;

    if !cmd.status.success() {
        bail!("lsblk exited with code {:?}", cmd.status.code())
    }

    let stdout = String::from_utf8(cmd.stdout)?;
    Ok(BlockDeviceInfo::parse(&stdout)?.pop().unwrap())
}
