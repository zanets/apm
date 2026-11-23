use crate::{config::store_dir, git};

pub fn run() -> anyhow::Result<()> {
    git::init_repo(&store_dir())
}
