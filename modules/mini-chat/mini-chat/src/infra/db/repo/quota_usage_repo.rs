use async_trait::async_trait;
use modkit_db::secure::{
    DBRunner, SecureEntityExt, SecureInsertExt, SecureOnConflict, SecureUpdateExt,
};
use modkit_security::AccessScope;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveEnum, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::repos::{IncrementReserveParams, SettleParams};
use crate::infra::db::entity::quota_usage::{
    ActiveModel, Column, Entity as QuotaUsageEntity, Model as QuotaUsageModel,
};

pub struct QuotaUsageRepository;

#[async_trait]
impl crate::domain::repos::QuotaUsageRepository for QuotaUsageRepository {
    async fn increment_reserve<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        params: IncrementReserveParams,
    ) -> Result<(), DomainError> {
        let now = OffsetDateTime::now_utc();
        let id = Uuid::new_v4();

        let am = ActiveModel {
            id: Set(id),
            tenant_id: Set(params.tenant_id),
            user_id: Set(params.user_id),
            period_type: Set(params.period_type),
            period_start: Set(params.period_start),
            bucket: Set(params.bucket),
            spent_credits_micro: Set(0),
            reserved_credits_micro: Set(params.amount_micro),
            calls: Set(0),
            input_tokens: Set(0),
            output_tokens: Set(0),
            file_search_calls: Set(0),
            web_search_calls: Set(0),
            rag_retrieval_calls: Set(0),
            image_inputs: Set(0),
            image_upload_bytes: Set(0),
            updated_at: Set(now),
        };

        // ON CONFLICT: increment reserved_credits_micro and refresh updated_at.
        let on_conflict = SecureOnConflict::<QuotaUsageEntity>::columns([
            Column::TenantId,
            Column::UserId,
            Column::PeriodType,
            Column::PeriodStart,
            Column::Bucket,
        ])
        .value(
            Column::ReservedCreditsMicro,
            Expr::col(Column::ReservedCreditsMicro).add(Expr::value(params.amount_micro)),
        )?
        .value(Column::UpdatedAt, Expr::value(now))?;

        QuotaUsageEntity::insert(am)
            .secure()
            .scope_unchecked(scope)?
            .on_conflict(on_conflict)
            .exec(runner)
            .await?;

        Ok(())
    }

    async fn settle<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        params: SettleParams,
    ) -> Result<(), DomainError> {
        let now = OffsetDateTime::now_utc();

        // Determine if token telemetry should be updated (only for `total` bucket).
        let is_total = params.bucket == "total";
        let (input_delta, output_delta) = if is_total {
            (
                params.input_tokens.unwrap_or(0),
                params.output_tokens.unwrap_or(0),
            )
        } else {
            (0, 0)
        };

        QuotaUsageEntity::update_many()
            .col_expr(
                Column::ReservedCreditsMicro,
                Expr::col(Column::ReservedCreditsMicro)
                    .sub(Expr::value(params.reserved_credits_micro)),
            )
            .col_expr(
                Column::SpentCreditsMicro,
                Expr::col(Column::SpentCreditsMicro).add(Expr::value(params.actual_credits_micro)),
            )
            .col_expr(
                Column::Calls,
                Expr::col(Column::Calls).add(Expr::value(1i32)),
            )
            .col_expr(
                Column::InputTokens,
                Expr::col(Column::InputTokens).add(Expr::value(input_delta)),
            )
            .col_expr(
                Column::OutputTokens,
                Expr::col(Column::OutputTokens).add(Expr::value(output_delta)),
            )
            .col_expr(Column::UpdatedAt, Expr::value(now))
            .filter(
                Condition::all()
                    .add(Column::TenantId.eq(params.tenant_id))
                    .add(Column::UserId.eq(params.user_id))
                    .add(Column::PeriodType.eq(params.period_type.into_value()))
                    .add(Column::PeriodStart.eq(params.period_start))
                    .add(Column::Bucket.eq(params.bucket)),
            )
            .secure()
            .scope_with(scope)
            .exec(runner)
            .await?;

        Ok(())
    }

    async fn find_bucket_rows<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<QuotaUsageModel>, DomainError> {
        Ok(QuotaUsageEntity::find()
            .filter(
                Condition::all()
                    .add(Column::TenantId.eq(tenant_id))
                    .add(Column::UserId.eq(user_id)),
            )
            .secure()
            .scope_with(scope)
            .all(runner)
            .await?)
    }
}
