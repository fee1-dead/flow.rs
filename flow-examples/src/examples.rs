use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::SplitWhitespace;

use crate::ExampleAccount;

#[macro_export]
macro_rules! example {
    ($run_ident: ident) => {
        pub fn __run<'a>(
            account: &'a mut crate::ExampleAccount,
            args: &'a mut std::str::SplitWhitespace<'_>,
        ) -> crate::examples::ExampleReturnTy<'a> {
            Box::pin(async move { $run_ident(account, args).await })
        }
    };
}

// https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#bit-twiddling
macro_rules! count_tts {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { (count_tts!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { count_tts!($($a)*) << 1 };
}

macro_rules! examples {
    ($(
        #[doc = $doc:literal]
        $(#[arguments = $args: literal])?
        pub mod $example_name:ident;
    )+) => {
        $(
            #[doc = $doc]
            pub mod $example_name;
        )+

        pub const EXAMPLES_LEN: usize = count_tts!($($example_name)+);
        pub static EXAMPLES: [Example; EXAMPLES_LEN] = [
            $(
                Example {
                    f: $example_name::__run,
                    name: stringify!($example_name),
                    arguments: concat!("", $($args)?),
                    description: $doc,
                },
            )+
        ];
    };
}

examples! {
    /// Creates an account by sending a transaction.
    pub mod create_account;

    /// Retrieves information about an account
    #[arguments = "ADDRESS [BLOCK_HEIGHT]"]
    pub mod get_account_info;

    /// Retrieves information about the latest block or specific block by id/height
    #[arguments = "[BLOCK_ID/BLOCK_HEIGHT]"]
    pub mod get_block;

    /// Builds a transaction.
    #[arguments = "TXN_SCRIPT_FILE [ARGUMENTS_FILE]"]
    pub mod build_txn;

    /// Sends a transaction after it got built.
    pub mod send_txn;

    /// Builds a transaction with two proposers.
    #[arguments = "TXN_SCRIPT_FILE [ARGUMENTS_FILE]"]
    pub mod build_txn_multi;

    /// Sends a transaction after it got built.
    pub mod send_txn_multi;

    /// Retrieves information about a transaction
    #[arguments = "TRANSACTION_ID"]
    pub mod get_txn;

    /// Retrieves information about a transaction's result
    #[arguments = "TRANSACTION_ID"]
    pub mod get_txn_result;

    /// Retrieves events within a height range or by block ids
    #[arguments = "EVENT_TYPE [START_HEIGHT END_HEIGHT or BLOCK_IDS..]"]
    pub mod get_events;

    /// Run a script on a specific block or on the latest blockchain state
    #[arguments = "SCRIPT_FILE [ARGUMENTS_FILE] [BLOCK_ID/BLOCK_HEIGHT]"]
    pub mod run_script;
}

lazy_static::lazy_static! {
    pub static ref EXAMPLES_BY_NAME: HashMap<&'static str, &'static Example> = {
        let mut map = HashMap::with_capacity(EXAMPLES_LEN);
        for example in &EXAMPLES {
            map.insert(example.name, example);
        }
        map
    };
}

pub type ExampleReturnTy<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>;

#[derive(Clone, Copy)]
pub struct Example {
    pub f: for<'a> fn(&'a mut ExampleAccount, &'a mut SplitWhitespace<'a>) -> ExampleReturnTy<'a>,
    pub name: &'static str,
    pub arguments: &'static str,
    pub description: &'static str,
}
