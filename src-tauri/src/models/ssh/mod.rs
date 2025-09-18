pub mod group;
pub mod profile;

pub use group::{CreateSSHGroupRequest, DeleteGroupAction, SSHGroup, UpdateSSHGroupRequest};
pub use profile::{
    AuthData, CreateSSHProfileRequest, ProxyConfig, ProxyType, SSHProfile,
    UpdateSSHProfileRequest,
};