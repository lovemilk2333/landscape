use landscape_common::flow::ip_mark::WanIpRuleConfig;
use sea_orm::DatabaseConnection;

use super::entity::{DstIpRuleConfigActiveModel, DstIpRuleConfigEntity, DstIpRuleConfigModel};
use crate::DBId;

#[derive(Clone)]
pub struct DstIpRuleRepository {
    db: DatabaseConnection,
}

impl DstIpRuleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

crate::impl_repository!(
    DstIpRuleRepository,
    DstIpRuleConfigModel,
    DstIpRuleConfigEntity,
    DstIpRuleConfigActiveModel,
    WanIpRuleConfig,
    DBId
);

crate::impl_flow_store!(DstIpRuleRepository, DstIpRuleConfigModel, DstIpRuleConfigEntity);
