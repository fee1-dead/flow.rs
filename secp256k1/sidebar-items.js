initSidebarItems({"enum":[["All","Represents the set of all capabilities."],["Error","An ECDSA error"],["SignOnly","Represents the set of capabilities needed for signing."],["VerifyOnly","Represents the set of capabilities needed for verification."]],"externcrate":[["rand",""],["secp256k1_sys",""]],"mod":[["constants","Constants"],["ecdh","ECDH"],["key","Public and secret keys"],["schnorrsig","schnorrsig"]],"struct":[["AllPreallocated","Represents the set of all capabilities with a user preallocated memory."],["Message","A (hashed) message input to an ECDSA signature"],["Secp256k1","The secp256k1 engine, used to execute all signature operations"],["SerializedSignature","A DER serialized Signature"],["SignOnlyPreallocated","Represents the set of capabilities needed for signing with a user preallocated memory."],["Signature","An ECDSA signature"],["VerifyOnlyPreallocated","Represents the set of capabilities needed for verification with a user preallocated memory."]],"trait":[["Context","A trait for all kinds of Context’s that Lets you define the exact flags and a function to deallocate memory. It shouldn’t be possible to implement this for types outside this crate."],["Signing","Marker trait for indicating that an instance of `Secp256k1` can be used for signing."],["ThirtyTwoByteHash","Trait describing something that promises to be a 32-byte random number; in particular, it has negligible probability of being zero or overflowing the group order. Such objects may be converted to `Message`s without any error paths."],["Verification","Marker trait for indicating that an instance of `Secp256k1` can be used for verification."]]});