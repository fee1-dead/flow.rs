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