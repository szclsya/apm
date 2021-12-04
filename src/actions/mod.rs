mod execute;
mod provide;
mod search;
use execute::execute;
//use search::search_deb_db;

use crate::{
    db::LocalDb,
    executor::MachineStatus,
    info, success,
    types::config::{Blueprints, Config, IgnoreRules, Opts, SubCmd}
};

use anyhow::{Context, Result};
use std::path::PathBuf;

pub enum UserRequest {
    // Vec<(PkgName, install_recomm)>
    Install(Vec<(String, bool)>),
    // Vec<(PkgName, remove_recomm)>
    Remove(Vec<(String, bool)>),
    Upgrade,
}

/// bool in return type indicated whether the blueprint is altered
pub async fn fullfill_command(
    config: &Config,
    opts: &Opts,
    blueprints: &mut Blueprints,
    ignorerules: &mut IgnoreRules,
) -> Result<()> {
    let downloader = crate::utils::downloader::Downloader::new();
    // Directory that stores trusted public keys for repos
    let key_root = opts.root.join(crate::DB_KEY_PATH);
    let localdb = LocalDb::new(
        opts.root.join(crate::DB_CACHE_PATH),
        key_root,
        config.repo.clone(),
        &config.arch,
    );

    match &opts.subcmd {
        SubCmd::Install(add) => {
            let req: Vec<(String, bool)> = add
                .names
                .iter()
                .map(|pkgname| (pkgname.clone(), !add.no_recommends))
                .collect();
            let req = UserRequest::Install(req);
            // Update local db
            info!("Refreshing local package databases...");
            localdb.update(&downloader).await?;
            // Execute blueprint
            execute(
                &localdb,
                &downloader,
                blueprints,
                ignorerules,
                opts,
                config,
                req,
            )
            .await?;
            Ok(())
        }
        SubCmd::Remove(rm) => {
            // Prepare request
            let req: Vec<(String, bool)> = rm
                .names
                .iter()
                .map(|name| (name.clone(), rm.remove_recommends))
                .collect();
            let req = UserRequest::Remove(req);
            // Update local db
            info!("Refreshing local package databases...");
            localdb.update(&downloader).await?;
            // Apply stuff
            execute(
                &localdb,
                &downloader,
                blueprints,
                ignorerules,
                opts,
                config,
                req,
            )
            .await?;
            Ok(())
        }
        SubCmd::Refresh => {
            info!("Refreshing local package databases...");
            localdb.update(&downloader).await?;
            success!("Refresh complete");
            Ok(())
        }
        SubCmd::Execute | SubCmd::Upgrade => {
            let req = UserRequest::Upgrade;
            info!("Refreshing local package databases...");
            localdb
                .update(&downloader)
                .await
                .context("Failed to refresh local package database")?;

            execute(
                &localdb,
                &downloader,
                blueprints,
                ignorerules,
                opts,
                config,
                req,
            )
            .await?;

            Ok(())
        }
        SubCmd::Search(search) => {
            let dbs: Vec<PathBuf> = localdb
                .get_all_contents_db()
                .context("Invalid local package database")?
                .into_iter()
                .map(|(_, path)| path)
                .collect();
            let machine_status = MachineStatus::new(&opts.root)?;

            for pkginfo in search::search_deb_db(&dbs, &search.keyword)? {
                pkginfo.show(&machine_status)?;
            }

            Ok(())
        }
        SubCmd::Provide(provide) => {
            let dbs: Vec<PathBuf> = localdb
                .get_all_contents_db()
                .context("Invalid local package database")?
                .into_iter()
                .map(|(_, path)| path)
                .collect();

            for pkgname in search::provide_file(&dbs, &provide.file)? {
                crate::WRITER.writeln("", &pkgname)?;
            }
            Ok(())
        }
        SubCmd::Clean(cleanconfig) => {
            info!("Cleaning local package cache...");
            let pkg_cache_path = opts.root.join(crate::PKG_CACHE_PATH);
            std::fs::remove_dir_all(&pkg_cache_path)?;
            std::fs::create_dir_all(&pkg_cache_path)?;

            if cleanconfig.all {
                info!("Cleaning local database cache...");
                let db_cache_path = opts.root.join(crate::DB_CACHE_PATH);
                std::fs::remove_dir_all(&db_cache_path)?;
                std::fs::create_dir_all(&db_cache_path)?;
            }
            Ok(())
        }
    }
}
