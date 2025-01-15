use anyhow::anyhow;
use aoide::desktop_app::ActionEffect;
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::libs::error::AnyResult;

use super::State;

#[tauri::command]
pub(crate) async fn set_music_directory<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, State>,
    root_url: Option<Url>,
) -> AnyResult<()> {
    {
        let collection_state = state.collection.read();
        if collection_state.is_synchronizing() {
            return Err(
                anyhow!("cannot update music directory while synchronizing collection").into(),
            );
        }
    }
    let root_dir = root_url
        .map(|url| {
            url.to_file_path()
                .map_err(|()| anyhow!("invalid URL: {url:?}"))
        })
        .transpose()?;
    if matches!(
        state
            .settings
            .update_music_dir(root_dir.as_deref().map(Into::into).as_ref()),
        ActionEffect::Unchanged
    ) {
        log::debug!("Music directory unchanged: {root_dir:?}");
        return Ok(());
    }
    log::info!("Music directory updated: {root_dir:?}");
    Ok(())
}
