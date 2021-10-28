use std::str::SplitWhitespace;
use std::{error::Error, future::Future, pin::Pin};
use std::collections::HashMap;

use crate::ExampleAccount;

#[macro_export]
macro_rules! example {
    (pub async fn run($($tt:tt)*) -> Result<$accty2:ty, $errty:ty> $block:block) => {
        async fn run_inner($($tt)*) -> Result<$accty2, $errty> $block

        pub fn run<'a>(account: &'a mut crate::ExampleAccount, args: &'a mut std::str::SplitWhitespace<'_>) -> crate::examples::ExampleReturnTy<'a> {
            Box::pin(async move {
                run_inner(account, args).await
            })
        }
    };
}

#[macro_export]
macro_rules! bail {
    ($($tt:tt)*) => {{
        eprintln!($($tt)*);
        return Ok(())
    }};
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
                    f: $example_name::run,
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
    #[arguments = "ADDRESS"]
    pub mod get_account_info;
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

pub type ExampleReturnTy<'a> = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + 'a>>;

#[derive(Clone, Copy)]
pub struct Example {
    pub f: for<'a> fn(&'a mut ExampleAccount, &'a mut SplitWhitespace<'a>) -> ExampleReturnTy<'a>,
    pub name: &'static str,
    pub arguments: &'static str,
    pub description: &'static str,
}

