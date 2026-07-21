use serde::Serialize;
use serde_json::{Value, json};

use crate::{OpenLvError, Session, SessionStateObject, SessionUri};

pub struct Provider {
    session: Session,
}

impl Provider {
    pub(crate) fn new(session: Session) -> Self {
        Self { session }
    }

    pub async fn connect(&self) -> Result<(), OpenLvError> {
        self.session.connect().await
    }

    pub async fn close(&self) -> Result<(), OpenLvError> {
        self.session.close().await
    }

    pub async fn wait_for_link(&self) -> Result<(), OpenLvError> {
        self.session.wait_for_link().await
    }

    pub fn state(&self) -> SessionStateObject {
        self.session.state()
    }

    pub fn uri(&self) -> &SessionUri {
        self.session.uri()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub async fn request<P>(
        &self,
        method: impl Into<String>,
        params: P,
    ) -> Result<Value, OpenLvError>
    where
        P: Serialize,
    {
        let response = self
            .session
            .send(json!({
                "method": method.into(),
                "params": serde_json::to_value(params)?,
            }))
            .await?;

        decode_response(response)
    }
}

fn decode_response(response: Value) -> Result<Value, OpenLvError> {
    let Some(error) = response.get("error").filter(|error| !error.is_null()) else {
        return Ok(response.get("result").cloned().unwrap_or(response));
    };

    let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32_603);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Provider request failed")
        .to_owned();
    let data = error.get("data").cloned();

    Err(OpenLvError::Provider {
        code,
        message,
        data,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::decode_response;
    use crate::OpenLvError;

    #[test]
    fn returns_direct_provider_results() {
        assert_eq!(decode_response(json!("0x1")).unwrap(), json!("0x1"));
    }

    #[test]
    fn unwraps_result_envelopes() {
        assert_eq!(
            decode_response(json!({"result": ["0xabc"]})).unwrap(),
            json!(["0xabc"]),
        );
    }

    #[test]
    fn converts_error_envelopes() {
        let error = decode_response(json!({
            "error": {
                "code": 4001,
                "message": "User rejected the request",
                "data": {"reason": "cancelled"},
            }
        }))
        .unwrap_err();

        assert!(matches!(
            error,
            OpenLvError::Provider {
                code: 4001,
                message,
                data: Some(_),
            } if message == "User rejected the request"
        ));
    }
}
