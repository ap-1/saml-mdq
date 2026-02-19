use crate::cache::MdqCache;
use crate::error::{MdqError, Result};
use crate::hash::encode_entity_id;
use reqwest::Client;
use samael::crypto::{CertificateDer, Crypto, CryptoProvider};
use samael::metadata::EntityDescriptor;
use std::time::Duration;

pub struct MdqClient {
    base_url: String,
    http: Client,
    cache: Option<MdqCache>,
    signing_cert: Option<Vec<u8>>,
}

pub struct MdqClientBuilder {
    base_url: String,
    cache: Option<MdqCache>,
    signing_cert: Option<Vec<u8>>,
    timeout: Duration,
}

impl MdqClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            cache: None,
            signing_cert: None,
            timeout: Duration::from_secs(10),
        }
    }

    pub fn cache(mut self, cache: MdqCache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn signing_cert(mut self, cert_der: Vec<u8>) -> Self {
        self.signing_cert = Some(cert_der);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<MdqClient> {
        let http = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(MdqError::Http)?;

        Ok(MdqClient {
            base_url: self.base_url.trim_end_matches('/').to_string(),
            http,
            cache: self.cache,
            signing_cert: self.signing_cert,
        })
    }
}

impl MdqClient {
    pub fn builder(base_url: impl Into<String>) -> MdqClientBuilder {
        MdqClientBuilder::new(base_url)
    }

    /// Fetch metadata for an entity by its entityID.
    ///
    /// The entityID is SHA-1 hashed per the MDQ protocol before querying.
    /// Results are cached if a cache was configured on the builder.
    pub async fn fetch_entity(&self, entity_id: &str) -> Result<EntityDescriptor> {
        if let Some(cache) = &self.cache
            && let Some(cached) = cache.get(entity_id).await
        {
            return Ok(cached);
        }

        let encoded = encode_entity_id(entity_id);
        let url = format!("{}/entities/{}", self.base_url, encoded);

        let response = self.http.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(MdqError::EntityNotFound(entity_id.to_string()));
        }

        if !response.status().is_success() {
            return Err(MdqError::InvalidXml(format!(
                "MDQ server returned status {}",
                response.status()
            )));
        }

        let xml = response.text().await?;

        if let Some(cert_der) = &self.signing_cert {
            let cert = CertificateDer::from(cert_der.clone());
            Crypto::verify_signed_xml(xml.as_bytes(), &cert, Some("ID"))
                .map_err(|e| MdqError::SignatureError(e.to_string()))?;
        }

        let descriptor: EntityDescriptor = samael::metadata::de::from_str(&xml)
            .map_err(|e| MdqError::InvalidXml(e.to_string()))?;

        if let Some(cache) = &self.cache {
            cache
                .insert(entity_id.to_string(), descriptor.clone())
                .await;
        }

        Ok(descriptor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_strips_trailing_slash() {
        let client = MdqClient::builder("https://mdq.example.org/")
            .build()
            .unwrap();
        assert_eq!(client.base_url, "https://mdq.example.org");
    }

    #[test]
    fn builder_preserves_clean_url() {
        let client = MdqClient::builder("https://mdq.example.org")
            .build()
            .unwrap();
        assert_eq!(client.base_url, "https://mdq.example.org");
    }
}
