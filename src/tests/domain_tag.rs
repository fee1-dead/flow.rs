use super::fixtures::Test;

#[derive(serde::Deserialize, Debug)]
pub struct DTagTest {
    #[serde(rename = "in")]
    pub dt_in: String,
    #[serde(rename = "out")]
    pub dt_out: String,
}

impl Test for DTagTest {
    fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::account::test_pad;
        assert_eq!(hex::encode(test_pad(self.dt_in.as_bytes())), self.dt_out);
        Ok(())
    }
}

crate::test_fixtures!(
    DTagTest,
    "domain-tag-encoding.json",
    test_domain_tag_encoding
);
