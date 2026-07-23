use crate::{config::claudemds_dir, git};

pub fn run() -> anyhow::Result<()> {
    git::init_repo(&claudemds_dir())
}
