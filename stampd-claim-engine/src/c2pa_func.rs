use blake3;
use c2pa::{Manifest, ManifestStore, ManifestStoreReport, create_signer, SigningAlg, IngredientOptions, Error}; //Ingredient,
use actix_multipart::form::tempfile::TempFile;
use serde_json;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use actix_web::Responder;


/// Generate a blake3 hash over the image in path using a fixed buffer
fn blake3_hash(path: &Path) -> Result<String, c2pa::Error> {
    use std::{fs::File, io::Read};
    // Hash an input incrementally.
    let mut hasher = blake3::Hasher::new();
    const BUFFER_LEN: usize = 1024 * 1024;
    let mut buffer = [0u8; BUFFER_LEN];
    let mut file = File::open(path)?;
    loop {
        let read_count = file.read(&mut buffer)?;
        hasher.update(&buffer[..read_count]);
        if read_count != BUFFER_LEN {
            break;
        }
    }
    let hash = hasher.finalize();
    Ok(hash.to_hex().as_str().to_owned())
}

pub async fn generate_claim(f: Vec<String>) -> Result<(), Error>  {
    // env_logger::init();

    // Defaulting to the default value
    let generator = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("{}", generator);
    //let software_agent = format!("{} {}", "Make Test Images", env!("CARGO_PKG_VERSION"));

    let manifest_path = &f[0];
    let original_path = &f[1];
    let signed_path = &f[2];

    let manifest_json_str = fs::read_to_string(manifest_path).expect("Unable to read file");

    let mut request_manifest = Manifest::from_json(&manifest_json_str).unwrap();
    println!("created manifest");

    // Create a ps256 signer using certs and key files
    let signcert_path = "knowbots_certificate.pem";
    let pkey_path = "knowbots_private.key";
    //let cert_pem = helper::azure_secret_creds().await;
    // let cert_comps = helper::pem_decoder(cert_pem);
    // let signcert_pem = cert_comps[0];
    // let pkey = cert_comps[1];
    
    let signer = create_signer::from_files(signcert_path, pkey_path, SigningAlg::Ps256, None);

    let _ = request_manifest.embed(original_path, signed_path, &*signer.unwrap());

    let _ = ManifestStore::from_file(signed_path.clone());
    // c2pa_functions::sign_files::SignImages::new(request_manifest_config);
    Ok(())
        
    }


pub async fn read_manifest(f: String) -> Result<ManifestStore, Error> {
    let manifest_store_report = ManifestStore::from_file(f).unwrap();
    Ok(manifest_store_report)
}