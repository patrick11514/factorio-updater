use crate::{
    steps::{items::resolve_updates, update::do_update, updates::get_updates},
    structs::Args,
};

mod items;
mod update;
mod updates;

pub(crate) static TICK_STRINGS: &[&'static str] =
    &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub async fn handle_update(args: Args) -> anyhow::Result<()> {
    let mut args = args;

    let updates = get_updates(&args).await?;
    let updates = resolve_updates(&mut args, &updates)?;
    do_update(&args, updates).await?;

    Ok(())
}
