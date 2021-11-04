use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::panic::{catch_unwind, UnwindSafe};

use serde::de::DeserializeOwned;

pub mod domain_tag;
pub mod rlp;

#[derive(serde::Deserialize)]
pub struct TestFixture<T> {
    pub title: String,
    pub description: String,
    pub tests: HashMap<String, T>,
}

#[derive(Debug)]
pub struct TestFixtureResult {
    pub passes: HashSet<String>,
    pub failures: HashMap<String, Box<dyn Error + Send + Sync>>,
}

impl Display for TestFixtureResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PASSED: {:?}", self.passes)?;
        writeln!(f, "FAILED: {:?}", self.failures)
    }
}

impl Error for TestFixtureResult {}

impl<T: Test + UnwindSafe> TestFixture<T> {
    pub fn run(self) -> Result<(), TestFixtureResult> {
        let mut passes = HashSet::new();
        let mut failures = HashMap::new();
        for (name, test) in self.tests {
            match catch_unwind(move || test.run()) {
                Ok(Ok(())) => {
                    passes.insert(name);
                }
                Ok(Err(e)) => {
                    failures.insert(name, e);
                }
                Err(e) => {
                    failures.insert(name, format!("Panicked: {:?}", e).into());
                }
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(TestFixtureResult { passes, failures })
        }
    }
}

pub trait Test: Sized + DeserializeOwned {
    fn run(self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn parse_test_fixture<T: Test + UnwindSafe>(input: &str) -> TestFixture<T> {
    serde_json::from_str(input).unwrap()
}

#[macro_export]
macro_rules! test_fixtures {
    ($TestTy:ty, $file_name:literal, $fn_name: ident) => {
        #[test]
        fn $fn_name() -> Result<(), crate::tests::fixtures::TestFixtureResult> {
            let fixture =
                crate::tests::fixtures::parse_test_fixture::<$TestTy>(include_str!($file_name));
            fixture.run()
        }
    };
}
