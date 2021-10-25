use std::error::Error;

use cadence_json::ValueRef;

use flow_sdk::prelude::TonicHyperFlowClient;
use flow_sdk::access::ExecuteScriptAtLatestBlockRequest;

const SIMPLE_SCRIPT: &str = "
    pub fun main(a: Int): Int {
        return a + 10
    }
";

const COMPLEX_SCRIPT: &str = "
    pub struct User {
        pub var balance: UFix64
        pub var address: Address
        pub var name: String

        init(name: String, address: Address, balance: UFix64) {
            self.name = name
            self.address = address
            self.balance = balance
        }
    }

    pub fun main(name: String): User {
        return User(
            name: name,
            address: 0x1,
            balance: 10.0
        )
    }
";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet()?;
    client.ping().await?;

    let ret = client
        .send(ExecuteScriptAtLatestBlockRequest {
            script: SIMPLE_SCRIPT,
            arguments: [ValueRef::Int(cadence_json::BigInt::from(32))],
        })
        .await?
        .parse()?;

    println!("{:#?}", ret);

    let ret = client
        .send(ExecuteScriptAtLatestBlockRequest {
            script: COMPLEX_SCRIPT,
            arguments: [ValueRef::String("John Doe")],
        })
        .await?
        .parse()?;

    println!("{:#?}", ret);

    Ok(())
}
