use std::{path::Path, sync::Arc};

use aoide::backend_embedded::Environment;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod database;

mod import;
#[allow(unused_imports)] // TODO
use self::import::{
    import_playlist_from_entity, import_playlist_track_entries, import_track_from_entity,
};

const PLUGIN_NAME: &str = "aoide";

struct State {
    env: Environment,
    settings: Arc<aoide::desktop_app::settings::ObservableState>,
    collection: Arc<aoide::desktop_app::collection::ObservableState>,
}

impl State {
    fn new(storage_dir: &Path) -> anyhow::Result<Self> {
        let db_config = database::default_config(storage_dir);
        let env = Environment::commission(&db_config)?;
        let settings = Default::default();
        let collection = Default::default();
        Ok(Self {
            env,
            settings,
            collection,
        })
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![
            commands::collection::set_music_directory,
        ])
        .setup(|app_handle, _plugin_api| {
            let storage_dir = app_handle.path().app_local_data_dir()?;
            app_handle.manage(State::new(&storage_dir)?);
            Ok(())
        })
        .build()
}
