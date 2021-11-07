use futures_util::future::*;

use super::immediate_fut;
use crate::access::*;
use crate::algorithms::*;
use crate::client::{FlowClient, GrpcClient};
use crate::entities::*;

pub const ACC01_KEY: [u8; 64] = {
    let mut base = [0; 64];
    base[0] = 0x02;
    base
};

pub const ACC01_KEY_NOWEIGHT: [u8; 64] = {
    let mut base = [0; 64];
    base[0] = 0x03;
    base
};

pub const ACC01_KEY_REVOKED: [u8; 64] = {
    let mut base = [0; 64];
    base[0] = 0x04;
    base
};

fn acc_01() -> Account {
    Account {
        address: [0x01].into(),
        balance: 1337,
        code: [].into(),
        keys: vec![
            AccountKey {
                index: 0,
                public_key: ACC01_KEY.into(),
                sign_algo: Secp256k1::CODE,
                hash_algo: Sha3::CODE,
                weight: 1000,
                sequence_number: 42,
                revoked: false,
            },
            AccountKey {
                index: 1,
                public_key: ACC01_KEY_NOWEIGHT.into(),
                sign_algo: Secp256k1::CODE,
                hash_algo: Sha3::CODE,
                weight: 0,
                sequence_number: 42,
                revoked: false,
            },
            AccountKey {
                index: 1,
                public_key: ACC01_KEY_REVOKED.into(),
                sign_algo: Secp256k1::CODE,
                hash_algo: Sha3::CODE,
                weight: 1000,
                sequence_number: 42,
                revoked: true,
            },
        ]
        .into(),
        contracts: Default::default(),
    }
}

pub struct MockClient;

impl GrpcClient<GetAccountAtLatestBlockRequest<&[u8]>, AccountResponse> for MockClient {
    type Error = &'static str;

    fn send<'a>(
        &'a mut self,
        input: GetAccountAtLatestBlockRequest<&[u8]>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AccountResponse, Self::Error>> + 'a>,
    > {
        let fut = match input.address {
            [0x01] => ok(AccountResponse { account: acc_01() }),
            _ => err("address not found"),
        };

        Box::pin(fut)
    }
}

#[test]
fn test_get_account() {
    let mut client = FlowClient::new(MockClient);

    let account = immediate_fut(client.account_at_latest_block(&[0x01]));

    assert_eq!(Ok(acc_01()), account);

    let unknown = immediate_fut(client.account_at_latest_block(&[0x02]));

    assert_eq!(Err("address not found"), unknown);
}
