use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use directories::ProjectDirs;

use crate::Args;

pub struct Config {
    pub cookie_file: PathBuf,
    pub icon_dir: PathBuf,
    pub check_interval: Duration,
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(_args: Args) -> Result<Self, Self::Error> {
        let project_dirs = ProjectDirs::from("", "", "ogs-notify")
            .ok_or(anyhow!("Could not get project directories."))?;
        let data_dir = project_dirs.data_dir();

        // TODO support config file.

        let mut cookie_file = data_dir.to_owned();
        cookie_file.push("cookies.json");

        let mut icon_dir = data_dir.to_owned();
        icon_dir.push("icons");

        let check_interval = Duration::from_secs(60);

        Ok(Config {
            cookie_file,
            icon_dir,
            check_interval,
        })
    }
}
