use ethers::{
    abi::{encode, Token},
    prelude::*,
    utils::keccak256,
};
use platform_lib_noah::noah_algebra::{bls12_381::BLSScalar, bn254::BN254Scalar, prelude::Scalar};
use platform_lib_noah::noah_crypto::anemoi_jive::{
    bls12_381_deprecated::AnemoiJive381Deprecated, AnemoiJive, AnemoiJive254,
};

fn old_hash(input: &[u8]) -> Vec<u8> {
    let num_elems = input.len() / 32;
    let mut field_elems = Vec::with_capacity(num_elems);
    for i in 0..num_elems {
        let res = BLSScalar::from_bytes(
            &input[i * 32..(i + 1) * 32]
                .iter()
                .rev()
                .copied()
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        field_elems.push(res);
    }

    let mut res = AnemoiJive381Deprecated::eval_variable_length_hash(&field_elems).to_bytes();
    res.reverse();
    res.to_vec()
}

fn new_hash(input: &[u8]) -> Vec<u8> {
    let num_elems = input.len() / 32;
    let mut field_elems = Vec::with_capacity(num_elems);
    for i in 0..num_elems {
        let res = BN254Scalar::from_bytes(
            &input[i * 32..(i + 1) * 32]
                .iter()
                .rev()
                .copied()
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        field_elems.push(res);
    }

    let mut res = AnemoiJive254::eval_variable_length_hash(&field_elems).to_bytes();
    res.reverse();
    res.to_vec()
}

const ERC20_PREFIX: &'static str = "Findora ERC20 Asset Type";
const ERC721_PREFIX: &'static str = "Findora ERC721 Asset Type";
const ERC1155_PREFIX: &'static str = "Findora ERC1155 Asset Type";

const RPC_UTXO_NODE: &'static str = "https://prod-mainnet.prod.findora.org";
const RPC_EVM_NODE: &'static str = "https://rpc-mainnet.findora.org";

fn main() {
    let erc20_prefix = Token::FixedBytes(keccak256(ERC20_PREFIX).to_vec());
    let erc721_prefix = Token::FixedBytes(keccak256(ERC721_PREFIX).to_vec());
    let erc1155_prefix = Token::FixedBytes(keccak256(ERC1155_PREFIX).to_vec());

    // 1. TODO list all assets1

    // 2. TODO check assets in in prmisxx

    // 3. check the old hash
    let address: Address = "0xbB64D716FAbDEC3a106bb913Fb4f82c1EeC851b8"
        .parse()
        .unwrap();
    let bytes = encode(&[erc20_prefix, Token::Address(address)]);

    let hash1 = old_hash(&bytes);
    println!("asset1: {}", hex::encode(&hash1));

    // 4. renew hash
    let hash2 = new_hash(&bytes);
    println!("asset2: {}", hex::encode(&hash2));

    // 5. TODO contracts call
}
