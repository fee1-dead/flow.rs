initSidebarItems({"constant":[["SECP256K1_SER_COMPRESSED","Flag for keys to indicate compressed serialization format"],["SECP256K1_SER_UNCOMPRESSED","Flag for keys to indicate uncompressed serialization format"],["SECP256K1_START_NONE","Flag for context to enable no precomputation"],["SECP256K1_START_SIGN","Flag for context to enable signing precomputation"],["SECP256K1_START_VERIFY","Flag for context to enable verification precomputation"]],"fn":[["ecdsa_signature_parse_der_lax",""],["rustsecp256k1_v0_4_1_context_create","A reimplementation of the C function `secp256k1_context_create` in rust."],["rustsecp256k1_v0_4_1_context_destroy","A reimplementation of the C function `secp256k1_context_destroy` in rust."],["rustsecp256k1_v0_4_1_default_error_callback_fn","This function is an override for the C function, this is the an edited version of the original description:"],["rustsecp256k1_v0_4_1_default_illegal_callback_fn","This function is an override for the C function, this is the an edited version of the original description:"],["secp256k1_context_create",""],["secp256k1_context_destroy",""],["secp256k1_context_preallocated_clone",""],["secp256k1_context_preallocated_clone_size",""],["secp256k1_context_preallocated_create",""],["secp256k1_context_preallocated_destroy",""],["secp256k1_context_preallocated_size",""],["secp256k1_context_randomize",""],["secp256k1_ec_privkey_negate",""],["secp256k1_ec_privkey_tweak_add",""],["secp256k1_ec_privkey_tweak_mul",""],["secp256k1_ec_pubkey_combine",""],["secp256k1_ec_pubkey_create",""],["secp256k1_ec_pubkey_negate",""],["secp256k1_ec_pubkey_parse",""],["secp256k1_ec_pubkey_serialize",""],["secp256k1_ec_pubkey_tweak_add",""],["secp256k1_ec_pubkey_tweak_mul",""],["secp256k1_ec_seckey_negate",""],["secp256k1_ec_seckey_tweak_add",""],["secp256k1_ec_seckey_tweak_mul",""],["secp256k1_ec_seckey_verify",""],["secp256k1_ecdh",""],["secp256k1_ecdsa_sign",""],["secp256k1_ecdsa_signature_normalize",""],["secp256k1_ecdsa_signature_parse_compact",""],["secp256k1_ecdsa_signature_parse_der",""],["secp256k1_ecdsa_signature_serialize_compact",""],["secp256k1_ecdsa_signature_serialize_der",""],["secp256k1_ecdsa_verify",""],["secp256k1_keypair_create",""],["secp256k1_keypair_pub",""],["secp256k1_keypair_sec",""],["secp256k1_keypair_xonly_pub",""],["secp256k1_keypair_xonly_tweak_add",""],["secp256k1_schnorrsig_sign",""],["secp256k1_schnorrsig_verify",""],["secp256k1_xonly_pubkey_from_pubkey",""],["secp256k1_xonly_pubkey_parse",""],["secp256k1_xonly_pubkey_serialize",""],["secp256k1_xonly_pubkey_tweak_add",""],["secp256k1_xonly_pubkey_tweak_add_check",""]],"macro":[["impl_array_newtype",""],["impl_raw_debug",""]],"mod":[["types",""]],"static":[["secp256k1_context_no_precomp",""],["secp256k1_ecdh_hash_function_default","Default ECDH hash function"],["secp256k1_nonce_function_bip340",""],["secp256k1_nonce_function_default",""],["secp256k1_nonce_function_rfc6979",""]],"struct":[["Context","A Secp256k1 context, containing various precomputed values and such needed to do elliptic curve computations. If you create one of these with `secp256k1_context_create` you MUST destroy it with `secp256k1_context_destroy`, or else you will have a memory leak."],["KeyPair",""],["PublicKey","Library-internal representation of a Secp256k1 public key"],["Signature","Library-internal representation of a Secp256k1 signature"],["XOnlyPublicKey",""]],"trait":[["CPtr","A trait for producing pointers that will always be valid in C. (assuming NULL pointer is a valid no-op) Rust doesn’t promise what pointers does it give to ZST (https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts) In case the type is empty this trait will give a NULL pointer, which should be handled in C."]],"type":[["EcdhHashFn","Hash function to use to post-process an ECDH point to get a shared secret."],["NonceFn","A nonce generation function. Ordinary users of the library never need to see this type; only if you need to control nonce generation do you need to use it. I have deliberately made this hard to do: you have to write your own wrapper around the FFI functions to use it. And it’s an unsafe type. Nonces are generated deterministically by RFC6979 by default; there should be no need to ever change this."],["SchnorrNonceFn","Same as secp256k1_nonce function with the exception of accepting an additional pubkey argument and not requiring an attempt argument. The pubkey argument can protect signature schemes with key-prefixed challenge hash inputs against reusing the nonce when signing with the wrong precomputed pubkey."]]});