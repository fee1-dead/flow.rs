use num_bigint::{BigInt, BigUint, Sign};

use crate::*;

// Tests for decoding JSON-Cadence values, and encoding them back.
//
// Testing equality.
macro_rules! parse_tests {
    ($(fn $name:ident() {$($json: tt)*} <==> $cadence_value:expr)+) => {$(
        #[test]
        fn $name() {
            let json = concat!("{", stringify!($($json)*), "}");

            let found_cadence: ValueOwned = serde_json::from_str(json).expect("Failed to parse JSON");
            let expected_cadence: ValueOwned = $cadence_value;

            assert_eq!(expected_cadence, found_cadence);

            let found_json_value = serde_json::to_value(&expected_cadence).expect("Failed to convert to JSON value");
            let expected_json_value: serde_json::Value = serde_json::from_str(json).expect("Failed to parse JSON");

            assert_eq!(expected_json_value, found_json_value);
        })+
    };
}

parse_tests! {
    fn void() {
        "type": "Void"
    } <==> ValueOwned::Void

    fn optional_none() {
        "type": "Optional",
        "value": null
    } <==> ValueOwned::Optional(None)

    fn bool() {
        "type": "Bool",
        "value": true
    } <==> ValueOwned::Bool(true)

    fn string() {
        "type": "String",
        "value": "Hello World"
    } <==> ValueOwned::String("Hello World".into())

    fn address() {
        "type": "Address",
        "value": "0x1234"
    } <==> ValueOwned::Address(AddressOwned { data: [0x12, 0x34].into() })

    fn uint8() {
        "type": "UInt8",
        "value": "123"
    } <==> ValueOwned::UInt8(123)

    fn uint() {
        "type": "UInt",
        "value": "115792089237316195423570985008687907853269984665640564039457584007913129639936"
    } <==> ValueOwned::UInt(BigUint::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 1]))

    fn fixed_point() {
        "type": "Fix64",
        "value": "12.30000000"
    } <==> ValueOwned::Fix64(Fix64::from_raw(1230000000))

    fn array() {
        "type": "Array",
        "value": [
          {
            "type": "Int16",
            "value": "123"
          },
          {
            "type": "String",
            "value": "test"
          },
          {
            "type": "Bool",
            "value": true
          }
        ]
    } <==> ValueOwned::Array(vec![ ValueOwned::Int16(123), ValueOwned::String("test".into()), ValueOwned::Bool(true) ])

    fn dictionary() {
        "type": "Dictionary",
        "value": [
          {
            "key": {
              "type": "UInt8",
              "value": "123"
            },
            "value": {
              "type": "String",
              "value": "test"
            }
          }
        ]
    } <==> ValueOwned::Dictionary(vec![ EntryOwned {
        key: ValueOwned::UInt8(123),
        value: ValueOwned::String("test".into())
    }])

    fn composite() {
        "type": "Resource",
        "value": {
          "id": "0x3.GreatContract.GreatNFT",
          "fields": [
            {
              "name": "power",
              "value": {"type": "Int", "value": "1"}
            }
          ]
        }
    } <==> ValueOwned::Resource(CompositeOwned {
        id: "0x3.GreatContract.GreatNFT".into(),
        fields: vec![
            CompositeFieldOwned {
                name: "power".into(),
                value: ValueOwned::Int(BigInt::new(Sign::Plus, vec![1]))
            }
        ]
    })

    fn path() {
        "type": "Path",
        "value": {
          "domain": "storage",
          "identifier": "flowTokenVault"
        }
    } <==> ValueOwned::Path(PathOwned {
        domain: PathDomain::Storage,
        identifier: "flowTokenVault".into(),
    })

    fn ty() {
        "type": "Type",
        "value": {
          "staticType": "Int"
        }
    } <==> ValueOwned::Type("Int".into())

    fn capability() {
        "type": "Capability",
        "value": {
          "path": "/public/someInteger",
          "address": "0x01",
          "borrowType": "Int"
        }
    } <==> ValueOwned::Capability(CapabilityOwned {
        path: "/public/someInteger".into(),
        address: AddressOwned { data: [0x01].into() },
        borrow_type: "Int".into(),
    })

    fn deeply_nested_array() {
        "type": "Array",
        "value": [
            {
                "type": "Array",
                "value": [{
                    "type": "Array",
                    "value": [
                        {
                            "type": "Array",
                            "value": [{
                                "type": "Array",
                                "value": [
                                    {
                                        "type": "Array",
                                        "value": [{
                                            "type": "Array",
                                            "value": [
                                                {
                                                    "type": "Array",
                                                    "value": []
                                                }
                                            ]
                                        }]
                                    }
                                ]
                            }]
                        }
                    ]
                }]
            }
        ]
    } <==> ValueOwned::Array(vec![
        ValueOwned::Array(vec![
            ValueOwned::Array(vec![
                ValueOwned::Array(vec![
                    ValueOwned::Array(vec![
                        ValueOwned::Array(vec![
                            ValueOwned::Array(vec![
                                ValueOwned::Array(vec![])
                            ])
                        ])
                    ])
                ])
            ])
        ])
    ])
}
