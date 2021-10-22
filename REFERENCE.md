<br />
<div align="center">
  <a href="">
    <img src="./" alt="Logo" width="300" height="auto">
  </a>
  <p align="center"> <br />
    <a href=""><strong>View on GitHub »</strong></a> <br /><br />
    <a href="https://docs.onflow.org/sdk-guidelines/">SDK Specifications</a> ·
    <a href="">Contribute</a> ·
    <a href="">Report a Bug</a>
  </p>
</div><br />

## Overview 

This reference documents all the methods available in the SDK, and explains in detail how these methods work.
SDKs are open source, and you can use them according to the licence.

The library client specifications can be found here:

// TODO specs here
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]()


## Getting Started

### Installing

To install Rust, refer to [rustup.rs](https://rustup.rs) for instructions. 

### Importing the Library

Add the folllowing to your project's `Cargo.toml`:

```toml
[dependencies]
flow-sdk = "0.1.0"
```

## Connect
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

The library uses gRPC to communicate with the access nodes and it must be configured with correct access node API URL. 

📖 **Access API URLs** can be found [here](https://docs.onflow.org/access-api/#flow-access-node-endpoints). An error will be returned if the host is unreachable.
The Access Nodes APIs hosted by DapperLabs are accessible at:
- Testnet `access.devnet.nodes.onflow.org:9000`
- Mainnet `access.mainnet.nodes.onflow.org:9000`
- Local Emulator `127.0.0.1:3569` 

Example:
```rust
use std::error::Error;
use flow_sdk::client::TonicHyperFlowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet()?;
    client.ping().await?;
}
```

## Querying the Flow Network
After you have established a connection with an access node, you can query the Flow network to retrieve data about blocks, accounts, events and transactions. We will explore how to retrieve each of these entities in the sections below.

### Get Blocks
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Query the network for block by id, height or get the latest block.

📖 **Block ID** is SHA3-256 hash of the entire block payload. This hash is stored as an ID field on any block response object (ie. response from `GetLatestBlock`). 

📖 **Block height** expresses the height of the block on the chain. The latest block height increases by one for every valid block produced.

#### Examples

This example depicts ways to get the latest block as well as any other block by height or ID:

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO link to example
```rust
use std::error::Error;

use flow_sdk::{Block, client::TonicHyperFlowClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::testnet()?;
    client.ping().await?;

    let latest_block: Block = client.latest_block(true).await?.0;

    let block_by_id = client.block_by_id(&latest_block.id).await?.0;

    let block_by_height = client.block_by_height(latest_block.height).await?.0;

    assert_eq!(latest_block, block_by_id);
    assert_eq!(latest_block, block_by_height);

    println!("OK: {:#?}", block);

    Ok(())
}
```
Result output:
```rust
OK: Block {
    id: "1e41b761cfa9ef16ffb61c74d87d503ce34186a185cd376e2d4763554e671f21",
    parent_id: "904fedf25aa7db120304ef2627de63232c90ee7174fee75553ce5ae31c9e3553",
    height: 48759415,
    timestamp: Timestamp {
        seconds: 1634809690,
        nanos: 38007528,
    },
    collection_guarantees: Repeated(
        [],
    ),
    block_seals: Repeated(
        [
            BlockSeal {
                block_id: "153dd17d10d51a0a7a53ffff8aa29d91c7c0ebfe8ba8e06db6d080f26b99b3eb",
                execution_receipt_id: "2828d4c14255912a1517b67afcb85cb44335c2c243678d4d5c8598467155c8f6",
                execution_receipt_signatures: [],
                result_approval_signatures: [],
            },
            BlockSeal {
                block_id: "4dc848e9d2e26c3f84e8ff3aa2377903695f2b0e47385232c908d46f9a94eba4",
                execution_receipt_id: "d046feb80d276e02b1bd96992f90b543b86d1cf667a0d3f464d683cd75322dc6",
                execution_receipt_signatures: [],
                result_approval_signatures: [],
            },
        ],
    ),
    signatures: [
        "8296192b30da2fd82bd96af60b93c448be6ee24edd0a4673fe2072d7bed7f0b7dbf6b5f2443bd5b2bfa6742bce5b6f0db7695d407589f35d9da858b879c120bcfbcec56e708e8e82ff91fde58bb646ec59440714c897399d3ba99f54ce2d199a",
    ],
}
```

### Get Account
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Retrieve any account from Flow network's latest block or from a specified block height.

📖 **Account address** is a unique account identifier. Be mindful about the `0x` prefix, you should use the prefix as a default representation but be careful and safely handle user inputs without the prefix.

An account includes the following data:
- Address: the account address.
- Balance: balance of the account.
- Contracts: list of contracts deployed to the account.
- Keys: list of keys associated with the account.

#### Examples
Example depicts ways to get an account at the latest block and at a specific block height:

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example link
```rust
use std::{
    error::Error,
    io::{stdin, BufRead},
};

use cadence_json::AddressOwned;
use flow_sdk::client::TonicHyperFlowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();

    println!("Enter the account's address:");

    stdin.read_line(&mut buf)?;

    let addr = buf.trim();

    let address: AddressOwned = addr.parse()?;
    let mut net = TonicHyperFlowClient::mainnet()?;

    let account = net.account_at_latest_block(&address.data).await?.account;

    let latest_block_height = net.latest_block_header(true).await?.0.height;

    let account1 = net.account_at_block_height(&address.data, latest_block_height).await?.account;

    println!("{:#?}", account);

    assert_eq!(account, account1);

    Ok(())
}

```
Result output:
```rust
Enter the account's address:
0x9e06eebf494e2d78
Account {
    address: 0x9e06eebf494e2d78,
    balance: 9411868000,
    code: [],
    keys: [
        AccountKey {
            index: 0,
            public_key: d5932abf2a4d22fe9fbf312ce44e984b0c6486cd221e9ea42d1fed48e8b685bdb7daf61f20ad560e2b5938958d48b9daf3fd9ae05608e012dd64f47453cb9ca2,
            sign_algo: 2,
            hash_algo: 3,
            weight: 1000,
            sequence_number: 35875,
            revoked: false,
        },
    ],
    contracts: {},
}
```


### Get Transactions
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Retrieve transactions from the network by providing a transaction ID. After a transaction has been submitted, you can also get the transaction result to check the status.

📖 **Transaction ID** is a hash of the encoded transaction payload and can be calculated before submitting the transaction to the network.

⚠️ The transaction ID provided must be from the current spork.

📖 **Transaction status** represents the state of transaction in the blockchain. Status can change until is finalized.

| Status      | Final | Description |
| ----------- | ----------- | ----------- |
|   UNKNOWN    |    ❌   |   The transaction has not yet been seen by the network  |
|   PENDING    |    ❌   |   The transaction has not yet been included in a block   |
|   FINALIZED    |   ❌     |  The transaction has been included in a block   |
|   EXECUTED    |   ❌    |   The transaction has been executed but the result has not yet been sealed  |
|   SEALED    |    ✅    |   The transaction has been executed and the result is sealed in a block  |
|   EXPIRED    |   ✅     |  The transaction reference block is outdated before being executed    |


**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example link
```rust
use std::error::Error;

use flow_sdk::{client::TonicHyperFlowClient, Block};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::testnet()?;
    client.ping().await?;

    // traverse the blocks until we find collection guarantees
    let mut latest_block: Block = client.latest_block(true).await?.0;

    let collection_guarrantee = loop {
        if latest_block.collection_guarantees.is_empty() {
            // Go to the next block
            latest_block = client.block_by_id(&latest_block.parent_id).await?.0;
        } else {
            break latest_block.collection_guarantees.pop().unwrap();
        }
    };

    let collection = client
        .collection_by_id(&collection_guarrantee.collection_id)
        .await?
        .collection;

    for transaction_id in collection.transactions.iter() {
        let txn = client.transaction_by_id(transaction_id).await?.transaction;
        println!("{:#?}", txn);
        for argument in txn.parse_arguments() {
            println!("Found a cadence argument in the wild: {:#?}", argument?);
        }
    }

    Ok(())
}
```
Example output:
```rust
Transaction {
    script: "\nimport NonFungibleToken from 0x631e88ae7f1d7c20\nimport Vouchers from 0xe94a6e229293f196\ntransaction(recipients: [Address], rewards: {Address: [UInt64]}) {\n    let adminCollection: &Vouchers.Collection\n    let recipientCollections: {Address: &{Vouchers.CollectionPublic}}\n    prepare(signer: AuthAccount) {\n        log(\"--voucher send to addresses activated--\")\n        self.recipientCollections = {}\n        // get the recipients public account object\n        for address in recipients {\n            self.recipientCollections[address] = getAccount(address).getCapability(Vouchers.CollectionPublicPath).borrow<&{Vouchers.CollectionPublic}>()\n                ?? panic(\"Could not borrow a reference to the recipient's collection\")\n        }\n\n        // borrow a reference to the signer's NFT collection\n        self.adminCollection = signer.borrow<&Vouchers.Collection>(from: Vouchers.CollectionStoragePath)\n            ?? panic(\"Could not borrow a reference to the signer's collection\")\n    }\n\n    execute {\n        log(\"rewarding recipients\")\n        for address in recipients {\n            log(\"user address: \" + address);\n            if (rewards[address] != nil) {\n                let rewards = rewards[address] as! [UInt64]\n                for reward in rewards {\n                    log(\"-- reward: \" + reward)\n                    self.recipientCollections[address]!.deposit(token: <- self.adminCollection.withdraw(withdrawID: reward))\n                }\n            }\n        }\n    }\n}\n",
    arguments: [
        "{\"type\":\"Array\",\"value\":[{\"type\":\"Address\",\"value\":\"0x8c33bf917ab63d5b\"}]}",
        "{\"type\":\"Dictionary\",\"value\":[]}",
    ],
    reference_block_id: 7de3a92f73726037ab554b3b8dd7ab29585c98d88d6d3b61532de1431dc0f4d3,
    gas_limit: 100,
    proposal_key: ProposalKey {
        address: 0xe94a6e229293f196,
        key_id: 1,
        sequence_number: 181,
    },
    payer: 0xe94a6e229293f196,
    authorizers: [
        0xe94a6e229293f196,
    ],
    payload_signatures: [],
    envelope_signatures: [
        Signature {
            address: 0xe94a6e229293f196,
            key_id: 1,
            signature: 2b11cf254a246df78b506c6a3c7dc3c314e192fdf4106521f16b81859cddb4dee8a379662392289d265d560d7f689cb9b168392f2c645de82142eadee85c10ad,
        },
    ],
}
Found a cadence argument in the wild: [
    0x8c33bf917ab63d5b,
]
Found a cadence argument in the wild: {}
```


### Get Events
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Retrieve events by a given type in a specified block height range or through a list of block IDs.

📖 **Event type** is a string that follow a standard format:
```
A.{contract address}.{contract name}.{event name}
```

Please read more about [events in the documentation](https://docs.onflow.org/core-contracts/flow-token/). The exception to this standard are 
core events, and you should read more about them in [this document](https://docs.onflow.org/cadence/language/core-events/).

📖 **Block height range** expresses the height of the start and end block in the chain.

#### Examples
Example depicts ways to get events within block range or by block IDs:

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example link
```rust
use std::error::Error;

use flow_sdk::client::TonicHyperFlowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::mainnet()?;
    client.ping().await?;

    let latest_block_height = client.latest_block_header(true).await?.0.height;
    let start_height = latest_block_height - 20;

    println!(
        "Searching for accounts created within the last 20 blocks ({}-{})...",
        start_height, latest_block_height
    );

    for events in client
        .events_for_height_range("flow.AccountCreated", start_height, latest_block_height)
        .await?
        .results
        .iter()
    {
        if events.events.is_empty() {
            continue;
        }
        println!(
            "\nBlock #{} ({}):",
            events.block_height,
            hex::encode(&events.block_id)
        );
        for event in events.events.iter() {
            let val: cadence_json::ValueOwned = serde_json::from_slice(&event.payload)?;

            println!(" - {:#?}", val);
        }
    }

    Ok(())
}

```
Example output:
```rust
Searching for accounts created within the last 20 blocks (19495374-19495394)...

Block #19495378 (6d9d4315127cc9f26fde6ca7429d1d6718d35cd4a8ef2a7804d3e1ae6d2b9bbe):
  - Event {
    id: "flow.AccountCreated",
    address: 0x5231653750457b87,
}
  - Event {
    id: "flow.AccountCreated",
    address: 0x44650004c94171e6,
}

Block #19495380 (aaecc42eceebc0c626f43515c56a4e2ca18736b70cc7f9bdcc51e0acbb0adc7b):
  - Event {
    id: "flow.AccountCreated",
    address: 0xa002b9d9d8bb7139,
}

Block #19495386 (1140190de576c9d6ec7da897b5cf6aa92a246b33505be9d05285941bf5878be7):
  - Event {
    id: "flow.AccountCreated",
    address: 0x4f4f329d05c40fcf,
}

Block #19495390 (daab11c3b29f44426f647748ab32db081b34eb38f8f237cd9a0d3d53b4cd9a93):
  - Event {
    id: "flow.AccountCreated",
    address: 0xab288b40143e0f10,
}

Block #19495391 (9abfed42ac1e79ab91dfcb4192d802c1db251a28493cf4d8cceb0addb319cb0c):
  - Event {
    id: "flow.AccountCreated",
    address: 0xbd7cee738d3a0571,
}

Block #19495392 (e3c457b6fc92d04c4087ce0ae3e647e76f4da8e10a12c456b9e0a888a161cc31):
  - Event {
    id: "flow.AccountCreated",
    address: 0x591b57ae9cc005ae,
}
```

### Get Collections
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Retrieve a batch of transactions that have been included in the same block, known as ***collections***. 
Collections are used to improve consensus throughput by increasing the number of transactions per block and they act as a link between a block and a transaction.

📖 **Collection ID** is SHA3-256 hash of the collection payload.

Example retrieving a collection:
```rust
use std::error::Error;

use flow_sdk::{client::TonicHyperFlowClient, Block};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::testnet()?;
    client.ping().await?;

    let mut latest_block: Block = client.latest_block(true).await?.0;

    // traverse latest blocks until we find a collection guarantee.
    let collection_guarrantee = loop {
        if latest_block.collection_guarantees.is_empty() {
            // Go to the next block
            latest_block = client.block_by_id(&latest_block.parent_id).await?.0;
        } else {
            break latest_block.collection_guarantees.pop().unwrap();
        }
    };

    // retrieve the collection by id.
    let collection = client
        .collection_by_id(&collection_guarrantee.collection_id)
        .await?
        .collection;

    println!("OK: {:#?}", collection);

    Ok(())
}
```
Example output:
```rust
OK: Collection {
    id: 6ccc4829aaab7e7d06446b201c49f092dcef9be0428eabd690692067e2e1d947,
    transactions: [
        9481cc10ed0938bf9a429e098684dd30b3a95fa66db6287c23eebd6b46c30eaf,
    ],
}
```

### Execute Scripts
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Scripts allow you to write arbitrary non-mutating Cadence code on the Flow blockchain and return data. You can learn more about [Cadence and scripts here](https://docs.onflow.org/cadence/language/), but we are now only interested in executing the script code and getting back the data.

We can execute a script using the latest state of the Flow blockchain or we can choose to execute the script at a specific time in history defined by a block height or block ID.

📖 **Block ID** is SHA3-256 hash of the entire block payload, but you can get that value from the block response properties.

📖 **Block height** expresses the height of the block in the chain.

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example link
```rust
use std::error::Error;

use cadence_json::ValueRef;
use flow_sdk::{ExecuteScriptAtLatestBlockRequest, client::TonicHyperFlowClient};

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

    let ret = client.send(ExecuteScriptAtLatestBlockRequest {
        script: SIMPLE_SCRIPT,
        arguments: [ValueRef::Int(cadence_json::BigInt::from(32))],
    }).await?.parse()?;

    println!("{:#?}", ret);

    let ret = client.send(ExecuteScriptAtLatestBlockRequest {
        script: COMPLEX_SCRIPT,
        arguments: [ValueRef::String("John Doe")],
    }).await?.parse()?;

    println!("{:#?}", ret);

    Ok(())
}
```
Example output:
```rust
42
Struct {
    id: "s.ccbac9e72ee36be8881671e8939b970bb8bc81fb8cfd695c6fd848cf75248802.User",
    balance: 10.00000000,
    address: 0x0000000000000001,
    name: "John Doe",
}
```

## Mutate Flow Network
Flow, like most blockchains, allows anybody to submit a transaction that mutates the shared global chain state. A transaction is an object that holds a payload, which describes the state mutation, and one or more authorizations that permit the transaction to mutate the state owned by specific accounts.

Transaction data is composed and signed with help of the SDK. The signed payload of transaction then gets submitted to the access node API. If a transaction is invalid or the correct number of authorizing signatures are not provided, it gets rejected. 

Executing a transaction requires couple of steps:
- [Building a transaction](#build-transactions).
- [Signing a transaction](#sign-transactions).
- [Sending a transaction](#send-transactions).

## Transactions
A transaction is nothing more than a signed set of data that includes script code which are instructions on how to mutate the network state and properties that define and limit it's execution. All these properties are explained bellow. 

📖 **Script** field is the portion of the transaction that describes the state mutation logic. On Flow, transaction logic is written in [Cadence](https://docs.onflow.org/cadence/). Here is an example transaction script:
```
transaction(greeting: String) {
  execute {
    log(greeting.concat(", World!"))
  }
}
```

📖 **Arguments**. A transaction can accept zero or more arguments that are passed into the Cadence script. The arguments on the transaction must match the number and order declared in the Cadence script. Sample script from above accepts a single `String` argument.

📖 **[Proposal key](https://docs.onflow.org/concepts/transaction-signing/#proposal-key)** must be provided to act as a sequence number and prevent reply and other potential attacks.

Each account key maintains a separate transaction sequence counter; the key that lends its sequence number to a transaction is called the proposal key.

A proposal key contains three fields:
- Account address
- Key index
- Sequence number

A transaction is only valid if its declared sequence number matches the current on-chain sequence number for that key. The sequence number increments by one after the transaction is executed.

📖 **[Payer](https://docs.onflow.org/concepts/transaction-signing/#signer-roles)** is the account that pays the fees for the transaction. A transaction must specify exactly one payer. The payer is only responsible for paying the network and gas fees; the transaction is not authorized to access resources or code stored in the payer account.

📖 **[Authorizers](https://docs.onflow.org/concepts/transaction-signing/#signer-roles)** are accounts that authorize a transaction to read and mutate their resources. A transaction can specify zero or more authorizers, depending on how many accounts the transaction needs to access.

The number of authorizers on the transaction must match the number of AuthAccount parameters declared in the prepare statement of the Cadence script.

Example transaction with multiple authorizers:
```
transaction {
  prepare(authorizer1: AuthAccount, authorizer2: AuthAccount) { }
}
```

📖 **Gas limit** is the limit on the amount of computation a transaction requires, and it will abort if it exceeds its gas limit.
Cadence uses metering to measure the number of operations per transaction. You can read more about it in the [Cadence documentation](/cadence).

The gas limit depends on the complexity of the transaction script. Until dedicated gas estimation tooling exists, it's best to use the emulator to test complex transactions and determine a safe limit.

📖 **Reference block** specifies an expiration window (measured in blocks) during which a transaction is considered valid by the network.
A transaction will be rejected if it is submitted past its expiry block. Flow calculates transaction expiry using the _reference block_ field on a transaction.
A transaction expires after `600` blocks are committed on top of the reference block, which takes about 10 minutes at average Mainnet block rates.

### Build Transactions
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Building a transaction involves setting the required properties explained above and producing a transaction object. 

Here we define a simple transaction script that will be used to execute on the network and serve as a good learning example.

```
transaction(greeting: String) {

  let guest: Address

  prepare(authorizer: AuthAccount) {
    self.guest = authorizer.address
  }

  execute {
    log(greeting.concat(",").concat(guest.toString()))
  }
}
```

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example link
```rust
use cadence_json::ValueRef;
use flow_sdk::TransactionHeaderBuilder;


const SCRIPT: &str = r#"
    transaction(greeting: String) {
       let guest: Address

       prepare(authorizer: AuthAccount) {
           self.guest = authorizer.address
       }

       execute {
           log(greeting.concat(",").concat(guest.toString()))
       }
    }
"#;

let argument = ValueRef::String("Hello");

let header = TransactionHeaderBuilder::new().script_static(SCRIPT).argument(&argument).build();
```

After you have successfully [built a transaction](#build-transactions) the next step in the process is to sign it.

### Sign Transactions
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO specs here

Flow introduces new concepts that allow for more flexibility when creating and signing transactions.
Before trying the examples below, we recommend that you read through the [transaction signature documentation](https://docs.onflow.org/concepts/accounts-and-keys/).

After you have successfully [built a transaction](#build-transactions) the next step in the process is to sign it. Flow transactions have envelope and payload signatures, and you should learn about each in the [signature documentation](https://docs.onflow.org/concepts/accounts-and-keys/#anatomy-of-a-transaction).

Quick example of building a transaction:
```
// TODO short example to build transaction
```

Signatures can be generated more securely using keys stored in a hardware device such as an [HSM](https://en.wikipedia.org/wiki/Hardware_security_module). The `crypto.Signer` interface is intended to be flexible enough to support a variety of signer implementations and is not limited to in-memory implementations.

Simple signature example:
```
// TODO signature example
```

Flow supports great flexibility when it comes to transaction signing, we can define multiple authorizers (multi-sig transactions) and have different payer account than proposer. We will explore advanced signing scenarios bellow.

### [Single party, single signature](https://docs.onflow.org/concepts/transaction-signing/#single-party-single-signature)

- Proposer, payer and authorizer are the same account (`0x01`).
- Only the envelope must be signed.
- Proposal key must have full signing weight.

| Account | Key ID | Weight |
| ------- | ------ | ------ |
| `0x01`  | 1      | 1.0    |

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">](https://github.com/onflow/flow-go-sdk/tree/master/examples#single-party-single-signature)**
```
// TODO example
```


### [Single party, multiple signatures](https://docs.onflow.org/concepts/transaction-signing/#single-party-multiple-signatures)

- Proposer, payer and authorizer are the same account (`0x01`).
- Only the envelope must be signed.
- Each key has weight 0.5, so two signatures are required.

| Account | Key ID | Weight |
| ------- | ------ | ------ |
| `0x01`  | 1      | 0.5    |
| `0x01`  | 2      | 0.5    |

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">](https://github.com/onflow/flow-go-sdk/tree/master/examples#single-party-multiple-signatures)**
```
// TODO example
```

### [Multiple parties](https://docs.onflow.org/concepts/transaction-signing/#multiple-parties)

- Proposer and authorizer are the same account (`0x01`).
- Payer is a separate account (`0x02`).
- Account `0x01` signs the payload.
- Account `0x02` signs the envelope.
    - Account `0x02` must sign last since it is the payer.

| Account | Key ID | Weight |
| ------- | ------ | ------ |
| `0x01`  | 1      | 1.0    |
| `0x02`  | 3      | 1.0    |

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">](https://github.com/onflow/flow-go-sdk/tree/master/examples#multiple-parties)**
```
// TODO example
```

### [Multiple parties, two authorizers](https://docs.onflow.org/concepts/transaction-signing/#multiple-parties)

- Proposer and authorizer are the same account (`0x01`).
- Payer is a separate account (`0x02`).
- Account `0x01` signs the payload.
- Account `0x02` signs the envelope.
    - Account `0x02` must sign last since it is the payer.
- Account `0x02` is also an authorizer to show how to include two AuthAccounts into an transaction

| Account | Key ID | Weight |
| ------- | ------ | ------ |
| `0x01`  | 1      | 1.0    |
| `0x02`  | 3      | 1.0    |

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">](https://github.com/onflow/flow-go-sdk/tree/master/examples#multiple-parties-two-authorizers)**
```
// TODO example
```

### [Multiple parties, multiple signatures](https://docs.onflow.org/concepts/transaction-signing/#multiple-parties)

- Proposer and authorizer are the same account (`0x01`).
- Payer is a separate account (`0x02`).
- Account `0x01` signs the payload.
- Account `0x02` signs the envelope.
    - Account `0x02` must sign last since it is the payer.
- Both accounts must sign twice (once with each of their keys).

| Account | Key ID | Weight |
| ------- | ------ | ------ |
| `0x01`  | 1      | 0.5    |
| `0x01`  | 2      | 0.5    |
| `0x02`  | 3      | 0.5    |
| `0x02`  | 4      | 0.5    |

**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example link
```
// TODO example
```


### Send Transactions
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO reference here

After a transaction has been [built](#build-transactions) and [signed](#sign-transactions), it can be sent to the Flow blockchain where it will be executed. If sending was successful you can then [retrieve the transaction result](#get-transactions).


**[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/try.svg" width="130">]()** // TODO example here
```
// TODO send transaction example
```


### Create Accounts
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO reference here

On Flow, account creation happens inside a transaction. Because the network allows for a many-to-many relationship between public keys and accounts, it's not possible to derive a new account address from a public key offline. 

The Flow VM uses a deterministic address generation algorithm to assigen account addresses on chain. You can find more details about address generation in the [accounts & keys documentation](https://docs.onflow.org/concepts/accounts-and-keys/).

#### Public Key
Flow uses ECDSA key pairs to control access to user accounts. Each key pair can be used in combination with the SHA2-256 or SHA3-256 hashing algorithms.

⚠️ You'll need to authorize at least one public key to control your new account.

Flow represents ECDSA public keys in raw form without additional metadata. Each key is a single byte slice containing a concatenation of its X and Y components in big-endian byte form.

A Flow account can contain zero (not possible to control) or more public keys, referred to as account keys. Read more about [accounts in the documentation](https://docs.onflow.org/concepts/accounts-and-keys/#accounts).

An account key contains the following data:
- Raw public key (described above)
- Signature algorithm
- Hash algorithm
- Weight (integer between 0-1000)

Account creation happens inside a transaction, which means that somebody must pay to submit that transaction to the network. We'll call this person the account creator. Make sure you have read [sending a transaction section](#send-transactions) first. 

```
// TODO create account example
```

After the account creation transaction has been submitted you can retrieve the new account address by [getting the transaction result](#get-transactions). 

The new account address will be emitted in a system-level `flow.AccountCreated` event.

```
// TODO get new account address
```

### Generate Keys
[<img src="https://raw.githubusercontent.com/onflow/sdks/main/templates/documentation/ref.svg" width="130">]() // TODO reference here

Flow uses [ECDSA](https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm) signatures to control access to user accounts. Each key pair can be used in combination with the `SHA2-256` or `SHA3-256` hashing algorithms.

Here's how to generate an ECDSA private key for the P-256 (secp256r1) curve.

```
// TODO generate key example
```

The example above uses an ECDSA key pair on the P-256 (secp256r1) elliptic curve. Flow also supports the secp256k1 curve used by Bitcoin and Ethereum. Read more about [supported algorithms here](https://docs.onflow.org/concepts/accounts-and-keys/#supported-signature--hash-algorithms).
