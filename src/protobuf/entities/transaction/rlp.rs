use rlp::RlpStream;

pub fn rlp_encode_transaction_envelope(
    s: &mut RlpStream,
    script: impl AsRef<[u8]>,
    arguments: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
    reference_block_id: impl AsRef<[u8]>,
    gas_limit: u64,
    proposal_key_address: impl AsRef<[u8]>,
    proposal_key_id: u32,
    proposal_key_sequence_number: u32,
    payer: impl AsRef<[u8]>,
    authorizers: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
    payload_signatures: impl IntoIterator<
        IntoIter = impl ExactSizeIterator<Item = (u32, u32, impl AsRef<[u8]>)>,
    >,
) {
    s.begin_list(2);
    rlp_encode_transaction_payload(
        s,
        script,
        arguments,
        reference_block_id,
        gas_limit,
        proposal_key_address,
        proposal_key_id,
        proposal_key_sequence_number,
        payer,
        authorizers,
    );
    let payload_signatures = payload_signatures.into_iter();
    s.begin_list(payload_signatures.len());
    for (signer_index, key_id, signature) in payload_signatures {
        s.begin_list(3)
            .append(&signer_index)
            .append(&key_id)
            .append(&signature.as_ref());
    }
}

pub fn rlp_encode_transaction_payload(
    stream: &mut RlpStream,
    script: impl AsRef<[u8]>,
    arguments: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
    reference_block_id: impl AsRef<[u8]>,
    gas_limit: u64,
    proposal_key_address: impl AsRef<[u8]>,
    proposal_key_id: u32,
    proposal_key_sequence_number: u32,
    payer: impl AsRef<[u8]>,
    authorizers: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
) {
    stream.begin_list(9).append(&script.as_ref());

    let arguments = arguments.into_iter();

    stream.begin_list(arguments.len());

    for arg in arguments {
        stream.append(&arg.as_ref());
    }

    stream
        .append(&reference_block_id.as_ref())
        .append(&gas_limit)
        .append(&proposal_key_address.as_ref())
        .append(&proposal_key_id)
        .append(&proposal_key_sequence_number)
        .append(&payer.as_ref());

    let authorizers = authorizers.into_iter();

    stream.begin_list(authorizers.len());

    for arg in authorizers {
        stream.append(&arg.as_ref());
    }
}
