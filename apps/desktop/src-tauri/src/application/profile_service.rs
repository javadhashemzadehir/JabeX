use uuid::Uuid;

use crate::domain::errors::JabeXResult;
use crate::domain::profile::Profile;
use crate::infrastructure::registry::ProfileRegistry;

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
}
