use uuid::Uuid;

use crate::domain::errors::{JabeXError, JabeXResult};
use crate::domain::profile::Profile;
use crate::infrastructure::registry::{ProfileRegistry, INITIAL_PROFILE_SETUP_KEY};

pub struct ProfileService {
    registry: ProfileRegistry,
}

impl ProfileService {
    pub fn new(registry: ProfileRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &ProfileRegistry {
        &self.registry
    }

    pub fn list(&self) -> JabeXResult<Vec<Profile>> {
        self.registry.list_profiles()
    }

    pub fn create(&self, display_name: String, avatar: Option<String>) -> JabeXResult<Profile> {
        self.registry.create_profile(display_name, avatar)
    }

    pub fn update(
        &self,
        id: Uuid,
        display_name: Option<String>,
        avatar: Option<Option<String>>,
    ) -> JabeXResult<Profile> {
        self.registry.update_profile(id, display_name, avatar)
    }

    pub fn delete(&self, id: Uuid) -> JabeXResult<()> {
        self.registry.soft_delete_profile(id)
    }

    pub fn get_active(&self) -> JabeXResult<Profile> {
        self.registry.get_active_profile()
    }

    pub fn set_active(&self, id: Uuid) -> JabeXResult<Profile> {
        self.registry.set_active_profile(id)
    }

    pub fn is_initial_profile_setup_pending(&self) -> JabeXResult<bool> {
        Ok(self
            .registry
            .get_meta(INITIAL_PROFILE_SETUP_KEY)?
            .as_deref()
            == Some("true"))
    }

    /// Rename the active profile and clear the first-launch naming prompt.
    pub fn complete_initial_profile_setup(&self, display_name: String) -> JabeXResult<Profile> {
        let name = display_name.trim().to_string();
        if name.is_empty() {
            return Err(JabeXError::Validation("display name is required".into()));
        }
        let active = self.get_active()?;
        let profile = self.update(active.id, Some(name), None)?;
        self.registry.set_meta(INITIAL_PROFILE_SETUP_KEY, "false")?;
        Ok(profile)
    }
}
