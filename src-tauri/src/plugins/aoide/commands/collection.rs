use anyhow::anyhow;
use aoide::desktop_app::StateUnchanged;
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::libs::error::AnyResult;

use super::State;

fn default_kind<R: Runtime>(app: &AppHandle<R>) -> &str {
    &app.config().identifier
}

#[tauri::command]
pub(crate) async fn set_music_directory<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, State>,
    root_url: Option<Url>,
) -> AnyResult<()> {
    let collection_state = state.collection.read();
    if state.collection.read().is_synchronizing() {
        return Err(
            anyhow!("cannot update music directories while synchronizing collection").into(),
        );
    }
    let root_dir = root_url
        .map(|url| {
            url.to_file_path()
                .map_err(|()| anyhow!("invalid URL: {url:?}"))
        })
        .transpose()?;
    if let Err(StateUnchanged) = state
        .settings
        .update_music_dir(root_dir.as_deref().map(Into::into).as_ref())
    {
        log::debug!("Music directory unchanged: {root_dir:?}");
        return Ok(());
    }
    log::info!("Music directory updated: {root_dir:?}");
    Ok(())
}
