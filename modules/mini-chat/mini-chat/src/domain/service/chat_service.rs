use std::sync::Arc;

use authz_resolver_sdk::PolicyEnforcer;
use modkit_macros::domain_model;

use crate::domain::repos::{ChatRepository, MessageRepository, ThreadSummaryRepository};

use super::DbProvider;

/// Service handling chat CRUD and message listing operations.
#[domain_model]
pub struct ChatService<MR: MessageRepository, CR: ChatRepository> {
    _db: Arc<DbProvider>,
    _chat_repo: Arc<CR>,
    _message_repo: Arc<MR>,
    _thread_summary_repo: Arc<dyn ThreadSummaryRepository>,
    _enforcer: PolicyEnforcer,
}

impl<MR: MessageRepository, CR: ChatRepository> ChatService<MR, CR> {
    pub(crate) fn new(
        db: Arc<DbProvider>,
        chat_repo: Arc<CR>,
        message_repo: Arc<MR>,
        thread_summary_repo: Arc<dyn ThreadSummaryRepository>,
        enforcer: PolicyEnforcer,
    ) -> Self {
        Self {
            _db: db,
            _chat_repo: chat_repo,
            _message_repo: message_repo,
            _thread_summary_repo: thread_summary_repo,
            _enforcer: enforcer,
        }
    }
}
