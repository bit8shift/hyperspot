use std::sync::Arc;

use authz_resolver_sdk::PolicyEnforcer;
use modkit_macros::domain_model;

use crate::domain::repos::QuotaUsageRepository;

use super::DbProvider;

/// Service handling quota tracking and enforcement.
#[domain_model]
pub struct QuotaService<QR: QuotaUsageRepository> {
    _db: Arc<DbProvider>,
    _repo: Arc<QR>,
    _enforcer: PolicyEnforcer,
}

impl<QR: QuotaUsageRepository> QuotaService<QR> {
    pub(crate) fn new(db: Arc<DbProvider>, repo: Arc<QR>, enforcer: PolicyEnforcer) -> Self {
        Self {
            _db: db,
            _repo: repo,
            _enforcer: enforcer,
        }
    }
}
