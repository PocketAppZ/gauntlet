use std::collections::HashMap;
use include_dir::{Dir, include_dir};

use common::model::{ActionShortcut, ActionShortcutKind, DownloadStatus, EntrypointId, PluginId, PluginPreference, PluginPreferenceUserData, PreferenceEnumValue, SearchResult, SettingsEntrypoint, SettingsEntrypointType, SettingsPlugin, UiPropertyValue, UiWidgetId};
use common::rpc::frontend_api::FrontendApi;
use common::rpc::frontend_server::wait_for_frontend_server;

use crate::dirs::Dirs;
use crate::plugins::config_reader::ConfigReader;
use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_from_str, DbPluginActionShortcutKind, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData, DbReadPluginEntrypoint};
use crate::plugins::icon_cache::IconCache;
use crate::plugins::js::{AllPluginCommandData, OnePluginCommandData, PluginCode, PluginCommand, PluginPermissions, PluginRuntimeData, start_plugin_runtime};
use crate::plugins::loader::PluginLoader;
use crate::plugins::run_status::RunStatusHolder;
use crate::search::SearchIndex;

pub mod js;
mod data_db_repository;
mod config_reader;
mod loader;
mod run_status;
mod download_status;
mod applications;
mod icon_cache;
pub(super) mod frecency;


static BUILTIN_PLUGINS: [(&str, Dir); 3] = [
    ("applications", include_dir!("$CARGO_MANIFEST_DIR/../../bundled_plugins/applications/dist")),
    ("calculator", include_dir!("$CARGO_MANIFEST_DIR/../../bundled_plugins/calculator/dist")),
    ("settings", include_dir!("$CARGO_MANIFEST_DIR/../../bundled_plugins/settings/dist")),
];

pub struct ApplicationManager {
    config_reader: ConfigReader,
    search_index: SearchIndex,
    command_broadcaster: tokio::sync::broadcast::Sender<PluginCommand>,
    db_repository: DataDbRepository,
    plugin_downloader: PluginLoader,
    run_status_holder: RunStatusHolder,
    icon_cache: IconCache,
    frontend_api: FrontendApi,
}

impl ApplicationManager {
    pub async fn create() -> anyhow::Result<Self> {
        wait_for_frontend_server().await;

        let frontend_api = FrontendApi::new().await?;
        let dirs = Dirs::new();
        let db_repository = DataDbRepository::new(dirs.clone()).await?;
        let plugin_downloader = PluginLoader::new(db_repository.clone());
        let config_reader = ConfigReader::new(dirs.clone(), db_repository.clone());
        let icon_cache = IconCache::new(dirs.clone());
        let run_status_holder = RunStatusHolder::new();
        let search_index = SearchIndex::create_index(frontend_api.clone())?;

        let (command_broadcaster, _) = tokio::sync::broadcast::channel::<PluginCommand>(100);

        Ok(Self {
            config_reader,
            search_index,
            command_broadcaster,
            db_repository,
            plugin_downloader,
            run_status_holder,
            icon_cache,
            frontend_api,
        })
    }

    pub fn clear_all_icon_cache_dir(&self) -> anyhow::Result<()> {
        self.icon_cache.clear_all_icon_cache_dir()
    }

    pub async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        self.plugin_downloader.download_plugin(plugin_id).await
    }

    pub fn download_status(&self) -> HashMap<PluginId, DownloadStatus> {
        self.plugin_downloader.download_status()
    }

    pub fn search(&self, text: &str) -> anyhow::Result<Vec<SearchResult>> {
        self.search_index.create_handle()
            .search(&text)
    }

    pub async fn save_local_plugin(
        &self,
        path: &str,
    ) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Saving local plugin at path: {:?}", path);

        let plugin_id = self.plugin_downloader.save_local_plugin(path).await?;

        self.reload_plugin(plugin_id).await?;

        Ok(())
    }

    pub async fn load_builtin_plugins(&self) -> anyhow::Result<()> {
        for (id, dir) in &BUILTIN_PLUGINS {
            tracing::info!(target = "plugin", "Saving builtin plugin with id: {:?}", id);

            let plugin_id = self.plugin_downloader.save_builtin_plugin(id, dir).await?;

            self.reload_plugin(plugin_id).await?;
        }

        Ok(())
    }

    pub async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>> {
        let result = self.db_repository
            .list_plugins_and_entrypoints()
            .await?
            .into_iter()
            .map(|(plugin, entrypoints)| {
                let entrypoints = entrypoints
                    .into_iter()
                    .map(|entrypoint| {
                        let entrypoint_id = EntrypointId::from_string(entrypoint.id);

                        let entrypoint = SettingsEntrypoint {
                            enabled: entrypoint.enabled,
                            entrypoint_id: entrypoint_id.clone(),
                            entrypoint_name: entrypoint.name,
                            entrypoint_description: entrypoint.description,
                            entrypoint_type: match db_entrypoint_from_str(&entrypoint.entrypoint_type) {
                                DbPluginEntrypointType::Command => SettingsEntrypointType::Command,
                                DbPluginEntrypointType::View => SettingsEntrypointType::View,
                                DbPluginEntrypointType::InlineView => SettingsEntrypointType::InlineView,
                                DbPluginEntrypointType::CommandGenerator => SettingsEntrypointType::CommandGenerator,
                            }.into(),
                            preferences: entrypoint.preferences.into_iter()
                                .map(|(key, value)| (key, plugin_preference_from_db(value)))
                                .collect(),
                            preferences_user_data: entrypoint.preferences_user_data.into_iter()
                                .map(|(key, value)| (key, plugin_preference_user_data_from_db(value)))
                                .collect(),
                        };

                        (entrypoint_id, entrypoint)
                    })
                    .collect();

                SettingsPlugin {
                    plugin_id: PluginId::from_string(plugin.id),
                    plugin_name: plugin.name,
                    plugin_description: plugin.description,
                    enabled: plugin.enabled,
                    entrypoints,
                    preferences: plugin.preferences.into_iter()
                        .map(|(key, value)| (key, plugin_preference_from_db(value)))
                        .collect(),
                    preferences_user_data: plugin.preferences_user_data.into_iter()
                        .map(|(key, value)| (key, plugin_preference_user_data_from_db(value)))
                        .collect(),
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn set_plugin_state(&self, plugin_id: PluginId, set_enabled: bool) -> anyhow::Result<()> {
        let currently_running = self.run_status_holder.is_plugin_running(&plugin_id);
        let currently_enabled = self.is_plugin_enabled(&plugin_id).await?;

        tracing::info!(target = "plugin", "Setting plugin state for plugin id: {:?}, currently_running: {}, currently_enabled: {}, set_enabled: {}", plugin_id, currently_running, currently_enabled, set_enabled);

        match (currently_running, currently_enabled, set_enabled) {
            (false, false, true) => {
                self.db_repository.set_plugin_enabled(&plugin_id.to_string(), true)
                    .await?;

                self.start_plugin(plugin_id).await?;
            }
            (false, true, true) => {
                self.start_plugin(plugin_id).await?;
            }
            (true, true, false) => {
                self.db_repository.set_plugin_enabled(&plugin_id.to_string(), false)
                    .await?;

                self.stop_plugin(plugin_id.clone()).await;
                self.search_index.remove_for_plugin(plugin_id)?;
            }
            (true, false, _) => {
                tracing::error!("Plugin is running but is disabled, please report this: {}", plugin_id.to_string())
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn set_entrypoint_state(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        self.db_repository.set_plugin_entrypoint_enabled(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled)
            .await?;

        self.request_search_index_reload(plugin_id);

        Ok(())
    }

    pub async fn set_preference_value(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, preference_name: String, preference_value: PluginPreferenceUserData) -> anyhow::Result<()> {
        let user_data = plugin_preference_user_data_to_db(preference_value);

        self.db_repository.set_preference_value(plugin_id.to_string(), entrypoint_id.map(|id| id.to_string()), preference_name, user_data)
            .await?;

        Ok(())
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        self.config_reader.reload_config().await?;

        Ok(())
    }

    pub async fn reload_all_plugins(&mut self) -> anyhow::Result<()> {
        tracing::info!("Reloading all plugins");

        self.reload_config().await?;

        for plugin in self.db_repository.list_plugins().await? {
            let plugin_id = PluginId::from_string(plugin.id);
            let running = self.run_status_holder.is_plugin_running(&plugin_id);
            match (running, plugin.enabled) {
                (false, true) => {
                    self.start_plugin(plugin_id).await?;
                }
                (true, false) => {
                    self.stop_plugin(plugin_id.clone()).await;
                    self.search_index.remove_for_plugin(plugin_id)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn remove_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Removing plugin with id: {:?}", plugin_id);

        self.stop_plugin(plugin_id.clone()).await;
        self.db_repository.remove_plugin(&plugin_id.to_string()).await?;
        self.search_index.remove_for_plugin(plugin_id)?;
        Ok(())
    }

    pub fn handle_inline_view(&self, text: &str) {
        self.send_command(PluginCommand::All {
            data: AllPluginCommandData::OpenInlineView {
                text: text.to_owned()
            }
        })
    }

    pub async fn handle_run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) {
        self.send_command(PluginCommand::One {
            id: plugin_id.clone(),
            data: OnePluginCommandData::RunCommand {
                entrypoint_id: entrypoint_id.to_string(),
            }
        });

        self.mark_entrypoint_frecency(plugin_id, entrypoint_id).await
    }

    pub async fn handle_run_generated_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) {
        self.send_command(PluginCommand::One {
            id: plugin_id.clone(),
            data: OnePluginCommandData::RunGeneratedCommand {
                entrypoint_id: entrypoint_id.to_string(),
            }
        });

        self.mark_entrypoint_frecency(plugin_id, entrypoint_id).await
    }

    pub async fn handle_render_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) {
        self.send_command(PluginCommand::One {
            id: plugin_id.clone(),
            data: OnePluginCommandData::RenderView {
                entrypoint_id: entrypoint_id.clone(),
            }
        });

        self.mark_entrypoint_frecency(plugin_id, entrypoint_id).await
    }

    pub fn handle_view_close(&self, plugin_id: PluginId) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::CloseView
        })
    }

    pub fn handle_view_event(&self, plugin_id: PluginId, widget_id: UiWidgetId, event_name: String, event_arguments: Vec<UiPropertyValue>) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::HandleViewEvent {
                widget_id,
                event_name,
                event_arguments
            }
        })
    }

    pub fn handle_keyboard_event(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, key: String, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::HandleKeyboardEvent {
                entrypoint_id,
                key,
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            }
        })
    }

    pub fn request_search_index_reload(&self, plugin_id: PluginId) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::ReloadSearchIndex
        })
    }

    async fn reload_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Reloading plugin with id: {:?}", plugin_id);

        let running = self.run_status_holder.is_plugin_running(&plugin_id);
        if running {
            self.stop_plugin(plugin_id.clone()).await;
        }

        self.start_plugin(plugin_id).await?;

        Ok(())
    }

    async fn is_plugin_enabled(&self, plugin_id: &PluginId) -> anyhow::Result<bool> {
        self.db_repository.is_plugin_enabled(&plugin_id.to_string())
            .await
    }

    pub async fn action_shortcuts(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, ActionShortcut>> {
        let DbReadPluginEntrypoint { actions, actions_user_data, .. } = self.db_repository.get_entrypoint_by_id(&plugin_id.to_string(), &entrypoint_id.to_string())
            .await?;

        let actions_user_data: HashMap<_, _> = actions_user_data.into_iter()
            .map(|data| (data.id, (data.key, data.kind)))
            .collect();

        let action_shortcuts = actions.into_iter()
            .map(|action| {
                let id = action.id;

                let shortcut = match actions_user_data.get(&id) {
                    None => {
                        ActionShortcut {
                            key: action.key,
                            kind: match action.kind {
                                DbPluginActionShortcutKind::Main => ActionShortcutKind::Main,
                                DbPluginActionShortcutKind::Alternative => ActionShortcutKind::Alternative,
                            },
                        }
                    }
                    Some((key, kind)) => {
                        ActionShortcut {
                            key: key.to_owned(),
                            kind: match kind {
                                DbPluginActionShortcutKind::Main => ActionShortcutKind::Main,
                                DbPluginActionShortcutKind::Alternative => ActionShortcutKind::Alternative,
                            },
                        }
                    }
                };

                (id, shortcut)
            })
            .collect();

        Ok(action_shortcuts)
    }

    async fn start_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Starting plugin with id: {:?}", plugin_id);

        let plugin_id_str = plugin_id.to_string();

        let plugin = self.db_repository.get_plugin_by_id(&plugin_id_str)
            .await?;

        let inline_view_entrypoint_id = self.db_repository.get_inline_view_entrypoint_id_for_plugin(&plugin_id_str)
            .await?;

        let receiver = self.command_broadcaster.subscribe();
        let data = PluginRuntimeData {
            id: plugin_id,
            uuid: plugin.uuid,
            code: PluginCode { js: plugin.code.js },
            inline_view_entrypoint_id,
            permissions: PluginPermissions {
                environment: plugin.permissions.environment,
                high_resolution_time: plugin.permissions.high_resolution_time,
                network: plugin.permissions.network,
                ffi: plugin.permissions.ffi,
                fs_read_access: plugin.permissions.fs_read_access,
                fs_write_access: plugin.permissions.fs_write_access,
                run_subprocess: plugin.permissions.run_subprocess,
                system: plugin.permissions.system
            },
            command_receiver: receiver,
            db_repository: self.db_repository.clone(),
            search_index: self.search_index.clone(),
            icon_cache: self.icon_cache.clone(),
            frontend_api: self.frontend_api.clone()
        };

        self.start_plugin_runtime(data);

        Ok(())
    }

    async fn stop_plugin(&self, plugin_id: PluginId) {
        tracing::info!(target = "plugin", "Stopping plugin with id: {:?}", plugin_id);

        let data = PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::Stop,
        };

        self.send_command(data)
    }

    fn start_plugin_runtime(&self, data: PluginRuntimeData) {
        let run_status_guard = self.run_status_holder.start_block(data.id.clone());

        tokio::spawn(async {
            start_plugin_runtime(data, run_status_guard)
                .await
                .expect("failed to start plugin runtime")
        });
    }

    fn send_command(&self, command: PluginCommand) {
        self.command_broadcaster.send(command).expect("all respective receivers were closed");
    }

    async fn mark_entrypoint_frecency(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) {
        let result = self.db_repository.mark_entrypoint_frecency(&plugin_id.to_string(), &entrypoint_id.to_string())
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when marking entrypoint frecency {:?}", err)
        }

        self.request_search_index_reload(plugin_id);
    }
}

fn plugin_preference_from_db(value: DbPluginPreference) -> PluginPreference {
    match value {
        DbPluginPreference::Number { default, description } => PluginPreference::Number { default, description },
        DbPluginPreference::String { default, description } => PluginPreference::String { default, description },
        DbPluginPreference::Enum { default, description, enum_values } => {
            let enum_values = enum_values.into_iter()
                .map(|value| PreferenceEnumValue { label: value.label, value: value.value })
                .collect();

            PluginPreference::Enum { default, description, enum_values }
        },
        DbPluginPreference::Bool { default, description } => PluginPreference::Bool { default, description },
        DbPluginPreference::ListOfStrings { default, description } => PluginPreference::ListOfStrings { default, description },
        DbPluginPreference::ListOfNumbers { default, description } => PluginPreference::ListOfNumbers { default, description },
        DbPluginPreference::ListOfEnums { default, enum_values, description } => {
            let enum_values = enum_values.into_iter()
                .map(|value| PreferenceEnumValue { label: value.label, value: value.value })
                .collect();

            PluginPreference::ListOfEnums { default, enum_values, description }
        },
    }
}

fn plugin_preference_user_data_to_db(value: PluginPreferenceUserData) -> DbPluginPreferenceUserData {
    match value {
        PluginPreferenceUserData::Number { value } => DbPluginPreferenceUserData::Number { value },
        PluginPreferenceUserData::String { value } => DbPluginPreferenceUserData::String { value },
        PluginPreferenceUserData::Enum { value } => DbPluginPreferenceUserData::Enum { value },
        PluginPreferenceUserData::Bool { value } => DbPluginPreferenceUserData::Bool { value },
        PluginPreferenceUserData::ListOfStrings { value } => DbPluginPreferenceUserData::ListOfStrings { value },
        PluginPreferenceUserData::ListOfNumbers { value } => DbPluginPreferenceUserData::ListOfNumbers { value },
        PluginPreferenceUserData::ListOfEnums { value } => DbPluginPreferenceUserData::ListOfEnums { value },
    }
}

fn plugin_preference_user_data_from_db(value: DbPluginPreferenceUserData) -> PluginPreferenceUserData {
    match value {
        DbPluginPreferenceUserData::Number { value } => PluginPreferenceUserData::Number { value },
        DbPluginPreferenceUserData::String { value } => PluginPreferenceUserData::String { value },
        DbPluginPreferenceUserData::Enum { value } => PluginPreferenceUserData::Enum { value },
        DbPluginPreferenceUserData::Bool { value } => PluginPreferenceUserData::Bool { value },
        DbPluginPreferenceUserData::ListOfStrings { value, .. } => PluginPreferenceUserData::ListOfStrings { value },
        DbPluginPreferenceUserData::ListOfNumbers { value, .. } => PluginPreferenceUserData::ListOfNumbers { value },
        DbPluginPreferenceUserData::ListOfEnums { value, .. } => PluginPreferenceUserData::ListOfEnums { value },
    }
}

