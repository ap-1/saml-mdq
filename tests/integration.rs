use saml_mdq::{MdqCache, MdqClient, MdqError, hash_entity_id};
use std::time::Duration;

const TEST_ENTITY_ID: &str = "https://login.cmu.edu/idp/shibboleth";

#[tokio::test]
async fn fetch_from_incommon() {
    let client = MdqClient::builder("https://mdq.incommon.org")
        .build()
        .unwrap();

    let metadata = client
        .fetch_entity(TEST_ENTITY_ID)
        .await
        .expect("should fetch CMU metadata");

    assert_eq!(metadata.entity_id.as_deref(), Some(TEST_ENTITY_ID));
    assert!(!metadata.contact_person.is_empty());
}

#[tokio::test]
async fn entity_not_found() {
    let client = MdqClient::builder("https://mdq.incommon.org")
        .build()
        .unwrap();

    let result = client
        .fetch_entity("https://nonexistent.example.org/does-not-exist")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn cached_fetch_returns_same_result() {
    let cache = MdqCache::new(100, Duration::from_secs(300));
    let client = MdqClient::builder("https://mdq.incommon.org")
        .cache(cache)
        .build()
        .unwrap();

    let first = client.fetch_entity(TEST_ENTITY_ID).await.unwrap();
    let second = client.fetch_entity(TEST_ENTITY_ID).await.unwrap();

    assert_eq!(first.entity_id, second.entity_id);
}

#[test]
fn hash_known_entity_id() {
    let hash = hash_entity_id(TEST_ENTITY_ID);
    // Verified against: printf "https://login.cmu.edu/idp/shibboleth" | sha1sum
    assert_eq!(hash, "eae8d5aaf1ba1a6f08f0c66bb31b147974bd7560");
}

// InCommon MDQ production signing certificate downloaded from
// https://md.incommon.org/certs/inc-md-cert-mdq.pem and converted to DER via
// openssl x509 -in inc-md-cert-mdq.pem -outform DER -out inc-md-cert-mdq.der
const INCOMMON_MDQ_CERT_DER: &[u8] = include_bytes!("certs/inc-md-cert-mdq.der");

#[tokio::test]
async fn fetch_with_valid_signing_cert() {
    let client = MdqClient::builder("https://mdq.incommon.org")
        .signing_cert(INCOMMON_MDQ_CERT_DER.to_vec())
        .build()
        .unwrap();

    let metadata = client
        .fetch_entity(TEST_ENTITY_ID)
        .await
        .expect("should fetch and verify signed metadata");

    assert_eq!(metadata.entity_id.as_deref(), Some(TEST_ENTITY_ID));
}

#[tokio::test]
async fn fetch_with_wrong_signing_cert_fails() {
    // Self-signed DER cert that is not the InCommon MDQ signing cert
    let wrong_cert = include_bytes!("certs/wrong-cert.der");

    let client = MdqClient::builder("https://mdq.incommon.org")
        .signing_cert(wrong_cert.to_vec())
        .build()
        .unwrap();

    let result = client.fetch_entity(TEST_ENTITY_ID).await;

    assert!(
        matches!(&result, Err(MdqError::SignatureError(_))),
        "expected SignatureError, got: {result:?}"
    );
}
