use std::fs;
use std::path::Path;

use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::FromJson;
use identity_iota::credential::Jws;
use identity_iota::document::verifiable::JwsVerificationOptions;
use identity_iota::iota::IotaDocument;
use identity_iota::prelude::Resolver;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwsSignatureOptions;
use identity_iota::storage::Storage;
use identity_iota::verification::jws::DecodedJws;

use identity_stronghold::StrongholdStorage;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;

use iota_sdk::client::Client;
use iota_sdk::client::Password;
use tcc::{Config, DBConnector, Output};
use tcc::SQLiteConnector;
use tcc::VariablesConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = VariablesConfig::get();
    let output = Output;

    output.show_welcome_message();

    let app = App::new();


    // Stronghold password.
    let password = Password::from(config.get_value("stronghold_password").to_owned());
    // Stronghold snapshot path.
    let stronghold_path: std::path::PathBuf =
        std::path::PathBuf::from(config.get_value("stronghold_path").to_owned());
    // Sqlite path.
    let sqlite_path : &String = config.get_value("sqlite_path");


    // Use match to handle the result of SQLiteConnector::new
    let sqlite = match SQLiteConnector::new(sqlite_path) {
        Ok(conn) => conn, // Successfully created the connection
        Err(e) => {
            // Handle the error, for example, by printing it
            eprintln!("Failed to connect to the database: {}", e);
            return Err(e); // Propagate the error
        }
    };

    let dids = sqlite.get_stored_dids()?;


    // every time enter is pressed, print "Hello World"
    // let mut input = String::new();
    // while input.trim() != "exit" {
    //     input.clear();
    //     std::io::stdin().read_line(&mut input)?;
    //     for a in &dids {
    //         println!("{:?}", a);
    //     }
    //     output.clear_screen()
    // }



    // return Ok(());

    tcc::create_did_table(&sqlite).expect("Table creation failed");


    // Create a new client instance.

    let client: Client = Client::builder()
        .with_primary_node(config.get_value("api_endpoint"), None)?
        .finish()
        .await?;

    let stronghold = StrongholdSecretManager::builder()
        .password(password.clone())
        .build(stronghold_path.clone())?;

    // Create a `StrongholdStorage`.
    // `StrongholdStorage` creates internally a `SecretManager` that can be
    // referenced to avoid creating multiple instances around the same stronghold snapshot.
    let stronghold_storage = StrongholdStorage::new(stronghold);

    // Create a DID document.
    // let address: Address = get_address_with_funds(
    //     &client,
    //     stronghold_storage.as_secret_manager(),
    //     config.faucet_endpoint(),
    // )
    // .await?;
    // let network_name: NetworkName = client.network_name().await?;
    // let mut document: IotaDocument = IotaDocument::new(&network_name);

    // Create storage for key-ids and JWKs.

    // In this example, the same stronghold file that is used to store
    // key-ids as well as the JWKs.
    let storage = Storage::new(stronghold_storage.clone(), stronghold_storage.clone());

    // Generates a verification method. This will store the key-id as well as the private key
    // in the stronghold file.
    // let fragment = document
    //     .generate_method(
    //         &storage,
    //         JwkMemStore::ED25519_KEY_TYPE,
    //         JwsAlgorithm::EdDSA,
    //         None,
    //         MethodScope::VerificationMethod,
    //     )
    //     .await?;

    // Construct an Alias Output containing the DID document, with the wallet address
    // set as both the state controller and governor.
    // let alias_output: AliasOutput = client.new_did_output(address, document, None).await?;

    // Publish the Alias Output and get the published DID document.
    // let document: IotaDocument = client
    //     .publish_did_output(stronghold_storage.as_secret_manager(), alias_output)
    //     .await?;

    // println!("{}", &document.to_json().unwrap());

    // save the document to a file
    let did_file_issuer = "issuer.did";

    let doc = if Path::new("issuer.did").exists() {
        // Load DID from file
        let did_data = fs::read_to_string(did_file_issuer)?;
        IotaDocument::from_json(&did_data)?
    } else {
        panic!("DID file not found");
    };

    // Resolve the published DID Document.
    let mut resolver = Resolver::<IotaDocument>::new();
    resolver.attach_iota_handler(client.clone());
    let resolved_document: IotaDocument = resolver.resolve(doc.id()).await?;

    sqlite.save_did_document(&resolved_document, String::from("Felipe"))?;

    return Ok(());



    // Retrieve the verification method fragment.
    let fragment = tcc::extract_kid(&resolved_document)?;

    println!("{}", fragment);
    // print resolved document
    println!("{:#}", resolved_document);

    // Sign data with the created verification method.
    let data = b"test_data";
    let jws: Jws = resolved_document
        .create_jws(&storage, &fragment, data, &JwsSignatureOptions::default())
        .await?;

    // Verify Signature.
    let decoded_jws: DecodedJws = resolved_document.verify_jws(
        &jws,
        None,
        &EdDSAJwsVerifier::default(),
        &JwsVerificationOptions::default(),
    )?;

    assert_eq!(
        String::from_utf8_lossy(decoded_jws.claims.as_ref()),
        "test_data"
    );

    Ok(())
}
