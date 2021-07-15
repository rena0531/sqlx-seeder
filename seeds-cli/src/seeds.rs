use anyhow::{bail, Context};
use chrono::Utc;
use console::style;
use sqlx::seeds::{AppliedSeeds, Seeds, SeedsError, SeedsType, Seeder};
use sqlx::{AnyConnection, Connection};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
// ファイル作成？
fn create_file(
    seeds_source: &str,
    file_prefix: &str,
    description: &str,
    seeds_type: SeedsType,
) -> anyhow::Result<()> {
    use std::path::PathBuf;

    let mut file_name = file_prefix.to_string();
    file_name.push_str("_");
    file_name.push_str(&description.replace(' ', "_"));
    file_name.push_str(seeds_type.suffix());
// パス作成
    let mut path = PathBuf::new();
    path.push(seeds_source);
    path.push(&file_name);

    println!("Creating {}", style(path.display()).cyan());

    let mut file = File::create(&path).context("Failed to create seeds file")?;

    file.write_all(seeds_type.file_content().as_bytes())?;

    Ok(())
}

pub async fn add(
    seeds_source: &str,
    description: &str,
    reversible: bool,
) -> anyhow::Result<()> {
    fs::create_dir_all(seeds_source).context("Unable to create seedss directory")?;

    let seeder = Seeder::new(Path::new(seeds_source)).await?;
    // This checks if all existing seedss are of the same type as the reverisble flag passed
    for seeds in seeder.iter() {
        if seeds.seeds_type.is_reversible() != reversible {
            bail!(SeedsError::InvalidMixReversibleAndSimple);
        }
    }

    let dt = Utc::now();
    let file_prefix = dt.format("%Y%m%d%H%M%S").to_string();
    // ファイル作成？
    
    if reversible {
        create_file(
            seeds_source,
            &file_prefix,
            description,
            SeedsType::ReversibleUp,
        )?;
        create_file(
            seeds_source,
            &file_prefix,
            description,
            SeedsType::ReversibleDown,
        )?;
    } else {
        create_file(
            seeds_source,
            &file_prefix,
            description,
            SeedsType::Simple,
        )?;
    }

    Ok(())
}

pub async fn info(seeds_source: &str, uri: &str) -> anyhow::Result<()> {
    let seeder = Seeder::new(Path::new(seeds_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_seedss_table().await?;

    let applied_seedss: HashMap<_, _> = conn
        .list_applied_seedss()
        .await?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for seeds in seeder.iter() {
        println!(
            "{}/{} {}",
            style(seeds.version).cyan(),
            if applied_seedss.contains_key(&seeds.version) {
                style("installed").green()
            } else {
                style("pending").yellow()
            },
            seeds.description,
        );
    }

    Ok(())
}

fn validate_applied_seedss(
    applied_seedss: &[AppliedSeeds],
    seeder: &Seeder,
    ignore_missing: bool,
) -> Result<(), SeedsError> {
    if ignore_missing {
        return Ok(());
    }

    let seedss: HashSet<_> = seeder.iter().map(|m| m.version).collect();

    for applied_seeds in applied_seedss {
        if !seedss.contains(&applied_seeds.version) {
            return Err(SeedsError::VersionMissing(applied_seeds.version));
        }
    }

    Ok(())
}

pub async fn run(
    seeds_source: &str,
    uri: &str,
    dry_run: bool,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    let seeder = Seeder::new(Path::new(seeds_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_seedss_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(SeedsError::Dirty(version));
    }

    let applied_seedss = conn.list_applied_seedss().await?;
    validate_applied_seedss(&applied_seedss, &seeder, ignore_missing)?;

    let applied_seedss: HashMap<_, _> = applied_seedss
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for seeds in seeder.iter() {
        if seeds.seeds_type.is_down_seeds() {
            // Skipping down seedss
            continue;
        }

        match applied_seedss.get(&seeds.version) {
            Some(applied_seeds) => {
                if seeds.checksum != applied_seeds.checksum {
                    bail!(SeedsError::VersionMismatch(seeds.version));
                }
            }
            None => {
                let elapsed = if dry_run {
                    Duration::new(0, 0)
                } else {
                    conn.apply(seeds).await?
                };
                let text = if dry_run { "Can apply" } else { "Applied" };

                println!(
                    "{} {}/{} {} {}",
                    text,
                    style(seeds.version).cyan(),
                    style(seeds.seeds_type.label()).green(),
                    seeds.description,
                    style(format!("({:?})", elapsed)).dim()
                );
            }
        }
    }

    Ok(())
}

pub async fn revert(
    seeds_source: &str,
    uri: &str,
    dry_run: bool,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    let seeder = Seeder::new(Path::new(seeds_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_seedss_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(SeedsError::Dirty(version));
    }

    let applied_seedss = conn.list_applied_seedss().await?;
    validate_applied_seedss(&applied_seedss, &seeder, ignore_missing)?;

    let applied_seedss: HashMap<_, _> = applied_seedss
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    let mut is_applied = false;
    for seeds in seeder.iter().rev() {
        if !seeds.seeds_type.is_down_seeds() {
            // Skipping non down seeds
            // This will skip any simple or up seeds file
            continue;
        }

        if applied_seedss.contains_key(&seeds.version) {
            let elapsed = if dry_run {
                Duration::new(0, 0)
            } else {
                conn.revert(seeds).await?
            };
            let text = if dry_run { "Can apply" } else { "Applied" };

            println!(
                "{} {}/{} {} {}",
                text,
                style(seeds.version).cyan(),
                style(seeds.seeds_type.label()).green(),
                seeds.description,
                style(format!("({:?})", elapsed)).dim()
            );

            is_applied = true;
            // Only a single seeds will be reverted at a time, so we break
            break;
        }
    }
    if !is_applied {
        println!("No seedss available to revert");
    }

    Ok(())
}
