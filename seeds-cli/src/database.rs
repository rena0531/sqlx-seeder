use crate::seeds::*;
use console::style;
use dialoguer::Confirm;
use sqlx::any::Any;

pub async fn create(uri: &str) -> anyhow::Result<()> {
    if !Any::database_exists(uri).await? {
        Any::create_database(uri).await?;
    }

    Ok(())
}

pub async fn drop(uri: &str, confirm: bool) -> anyhow::Result<()> {
    if confirm
        && !Confirm::new()
            .with_prompt(format!(
                "\nAre you sure you want to drop the database at {}?",
                style(uri).cyan()
            ))
            .wait_for_newline(true)
            .default(false)
            .interact()?
    {
        return Ok(());
    }

    if Any::database_exists(uri).await? {
        Any::drop_database(uri).await?;
    }

    Ok(())
}

pub async fn reset(seeds_source: &str, uri: &str, confirm: bool) -> anyhow::Result<()> {
    drop(uri, confirm).await?;
    setup(seeds_source, uri).await
}

pub async fn setup(seeds_source: &str, uri: &str) -> anyhow::Result<()> {
    create(uri).await?;
    seeds::run(seeds_source, uri, false, false).await
}
