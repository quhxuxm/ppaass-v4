use crate::rpc::tunnel::Tunnel;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;
use ppaass_common::{generate_uuid, Encryption};

pub(crate) struct Session {
    pub id: String,
    pub username: String,
    pub agent_encryption: Encryption,
    pub proxy_encryption: Encryption,
    pub tunnels: DashMap<String, Tunnel>,
}

pub(crate) struct CreateSessionDto {
    pub username: String,
    pub agent_encryption: Encryption,
    pub proxy_encryption: Encryption,
}

#[derive(Default)]
pub(crate) struct SessionManager {
    sessions: DashMap<String, Session>,
}

impl SessionManager {
    pub fn create_session(&self, create_session_input: CreateSessionDto) -> String {
        let session_id = generate_uuid();
        let session = Session {
            id: session_id.clone(),
            username: create_session_input.username,
            agent_encryption: create_session_input.agent_encryption,
            proxy_encryption: create_session_input.proxy_encryption,
            tunnels: DashMap::new(),
        };
        self.sessions.insert(session_id.clone(), session);
        session_id
    }

    pub fn fetch_session(&self, session_id: &str) -> Option<Ref<String, Session>> {
        let session = self.sessions.get(session_id)?;
        Some(session)
    }

    pub fn fetch_session_mut(&self, session_id: &str) -> Option<RefMut<String, Session>> {
        let session = self.sessions.get_mut(session_id)?;
        Some(session)
    }
}
