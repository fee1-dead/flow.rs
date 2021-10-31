# Flow.rs Examples

This repository contains examples of using the Flow.rs SDK.

To run the examples, the Flow CLI must be installed and in the PATH.

You can set your working directory to this directory and run `cargo run`.

# Available Examples

You can type `list` to get a table of available examples.

## Get information about an account (at the latest block or a specific block height)

```
run get_account_info 0x01cf0e2f2f715450
```

Or at a specific block height:

```
run get_account_info 0x01cf0e2f2f715450 1
```

## Create another account

```
run create_account
```

## Get information about a block

Latest block:

```
run get_block
```

Block at block height:

```
run get_block 1
```

Block by block ID:

```
run get_block 7bc42fe85d32ca513769a74f97f7e1a7bad6c9407f0d934c2aa645ef9cf613c7
```

## Get information about a transaction

```
run get_txn ba4819ded52e457820936aef656651fdd22ee6314090c8feba1c2391df4b2c05
```

## Get result of a transaction

```
run get_txn_result ba4819ded52e457820936aef656651fdd22ee6314090c8feba1c2391df4b2c05
```

## Get events within a block height range or by list of block ids

Within a block height range:

```
run get_events flow.AccountCreated 1 2
```

By a list of block IDs:

```
run get_events flow.AccountCreated 51392d353c878a0d5c23917783ef2d9b7f3f44a16f82e8efff21aa2cd090bc00 8ce36abc134eb81ba092513a299f0b70138a86333adab5291bfe682929fd5e30
```

## Execute script on the latest block or a specific block

`my_script.cdc`:

```
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
```

`my_script_arguments.json`:

```json
[
    {
        "type": "String",
        "value": "John Doe"
    }
]
```

On the latest block:

```
run run_script my_script.cdc my_script_arguments.json
```

On a specific block height:

```
run run_script my_script.cdc my_script_arguments.json 1
```

On a block by ID:

```
run run_script my_script.cdc my_script_arguments.json 7bc42fe85d32ca513769a74f97f7e1a7bad6c9407f0d934c2aa645ef9cf613c7
```

## Build a transaction

`transaction.cdc`:

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

`transaction_multi.cdc`:

```
transaction {
    prepare(acct1: AuthAccount, acct2: AuthAccount) {
        log([acct1, acct2])
    }
}
```

`arguments.json`:

```json
[
    {
        "type": "String",
        "value": "Hello"
    }
]
```

### Build a simple transaction

```
run build_txn transaction.cdc arguments.json
```

### Build a transaction with two authorizers

```
run build_txn_multi transaction_multi.cdc
```

## Sending transactions

### Send a previously built, simple transaction

```
run send_txn
```

### Send a previously built, transaction with two authorizers

```
run send_txn_multi
```