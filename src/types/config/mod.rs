mod wishlist;
pub use wishlist::{PkgRequest, Wishlist};

use clap::Clap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub arch: String,
    pub purge_on_remove: bool,
    pub repo: HashMap<String, RepoConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepoConfig {
    pub url: String,
    pub distribution: String,
    pub components: Vec<String>,
    pub certs: Vec<PathBuf>,
}

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Leo Shen <i@szclsya.me>")]
pub struct Opts {
    #[clap(long, default_value = "/", about = "Root directory for operation")]
    pub root: PathBuf,
    #[clap(long, default_value = "etc/apm/", about = "Position of the config folder")]
    pub config_root: PathBuf,
    #[clap(long, about = "Print additional debug information")]
    pub debug: bool,
    #[clap(long, about = "Unpack but not configure desired packages")]
    pub unpack_only: bool,
    #[clap(subcommand)]
    pub subcmd: Option<SubCmd>,
}

#[derive(Clap)]
pub enum SubCmd {
    #[clap(about = "Add or modify a package to wishlist")]
    Add(AddPkg),
    #[clap(about = "Remove a package from wishlist")]
    Rm(RmPkg),
    #[clap(about = "Update local package databases")]
    UpdateDb,
    #[clap(about = "Install and upgrade all packages according to wishlist")]
    InstallAll,
    #[clap(about = "Search packages from package database")]
    Search(SearchPkg),
}

#[derive(Clap)]
pub struct AddPkg {
    pub name: String,
}

#[derive(Clap)]
pub struct RmPkg {
    pub name: String,
}

#[derive(Clap)]
pub struct SearchPkg {
    pub keyword: String,
}
