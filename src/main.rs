use anyhow::{Result, anyhow};
use bech32::{Bech32, Hrp};
use blake2::{Blake2b, Digest, digest::consts::U28};
use frost_ed25519 as frost;
use pallas::crypto::key::ed25519::SecretKey;
use rand::SeedableRng;
use std::{collections::BTreeMap, io, time::Duration};
use tokio::time::sleep;
use tx3_sdk::trp::{
    SubmitWitness, VKeyWitness,
    args::{ArgValue, BytesEncoding, BytesEnvelope},
};
mod lib;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate key shares in a trusted way; can also do so via DKG;
    println!("===========================================");
    println!("| Key Generation                          |");
    println!("| Let's generate a 5/5 threshold multisig |");
    println!("===========================================");
    let mut rng = rand::rngs::StdRng::from_seed([0x00; 32]);
    let max_signers = 5;
    let min_signers = 5;
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        max_signers,
        min_signers,
        frost::keys::IdentifierList::Default,
        &mut rng,
    )?;

    // Validate each key share
    let mut key_packages: BTreeMap<_, _> = BTreeMap::new();
    let mut idx = 0;
    for (identifier, secret_share) in shares {
        println!("Key {}: {}", idx, hex::encode(&secret_share.serialize()?));
        let key_package = frost::keys::KeyPackage::try_from(secret_share)?;
        key_packages.insert(identifier, key_package);
        idx += 1;
    }

    println!();
    println!("====================================================");
    println!("| Public Key                                       |");
    println!("| These private key shares produce this public key |");
    println!("====================================================");

    // Get the wallet funded
    let pub_key_bytes = pubkey_package.verifying_key().serialize()?;
    let mut hasher = Blake2b::<U28>::new();
    hasher.update(&pub_key_bytes);
    let pub_key_hash = hasher.finalize().to_vec();
    let address = bech32::encode::<Bech32>(
        Hrp::parse("addr_test")?,
        &[&[0x60], &pub_key_hash[..]].concat(),
    )?;

    println!("FROST address: {}", address);

    println!();
    println!("=====================================================");
    println!("| Fund the FROST key                                |");
    println!("| We're going to send 100k ADA to the FROST address |");
    println!("=====================================================");

    let faucet = ArgValue::from("addr_test1vzppstljc03v2chrl6fzkgha8qjk2qfmhwc3eqxaxducxaghcpu9e");
    let frost = ArgValue::from(address);

    let fund_params = lib::FundParams {
        faucet: faucet.clone(),
        frost: frost.clone(),
    };
    let fund_tx = lib::PROTOCOL.fund_tx(fund_params).await?;
    let sk: SecretKey = TryInto::<[u8; 32]>::try_into(hex::decode(
        "6634306234633532343163363863616663373264373565616464396339376565",
    )?)
    .map_err(|_| anyhow!("failed to parse secret key"))?
    .into();
    let pk = sk.public_key();
    let pk_envelope = BytesEnvelope {
        content: hex::encode(&pk.as_ref()),
        encoding: BytesEncoding::Hex,
    };
    let signature = sk.sign(hex::decode(fund_tx.hash.clone())?);
    let sig_envelope = BytesEnvelope {
        content: hex::encode(&signature.as_ref()),
        encoding: BytesEncoding::Hex,
    };
    lib::PROTOCOL
        .submit(
            fund_tx,
            vec![SubmitWitness::VKey(VKeyWitness {
                key: pk_envelope,
                signature: sig_envelope,
            })],
        )
        .await?;

    println!("Transaction submitted, waiting for settlement");

    sleep(Duration::from_secs(5)).await;

    let mut nonces_map = BTreeMap::new();
    let mut commitments_map = BTreeMap::new();

    // ROUND 1: Each participant willing to sign generates nonces and commitments
    for participant_index in 1..=min_signers {
        let participant_identifier = participant_index.try_into().expect("should be nonzero");
        let key_package = &key_packages[&participant_identifier];
        // Generate one (1) nonce and one SigningCommitments instance for each
        // participant, up to _threshold_.
        let (nonces, commitments) = frost::round1::commit(key_package.signing_share(), &mut rng);
        // In practice, the nonces must be kept by the participant to use in the
        // next round, while the commitment must be sent to the coordinator
        // (or to every other participant if there is no coordinator) using
        // an authenticated channel.
        nonces_map.insert(participant_identifier, nonces);
        commitments_map.insert(participant_identifier, commitments);
    }

    println!();
    println!("=====================================================");
    println!("| Spending PlutusTx                                 |");
    println!("| Now running a SC with FROST as the collateral     |");
    println!("=====================================================");

    // Prompt for the tx bytes utilizing the collateral from the address
    let mut signature_shares = BTreeMap::new();

    let redeemer = ArgValue::from("0x00");
    let spend_params = lib::SpendParams {
        faucet,
        frost,
        redeemer,
    };
    let spend_tx = lib::PROTOCOL.spend_tx(spend_params).await?;
    println!("Tx: {}", spend_tx.tx);
    let spend_tx_hash = spend_tx.hash.clone();

    //let mut spend_tx = String::new();
    //io::stdin().read_line(&mut spend_tx);
    let message = hex::decode(spend_tx_hash)?;
    let signing_package = frost::SigningPackage::new(commitments_map, &message);

    // And have each participant generate their signature
    for participant_identifier in nonces_map.keys() {
        let key_package = &key_packages[participant_identifier];

        let nonces = &nonces_map[participant_identifier];

        // Each participant generates their signature share.
        let signature_share = frost::round2::sign(&signing_package, nonces, key_package)?;

        // In practice, the signature share must be sent to the Coordinator
        // using an authenticated channel.
        signature_shares.insert(*participant_identifier, signature_share);
    }

    // We can now aggregate the signature and submit it to the chain
    let group_signature = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)?;

    let signature = group_signature.serialize()?;
    println!("Publickey: {}", hex::encode(&pub_key_bytes));
    println!("Signature: {}", hex::encode(&signature));

    let frost_pk_envelope = BytesEnvelope {
        content: hex::encode(pub_key_bytes),
        encoding: BytesEncoding::Hex,
    };
    let frost_sig_envelope = BytesEnvelope {
        content: hex::encode(signature),
        encoding: BytesEncoding::Hex,
    };

    println!("Submitting transaction...");

    lib::PROTOCOL
        .submit(
            spend_tx,
            vec![SubmitWitness::VKey(VKeyWitness {
                key: frost_pk_envelope,
                signature: frost_sig_envelope,
            })],
        )
        .await?;

    println!("Done!");
    Ok(())
}
