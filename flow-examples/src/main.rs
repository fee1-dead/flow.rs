use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process::Child;
use std::thread::sleep;
use std::time::Duration;

use flow_sdk::algorithms::{self as algo, HashAlgorithm, SignatureAlgorithm};
use flow_sdk::prelude::*;
use rustyline::error::ReadlineError;

mod examples;

pub type ExampleAccount = DefaultAccount<TonicHyperFlowClient, algo::secp256k1::SecretKey>;

#[derive(serde::Deserialize, Debug)]
pub struct AccountKeyConfig {
    #[serde(rename = "type")]
    pub ty: String,
    pub index: u64,
    #[serde(rename = "signatureAlgorithm")]
    pub sign_algo: String,
    #[serde(rename = "hashAlgorithm")]
    pub hash_algo: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct AccountConfig {
    pub address: String,
    pub key: AccountKeyConfig,
}

#[derive(serde::Deserialize, Debug)]
pub struct FlowConfig {
    pub networks: HashMap<String, String>,
    pub accounts: HashMap<String, AccountConfig>,
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    use std::process::Command;

    let json = Path::new("./flow.json");

    if json.exists() {
        std::fs::remove_file(json)?;
    }

    let extra_arguments = if let Ok(args) = env::var("FLOW_CLI_EXTRA_ARGS") {
        args
    } else {
        String::new()
    };

    let mut command = Command::new("flow");

    command
        .arg("emulator")
        .arg("--init")
        .arg("--service-sig-algo")
        .arg(algo::Secp256k1::NAME)
        .arg("-v")
        .arg("-f")
        .arg(json)
        .args(extra_arguments.split(' '));

    struct KillChildGuard(Child);

    impl Drop for KillChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
        }
    }

    let _child = KillChildGuard(command.spawn()?);

    while !json.exists() {
        sleep(Duration::from_millis(100));
    }

    // wait 0.5s for server to start.
    sleep(Duration::from_millis(500));

    let json_file = File::open(json)?;

    let mut cfg: FlowConfig = serde_json::from_reader(json_file).unwrap();

    // println!("{:#?}", cfg);

    let emulator = cfg.networks.remove("emulator").unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _enter = rt.enter();

    let client = TonicHyperFlowClient::connect_shared(format!("http://{}", emulator))?;

    rt.block_on(main_inner(cfg, client))?;

    Ok(())
}

async fn main_inner(
    mut cfg: FlowConfig,
    client: TonicHyperFlowClient,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let acc = cfg.accounts.remove("emulator-account").unwrap();

    let addr = hex::decode(acc.address).unwrap();

    assert_eq!(acc.key.ty, "hex");
    assert_eq!(acc.key.sign_algo, algo::Secp256k1::NAME);
    assert_eq!(acc.key.hash_algo, algo::Sha3::NAME);

    let sec = hex::decode(acc.key.private_key).unwrap();
    let secret_key = algo::secp256k1::SecretKey::from_slice(&sec).unwrap();
    //let public_key = algo::secp256k1::PublicKey::from_secret_key(&algo::secp256k1::Secp256k1::signing_only(), &secret_key);

    //println!("{}", hex::encode(public_key.serialize_uncompressed()));

    let mut account = DefaultAccount::new(client, &addr, secret_key).await?;

    println!("Successfully logged in to the service account!");
    println!("type \"help\" for help");

    let mut rl = rustyline::Editor::<()>::new();

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let mut args = line.split_whitespace();
                match args.next() {
                    Some("help") => match args.next() {
                        Some(cmd) => help_command(cmd),
                        None => show_help(),
                    },
                    Some("list") => {
                        let mut table = comfy_table::Table::new();
                        table.set_header(["Example Name", "Arguments", "Description"]);
                        for example in &examples::EXAMPLES {
                            table.add_row([example.name, example.arguments, example.description]);
                        }
                        println!("{}", table);
                    }
                    Some("run") => match args.next() {
                        Some(example) => match examples::EXAMPLES_BY_NAME.get(example) {
                            Some(example) => {
                                println!("Running example {}...", example.name);
                                if let Err(e) = (example.f)(&mut account, &mut args).await {
                                    println!("Error while running example: {}", e);
                                }
                            }
                            None => println!(
                                r#"Example "{}" not found, type "list" to list available examples."#,
                                example
                            ),
                        },
                        None => help_command("run"),
                    },
                    Some("exit") => break,
                    Some(_) => show_help(),
                    None => {}
                }
                rl.add_history_entry(line);
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D");
                break;
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
                break;
            }
            Err(e) => Err(e)?,
        }
    }

    Ok(())
}

fn show_help() {
    println!(
        "\
Usage:
    help              displays this message
    help COMMAND_NAME shows help about a specific command
    list              lists examples available and their usage
    run EXAMPLE_NAME  runs the specified example
    exit              stops the program
"
    );
}

fn help_command(command_name: &str) {
    println!(
        "{}",
        match command_name {
            "run" => "Usage: run COMMAND_NAME  runs the specified example",
            "list" => "Usage: list              lists examples available and their usage",
            "help" => "help help help",
            "exit" => "Z-Z or :q or :wq or even :q!",
            _ => "Invalid command name. Type \"help\" to get a list of available commands.",
        }
    )
}
