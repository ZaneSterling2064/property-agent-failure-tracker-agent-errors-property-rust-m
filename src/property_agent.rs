use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Routine,
    Urgent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentState {
    Current,
    Missing,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MaintenanceRequest {
    pub request_id: String,
    pub property_id: String,
    pub summary: String,
    pub urgency: Urgency,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TenantDocument {
    pub tenant_id: String,
    pub state: DocumentState,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InspectionReminder {
    pub inspection_id: String,
    pub due_in_days: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AgentInput {
    pub maintenance: MaintenanceRequest,
    pub tenant_document: TenantDocument,
    pub inspection: InspectionReminder,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDecision {
    DispatchAndRemind,
    RequestDocument,
    ScheduleInspection,
    QueueMaintenance,
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("request_id must not be empty")]
    MissingRequestId,
    #[error("inspection due_in_days is outside the scheduling window")]
    InvalidInspectionWindow,
}

pub fn decide(input: &AgentInput) -> Result<AgentDecision, AgentError> {
    if input.maintenance.request_id.trim().is_empty() {
        return Err(AgentError::MissingRequestId);
    }
    if !(-30..=365).contains(&input.inspection.due_in_days) {
        return Err(AgentError::InvalidInspectionWindow);
    }

    if matches!(input.maintenance.urgency, Urgency::Urgent) {
        return Ok(AgentDecision::DispatchAndRemind);
    }
    if matches!(input.tenant_document.state, DocumentState::Missing) {
        return Ok(AgentDecision::RequestDocument);
    }
    if input.inspection.due_in_days <= 7 {
        return Ok(AgentDecision::ScheduleInspection);
    }
    Ok(AgentDecision::QueueMaintenance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urgent_maintenance_wins_when_a_document_is_missing() {
        let input = AgentInput {
            maintenance: MaintenanceRequest {
                request_id: "mx-1042".into(),
                property_id: "elm-17".into(),
                summary: "Water entering electrical closet".into(),
                urgency: Urgency::Urgent,
            },
            tenant_document: TenantDocument {
                tenant_id: "tenant-88".into(),
                state: DocumentState::Missing,
            },
            inspection: InspectionReminder {
                inspection_id: "inspection-31".into(),
                due_in_days: 2,
            },
        };

        assert_eq!(decide(&input).unwrap(), AgentDecision::DispatchAndRemind);
    }
}
