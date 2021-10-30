# Flow.rs Examples

This repository contains examples of using the Flow.rs SDK.

To run the examples, the Flow CLI must be installed and in the PATH.

You can set your working directory to this directory and run `cargo run`.

# Available Examples

You can type `list` to get a table of available examples.

## Get information about an account (at the latest block or a specific block height)

```
run get_account_info 0x01
```

Or at a specific block height:

```
run get_account_info 0x01 1
```

## Create another account

```
run create_account
```

## Get information about a block

```
run get_block BLOCK_HASH
```

## Get information about a transaction

```
run get_txn TX_HASH
```

## Get result of a transaction

```
run get_txn_result TX_HASH
```