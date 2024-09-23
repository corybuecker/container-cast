use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use openssl::{error::ErrorStack, hash::MessageDigest, memcmp, pkey::PKey, sign::Signer};
use std::{
    env::{self, VarError},
    error::Error,
    fmt::{Display, Formatter},
};
use tracing::{debug, error};

#[derive(Debug)]
enum WebhookRequestError {
    Environment,
    MissingHeader,
    InvalidSignature,
}

impl From<ErrorStack> for WebhookRequestError {
    fn from(value: ErrorStack) -> Self {
        debug!("{}", value);
        WebhookRequestError::Environment
    }
}

impl From<VarError> for WebhookRequestError {
    fn from(value: VarError) -> Self {
        debug!("{}", value);
        WebhookRequestError::Environment
    }
}

impl Display for WebhookRequestError {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl IntoResponse for WebhookRequestError {
    fn into_response(self: WebhookRequestError) -> Response {
        error!("{}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl Error for WebhookRequestError {}

pub fn router() -> Router {
    Router::new().route("/", post(handler))
}

#[axum::debug_handler]
async fn handler(headers: HeaderMap, body: Bytes) -> Result<Response, WebhookRequestError> {
    debug!("{:#?}", headers);
    debug!("{:#?}", body);

    let webhook_signature = headers
        .get("X-Hub-Signature-256")
        .ok_or(WebhookRequestError::MissingHeader)?;

    let webhook_signature = webhook_signature.as_bytes();

    let computed_signature = computed_signature(&body)?;
    let computed_signature = computed_signature.as_slice();

    debug!("computed: {:#?}", computed_signature);
    debug!("external: {:#?}", webhook_signature);

    if webhook_signature.len() != computed_signature.len() {
        return Err(WebhookRequestError::InvalidSignature);
    }

    let valid_signature = memcmp::eq(webhook_signature, computed_signature);

    debug!("{:#?}", valid_signature);

    Ok(StatusCode::OK.into_response())
    //   let docker = Docker::connect_with_local_defaults().unwrap();
    //   let images = docker.list_images::<String>(None).await;
    //   debug!("{:#?}", images);
    //
    //   let containers = docker.list_containers::<String>(None).await;

    //  debug!("{:#?}", containers);
}

fn computed_signature(body: &[u8]) -> Result<Vec<u8>, WebhookRequestError> {
    let secret = env::var("SECRET")?;
    let key = PKey::hmac(secret.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;

    signer.update(body)?;

    Ok(format!("sha256={}", hex::encode(signer.sign_to_vec()?)).into())
}
