use crate::{
    classify_risk, Approval, ApprovalKind, ApprovalOption, Decision, PermissionProfile, Risk,
};
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn settings(profile: PermissionProfile) -> (&'static str, &'static str) {
    match profile {
        PermissionProfile::Strict => ("untrusted", "read-only"),
        PermissionProfile::AskForWrites => ("untrusted", "workspace-write"),
    }
}
pub(super) fn approval(id: String, method: &str, params: &Value, cwd: &Path) -> Approval {
    let kind = match method {
        "item/commandExecution/requestApproval" | "execCommandApproval" => ApprovalKind::Command,
        "item/fileChange/requestApproval" | "applyPatchApproval" => ApprovalKind::FileChange,
        "item/tool/requestUserInput" | "mcpServer/elicitation/request" => ApprovalKind::Question,
        _ => ApprovalKind::Permission,
    };
    let detail = params.to_string();
    let risk = if kind == ApprovalKind::Permission {
        Risk::High
    } else {
        classify_risk(&detail, cwd)
    };
    Approval {
        id,
        kind,
        title: method.to_owned(),
        detail,
        risk,
        options: vec![
            ApprovalOption {
                id: "allow".into(),
                label: "Allow".into(),
            },
            ApprovalOption {
                id: "deny".into(),
                label: "Deny".into(),
            },
        ],
    }
}
pub(super) fn response(
    method: &str,
    params: &Value,
    decision: Decision,
) -> Result<Value, crate::BridgeError> {
    if method == "item/permissions/requestApproval" {
        return match decision {
            Decision::Allow | Decision::AllowForSession => Ok(
                json!({"permissions":params.get("permissions").cloned().unwrap_or(json!({})),"scope":if matches!(decision, Decision::AllowForSession) {"session"} else {"turn"}}),
            ),
            Decision::Deny => Ok(json!({"permissions":{},"scope":"turn"})),
            _ => Err(crate::BridgeError::InvalidArgument),
        };
    }
    if method == "item/tool/requestUserInput" {
        let Decision::Answer(answer) = decision else {
            return Err(crate::BridgeError::InvalidArgument);
        };
        let questions = params
            .get("questions")
            .and_then(Value::as_array)
            .ok_or(crate::BridgeError::Protocol)?;
        if questions.len() != 1 {
            return Err(crate::BridgeError::Unsupported);
        }
        let id = questions[0]
            .get("id")
            .and_then(Value::as_str)
            .ok_or(crate::BridgeError::Protocol)?;
        return Ok(json!({"answers":{id:{"answers":[answer]}}}));
    }
    if method == "mcpServer/elicitation/request" {
        return Err(crate::BridgeError::Unsupported);
    }
    let value = match decision {
        Decision::Allow => "accept",
        Decision::AllowForSession => "acceptForSession",
        Decision::Deny => "decline",
        Decision::Answer(_) => return Err(crate::BridgeError::InvalidArgument),
    };
    Ok(json!({"decision":value}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_and_approval_response_values() {
        assert_eq!(
            settings(PermissionProfile::Strict),
            ("untrusted", "read-only")
        );
        assert_eq!(
            settings(PermissionProfile::AskForWrites),
            ("untrusted", "workspace-write")
        );
        for (decision, expected) in [
            (Decision::Allow, "accept"),
            (Decision::AllowForSession, "acceptForSession"),
            (Decision::Deny, "decline"),
        ] {
            assert_eq!(
                response(
                    "item/commandExecution/requestApproval",
                    &json!({}),
                    decision
                )
                .unwrap(),
                json!({"decision":expected})
            );
        }
        assert_eq!(
            response(
                "item/permissions/requestApproval",
                &json!({"permissions":{"network":{"enabled":true}}}),
                Decision::Deny
            )
            .unwrap(),
            json!({"permissions":{},"scope":"turn"})
        );
        assert_eq!(
            response(
                "item/tool/requestUserInput",
                &json!({"questions":[{"id":"q1"}]}),
                Decision::Answer("yes".into())
            )
            .unwrap(),
            json!({"answers":{"q1":{"answers":["yes"]}}})
        );
    }
}
