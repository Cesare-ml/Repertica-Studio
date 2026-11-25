//! Typed boundary shared by release IPC (disabled) and the native-only
//! isolated acceptance target. No provider, URL, token or idempotency-key input.
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum Request {
    Status {},
    ReadSettlement { artwork_id: String, scope: String },
    PollSettlement { artwork_id: String, scope: String },
    HistorySettlement { artwork_id: String, scope: String },
    ReviewSettlementAck { artwork_id: String, scope: String, expected_sequence: u64 },
    AckSettlement { artwork_id: String, scope: String, expected_sequence: u64, confirmation_id: String },
    Disconnect {},
    Organizations {},
    SelectOrganization { tenant_id: String },
    ClearSelection {},
    CancelReview {},
    ReviewNew { artwork_id: String, scope: String, revision: String, public_body: String },
    ReviewPending { artwork_id: String, scope: String, revision: String },
    ConfirmPrepare { confirmation_id: String },
    ConfirmReconcile { confirmation_id: String },
}

impl Request {
    /// Release builds have no native remote provider. Even a compromised
    /// renderer cannot turn this gate on or cause network/storage mutations.
    pub(crate) fn disabled(self) -> Result<Value, String> {
        match self {
            Self::Status {} | Self::Disconnect {} => Ok(json!({"status":"disabled"})),
            _ => Err("oauth_provider_disabled".into()),
        }
    }
}

/// Mounted boundary independent of archive, transport and identity provider.
/// Enabling a provider requires a separate reviewed change. This handler
/// cannot open a browser, read secrets or send a network request.
#[tauri::command]
pub fn auction_preview_command(request: Request) -> Result<Value, String> {
    request.disabled()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_dispatch_is_fail_closed_for_every_remote_operation() {
        for request in [
            json!({"action":"readSettlement","artworkId":"a","scope":"s"}),
            json!({"action":"pollSettlement","artworkId":"a","scope":"s"}),
            json!({"action":"historySettlement","artworkId":"a","scope":"s"}),
            json!({"action":"reviewSettlementAck","artworkId":"a","scope":"s","expectedSequence":1}),
            json!({"action":"ackSettlement","artworkId":"a","scope":"s","expectedSequence":1,"confirmationId":"c"}),
            json!({"action":"organizations"}),
            json!({"action":"selectOrganization","tenantId":"tenant-test"}),
            json!({"action":"clearSelection"}), json!({"action":"cancelReview"}),
            json!({"action":"reviewNew","artworkId":"a","scope":"s","revision":"r","publicBody":"{}"}),
            json!({"action":"reviewPending","artworkId":"a","scope":"s","revision":"r"}),
            json!({"action":"confirmPrepare","confirmationId":"c"}),
            json!({"action":"confirmReconcile","confirmationId":"c"}),
        ] {
            let cmd: Request = serde_json::from_value(request).unwrap();
            assert_eq!(auction_preview_command(cmd).unwrap_err(), "oauth_provider_disabled");
        }
        for action in ["status","disconnect"] {
            let cmd: Request = serde_json::from_value(json!({"action":action})).unwrap();
            assert_eq!(auction_preview_command(cmd).unwrap(), json!({"status":"disabled"}));
        }
    }

    #[test]
    fn wire_rejects_destination_token_keys_and_unknown_operations() {
        for invalid in [
            json!({"action":"readSettlement","artworkId":"a","scope":"s","auctionId":"arbitrary"}),
            json!({"action":"readSettlement","artworkId":"a","scope":"s","ownerSubject":"repertica:1"}),
            json!({"action":"readSettlement","artworkId":"a","scope":"s","accessToken":"x"}),
            json!({"action":"status","provider":"isolated"}),
            json!({"action":"organizations","url":"http://127.0.0.1:1234"}),
            json!({"action":"selectOrganization","tenantId":"tenant-test","accessToken":"x"}),
            json!({"action":"confirmPrepare","confirmationId":"c","idempotencyKey":"x"}),
            json!({"action":"confirmReconcile","confirmationId":"c","publicBody":"{}"}),
            json!({"action":"reviewPending","artworkId":"a","scope":"s","revision":"r","tenantId":"other"}),
            json!({"action":"pollSettlement","artworkId":"a","scope":"s","auctionId":"other"}),
            json!({"action":"historySettlement","artworkId":"a","scope":"s","afterSequence":99}),
            json!({"action":"ackSettlement","artworkId":"a","scope":"s"}),
            json!({"action":"ackSettlement","artworkId":"a","scope":"s","expectedSequence":1}),
            json!({"action":"reviewSettlementAck","artworkId":"a","scope":"s","expectedSequence":1,"url":"http://127.0.0.1"}),
            json!({"action":"ackSettlement","artworkId":"a","scope":"s","expectedSequence":1,"resultId":"other"}),
            json!({"action":"connect","provider":"pilot"}),
        ] {
            assert!(serde_json::from_value::<Request>(invalid).is_err());
        }
    }
}
