use std::{io, time::Duration};
use std::thread::sleep;
use std::future::Future;
use actix_web::Error;
use azure_security_keyvault::{KeyvaultClient, KeyClient};
use azure_identity::DefaultAzureCredentialBuilder;
use futures::StreamExt;
use async_std::sync::Arc;
use futures_retry::{RetryPolicy, StreamRetryExt};
use retry;
use warp::filters::header::value;

pub async fn get_credentials() -> Result<String, Error> {
    // Set keyvault url through function itself
    // let keyvault_url = env::var("KEYVAULT_URL").expect("Missing KEYVAULT_URL environment variable.");
    // let certificate_name = env::var("certificate_name").expect("Missing certificate_name environment variable.");

    let keyvault_url = "https://foj-signer.vault.azure.net/certificates/foj-sign/280a9b12eb57499a80dcd5c2ccbb3c98";

    let key_url = "https://foj-signer.vault.azure.net/keys/foj-key/936fb9c0d36b4707bcd25015eba6260c";

    let certificate_name = "foj-sign";
    let key_name = "foj-key";

    let cert_creds = DefaultAzureCredentialBuilder::new()
    .exclude_environment_credential()
    .exclude_azure_cli_credential() // disable using environment variables for credentials (just as an example)
    .build();

    let mut cert_client = KeyvaultClient::new(
        keyvault_url,
        Arc::new(cert_creds),
    ).unwrap().certificate_client();

    let key_creds = DefaultAzureCredentialBuilder::new()
    .exclude_environment_credential()
    .exclude_azure_cli_credential() // disable using environment variables for credentials (just as an example)
    .build();

    let mut key_client = KeyvaultClient::new(
        key_url,
        Arc::new(key_creds),
    ).unwrap().key_client();
    
    let mut retries = 0;
    let max_retries = 30;
    let base_delay_ms = 1250;

    let certificate = loop{
        println!("{} retries", retries);
        match cert_client.get(certificate_name).await {
            Ok(value) => break Ok(value),
            Err(err) => {
                println!("{} retries", retries);
                if retries >= max_retries {
                    break Err(err); // too many retries, fail
                }
                else {
                    println!("{} retries", retries);
                    sleep(Duration::from_millis(base_delay_ms));
                    retries += 1;
                    }
                }
            }
        };

    let key = loop{
        println!("{} retries", retries);
        match key_client.get(key_name).await {
            Ok(value) => break Ok(value),
            Err(err) => {
                println!("{} retries", retries);
                if retries >= max_retries {
                    break Err(err); // too many retries, fail
                }
                else {
                    println!("{} retries", retries);
                    sleep(Duration::from_millis(base_delay_ms));
                    retries += 1;
                    }
                }
            }  
    };

//client.get(certificate_name).await.unwrap(); //.retry(handle_connection_error)
    let unwrapped_cert = certificate.unwrap();
    let unwrapped_key = key.unwrap();
    let key = unwrapped_key.key;
    let secret = unwrapped_cert.cer.secret();
    dbg!(&unwrapped_cert);
    dbg!(&key);
    dbg!(&secret);
    Ok(secret.to_owned())
}





fn handle_connection_error(e: io::Error) -> RetryPolicy<io::Error> {
    match e.kind() {
        _ => RetryPolicy::WaitRetry(Duration::from_millis(5))
    }
}

// /// Represents the types of errors you might encounter
// #[derive(Debug)]
// pub enum Error {
//     Timeout,
//     Credential,
//     Other,
// }

// Async retry function
async fn with_retry<F, Fut>(mut op: F, max_retries: usize, base_delay_ms: u64) -> Result<String, Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<String, Error>>,
{
    let mut retries = 0;
    loop {
        println!("{} retries", retries);
        match op().await {
            Ok(value) => break Ok(value),
            Err(err) => {
                println!("{} retries", retries);
                if retries >= max_retries {
                    break Err(err); // too many retries, fail
                }
                else {
                    println!("{} retries", retries);
                    sleep(Duration::from_millis(base_delay_ms));
                    retries += 1;
                }
            }
                // match err {
                //     // Only retry on specific errors
                //     Error::Timeout => {
                //         let delay = base_delay_ms;
                //         sleep(Duration::from_millis(delay));
                //         retries += 1;
                //     }
                //     // Handle or rethrow other errors
                //     _ => return Err(err),
                // }
            //}
        }
    }
}

pub async fn main() -> Result<String, Error> {
    let result = with_retry(get_credentials, 5, 250).await;
    result
}
     //{

        // Ok(_) => Ok(value),
        // Err(err) => Error::Credential
        // match err {
        //     Error::Timeout => println!("Operation failed after retries due to timeout"),
        //     Error::Credential => println!("Operation failed due to credential error"),
        //     Error::Other => println!("Operation failed due to an unforeseen error"),
        // }
    //}
// }