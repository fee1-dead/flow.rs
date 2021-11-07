use super::algorithms::*;
use super::client::*;
use super::immediate_fut;
use crate::account::*;

type MockAccount = Account<MockClient, MockKey, MockSigner, MockHasher>;

#[test]
fn test_account_new() {
    let res = immediate_fut(MockAccount::new(MockClient, &[0x01], ACC01_KEY));

    let account = res.expect("Failed to create MockAccount");

    assert_eq!([0x01], account.address());

    let mut signatures = account.sign_data("Sign me! Sign me! Sign me!");

    assert_eq!(
        Some(MockSig(*b"Sign me! Sign me! Sign me!\0\0\0\0\0\0")),
        signatures.next()
    );
    assert_eq!(None, signatures.next());

    let res = immediate_fut(MockAccount::new(MockClient, &[0x01], [0; 64]));

    assert!(matches!(res, Err(Error::NoMatchingKeyFound)));

    let res = immediate_fut(MockAccount::new(MockClient, &[0x01], ACC01_KEY_NOWEIGHT));

    assert!(matches!(res, Err(Error::NotEnoughWeight)));

    let res = immediate_fut(MockAccount::new(MockClient, &[0x01], ACC01_KEY_REVOKED));

    assert!(matches!(res, Err(Error::KeyRevoked)));
}
