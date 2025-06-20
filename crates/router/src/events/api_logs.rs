use actix_web::HttpRequest;
pub use common_utils::events::{ApiEventMetric, ApiEventsType};
use common_utils::impl_api_event_type;
use router_env::{tracing_actix_web::RequestId, types::FlowMetric};
use serde::Serialize;
use time::OffsetDateTime;

use super::EventType;
#[cfg(feature = "dummy_connector")]
use crate::routes::dummy_connector::types::{
    DummyConnectorPaymentCompleteRequest, DummyConnectorPaymentConfirmRequest,
    DummyConnectorPaymentRequest, DummyConnectorPaymentResponse,
    DummyConnectorPaymentRetrieveRequest, DummyConnectorRefundRequest,
    DummyConnectorRefundResponse, DummyConnectorRefundRetrieveRequest,
};
use crate::{
    core::payments::PaymentsRedirectResponseData,
    services::{authentication::AuthenticationType, kafka::KafkaMessage},
    types::api::{
        AttachEvidenceRequest, Config, ConfigUpdate, CreateFileRequest, DisputeId, FileId, PollId,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiEvent {
    tenant_id: common_utils::id_type::TenantId,
    merchant_id: Option<common_utils::id_type::MerchantId>,
    api_flow: String,
    created_at_timestamp: i128,
    request_id: String,
    latency: u128,
    status_code: i64,
    #[serde(flatten)]
    auth_type: AuthenticationType,
    request: String,
    user_agent: Option<String>,
    ip_addr: Option<String>,
    url_path: String,
    response: Option<String>,
    error: Option<serde_json::Value>,
    #[serde(flatten)]
    event_type: ApiEventsType,
    hs_latency: Option<u128>,
    http_method: String,
    #[serde(flatten)]
    infra_components: Option<serde_json::Value>,
}

impl ApiEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: common_utils::id_type::TenantId,
        merchant_id: Option<common_utils::id_type::MerchantId>,
        api_flow: &impl FlowMetric,
        request_id: &RequestId,
        latency: u128,
        status_code: i64,
        request: serde_json::Value,
        response: Option<serde_json::Value>,
        hs_latency: Option<u128>,
        auth_type: AuthenticationType,
        error: Option<serde_json::Value>,
        event_type: ApiEventsType,
        http_req: &HttpRequest,
        http_method: &http::Method,
        infra_components: Option<serde_json::Value>,
    ) -> Self {
        Self {
            tenant_id,
            merchant_id,
            api_flow: api_flow.to_string(),
            created_at_timestamp: OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000,
            request_id: request_id.as_hyphenated().to_string(),
            latency,
            status_code,
            request: request.to_string(),
            response: response.map(|resp| resp.to_string()),
            auth_type,
            error,
            ip_addr: http_req
                .connection_info()
                .realip_remote_addr()
                .map(ToOwned::to_owned),
            user_agent: http_req
                .headers()
                .get("user-agent")
                .and_then(|user_agent_value| user_agent_value.to_str().ok().map(ToOwned::to_owned)),
            url_path: http_req.path().to_string(),
            event_type,
            hs_latency,
            http_method: http_method.to_string(),
            infra_components,
        }
    }
}

impl KafkaMessage for ApiEvent {
    fn event_type(&self) -> EventType {
        EventType::ApiLogs
    }

    fn key(&self) -> String {
        self.request_id.clone()
    }
}

impl_api_event_type!(
    Miscellaneous,
    (
        Config,
        CreateFileRequest,
        FileId,
        AttachEvidenceRequest,
        ConfigUpdate
    )
);

#[cfg(feature = "dummy_connector")]
impl_api_event_type!(
    Miscellaneous,
    (
        DummyConnectorPaymentCompleteRequest,
        DummyConnectorPaymentRequest,
        DummyConnectorPaymentResponse,
        DummyConnectorPaymentRetrieveRequest,
        DummyConnectorPaymentConfirmRequest,
        DummyConnectorRefundRetrieveRequest,
        DummyConnectorRefundResponse,
        DummyConnectorRefundRequest
    )
);

#[cfg(feature = "v1")]
impl ApiEventMetric for PaymentsRedirectResponseData {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::PaymentRedirectionResponse {
            connector: self.connector.clone(),
            payment_id: match &self.resource_id {
                api_models::payments::PaymentIdType::PaymentIntentId(id) => Some(id.clone()),
                _ => None,
            },
        })
    }
}

#[cfg(feature = "v2")]
impl ApiEventMetric for PaymentsRedirectResponseData {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::PaymentRedirectionResponse {
            payment_id: self.payment_id.clone(),
        })
    }
}

impl ApiEventMetric for DisputeId {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::Dispute {
            dispute_id: self.dispute_id.clone(),
        })
    }
}

impl ApiEventMetric for PollId {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::Poll {
            poll_id: self.poll_id.clone(),
        })
    }
}
