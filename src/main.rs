use ethers::{
    abi::{encode, AbiEncode, Token},
    prelude::*,
    utils::keccak256,
};
use platform_lib_noah::{
    noah_algebra::{bls12_381::BLSScalar, bn254::BN254Scalar, prelude::Scalar},
    noah_crypto::anemoi_jive::{
        bls12_381_deprecated::AnemoiJive381Deprecated, AnemoiJive, AnemoiJive254,
    },
};
use serde::Deserialize;
use std::sync::Arc;

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
const RPC_EVM_ARCH: &'static str = "https://archive.prod.findora.org:8545";

const PRISMXX_HEIGHT_START: u64 = 4004430; // 4004430
const PRISMXX_HEIGHT_END: u64 = 4506595;
const PRISMXX_BRIDGE: &'static str = "0x4672372fDB139B7295Fc59b55b43EC5fF2761A0b";
// const PRISMXX_ASSET: &'static str = "0xa2DF66F3c497e713Da29984C12591fc2e8507442";

abigen!(PrismXXBridge, "./PrismXXBridge.abi.json");
abigen!(PrismXXAsset, "./PrismXXAsset.abi.json");

#[derive(Deserialize, Clone, Debug, EthEvent, Default)]
struct DepositAsset {
    pub asset: H256,
    pub receiver: Bytes,
    pub amount: U256,
    pub decimal: u8,
    pub max_supply: U256,
}

#[tokio::main]
async fn main() {
    let erc20_prefix = Token::FixedBytes(keccak256(ERC20_PREFIX).to_vec());
    let erc721_prefix = Token::FixedBytes(keccak256(ERC721_PREFIX).to_vec());
    let erc1155_prefix = Token::FixedBytes(keccak256(ERC1155_PREFIX).to_vec());

    println!("Start init contracts...");
    let provider = Arc::new(Provider::<Http>::try_from(RPC_EVM_ARCH).unwrap());
    let provider2 = Arc::new(Provider::<Http>::try_from(RPC_EVM_NODE).unwrap());
    let prism_bridge: Address = PRISMXX_BRIDGE.parse().unwrap();
    let bridge_contract = PrismXXBridge::new(prism_bridge, provider.clone());
    let prism_asset: Address = bridge_contract.asset_contract().await.unwrap();
    let asset_contract = PrismXXAsset::new(prism_asset, provider);

    // 1. list all assets1
    let mut from = PRISMXX_HEIGHT_START;
    let mut to = from + 10000;
    let mut assets = vec![];

    loop {
        println!("from: {} to {}", from, to);

        let logs = bridge_contract
            .event::<DepositAsset>()
            .from_block(from)
            .to_block(to)
            .query()
            .await
            .unwrap();
        for log in logs {
            if !assets.contains(&log.asset) {
                assets.push(log.asset);
            }
        }

        if to >= PRISMXX_HEIGHT_END {
            break;
        }

        from = to;
        to = from + 10000;

        if to > PRISMXX_HEIGHT_END {
            to = PRISMXX_HEIGHT_END;
        }
    }

    // handle asset
    let mut i = 0;
    for asset in assets {
        // 2. check assets in in prmisxx
        let info = asset_contract.assets(asset.0).await.unwrap();

        // 3. check the old hash
        let address: Address = info.0;
        let token_id = info.1;
        let token_type = info.2;
        if info.0 != Address::zero() {
            println!("asset0: 0x{}", hex::encode(&asset.0));

            //  check asset type
            let bytes = match token_type {
                1 => {
                    let mut code = [0u8; 32];
                    token_id.to_big_endian(&mut code);
                    let mut code0 = vec![0u8; 32];
                    let mut code1 = vec![0u8; 32];
                    code0[0..31].copy_from_slice(&code[0..31]);
                    code1[0] = code[31];
                    encode(&[
                        erc721_prefix.clone(),
                        Token::Address(address),
                        Token::FixedBytes(code0),
                        Token::FixedBytes(code1),
                    ])
                }
                2 => {
                    let mut code = [0u8; 32];
                    token_id.to_big_endian(&mut code);
                    let mut code0 = vec![0u8; 32];
                    let mut code1 = vec![0u8; 32];
                    code0[0..31].copy_from_slice(&code[0..31]);
                    code1[0] = code[31];
                    encode(&[
                        erc1155_prefix.clone(),
                        Token::Address(address),
                        Token::FixedBytes(code0),
                        Token::FixedBytes(code1),
                    ])
                }
                _ => encode(&[erc20_prefix.clone(), Token::Address(address)]),
            };

            let hash1 = old_hash(&bytes);
            println!("asset1: 0x{}", hex::encode(&hash1));

            // 4. renew hash
            let hash2 = new_hash(&bytes);
            println!("asset2: 0x{}", hex::encode(&hash2));

            // 5. TODO renew contracts call
            i += 1;
            println!("-------------------------------- {}", i);
        }
    }
}
