use rlp::RlpStream;

pub fn rlp_encode_transaction_envelope(
    s: &mut RlpStream,
    script: impl AsRef<[u8]>,
    arguments: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
    reference_block_id: impl AsRef<[u8]>,
    gas_limit: u64,
    proposal_key_address: impl AsRef<[u8]>,
    proposal_key_id: u64,
    proposal_key_sequence_number: u64,
    payer: impl AsRef<[u8]>,
    authorizers: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl AsRef<[u8]>>>,
    payload_signatures: impl IntoIterator<
        IntoIter = impl ExactSizeIterator<Item = (impl AsRef<[u8]>, u32, impl AsRef<[u8]>)>,
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
            .append(&signer_index.as_ref())
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
    proposal_key_id: u64,
    proposal_key_sequence_number: u64,
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
        .append_iter(lpad(&proposal_key_address, 8))
        .append(&proposal_key_id)
        .append(&proposal_key_sequence_number)
        .append_iter(lpad(&payer, 8));

    let authorizers = authorizers.into_iter();

    stream.begin_list(authorizers.len());

    for authorizer in authorizers {
        stream.append_iter(lpad(&authorizer, 8));
    }
}

// Given a slice of bytes and the padded length, returns an iterator of the data left-padded with zeros.
fn lpad(data: &impl AsRef<[u8]>, padded_len: usize) -> impl Iterator<Item = u8> + '_ {
    let data = data.as_ref();

    std::iter::repeat(0)
        .take(padded_len - data.as_ref().len())
        .chain(data.iter().copied())
}
