use crate::models::base::BaseModel;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SyncStrategy {
    /// Local changes always win (default)
    LastWriteWins,
    /// Remote changes always win
    FirstWriteWins,
    /// Require manual conflict resolution
    ManualResolve,
}

#[allow(dead_code)]
impl SyncStrategy {
    /// Resolve conflict between two models using the strategy
    pub fn resolve_conflict<T>(&self, local: &T, remote: &T) -> SyncResolution
    where
        T: HasBaseModel,
    {
        match self {
            SyncStrategy::LastWriteWins => {
                if local.base_model().updated_at > remote.base_model().updated_at {
                    SyncResolution::UseLocal
                } else {
                    SyncResolution::UseRemote
                }
            }
            SyncStrategy::FirstWriteWins => {
                if local.base_model().created_at < remote.base_model().created_at {
                    SyncResolution::UseLocal
                } else {
                    SyncResolution::UseRemote
                }
            }
            SyncStrategy::ManualResolve => SyncResolution::RequiresManualResolution,
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum SyncResolution {
    UseLocal,
    UseRemote,
    RequiresManualResolution,
}

#[allow(dead_code)]
pub trait HasBaseModel {
    fn base_model(&self) -> &BaseModel;
}

impl Default for SyncStrategy {
    fn default() -> Self {
        Self::LastWriteWins
    }
}
