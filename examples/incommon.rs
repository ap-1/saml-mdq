use saml_mdq::MdqClient;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let client = MdqClient::builder("https://mdq.incommon.org").build()?;

    let entity_id = "https://login.cmu.edu/idp/shibboleth";

    println!("Fetching metadata for: {entity_id}");

    match client.fetch_entity(entity_id).await {
        Ok(metadata) => {
            println!("Entity ID: {:?}", metadata.entity_id);

            if let Some(idp_descriptors) = &metadata.idp_sso_descriptors {
                for idp in idp_descriptors {
                    for sso in &idp.single_sign_on_services {
                        println!("SSO endpoint: {:?}", sso.location);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    Ok(())
}
