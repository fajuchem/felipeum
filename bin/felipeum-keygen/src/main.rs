use std::error::Error;

use ed25519_dalek::Verifier;
use ed25519_dalek::{ed25519, ExpandedSecretKey, SignatureError};

use rand::{rngs::OsRng, CryptoRng, RngCore};

// #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct Pubkey(pub(crate) [u8; 32]);
//
// impl Pubkey {
//     pub fn new(pubkey_vec: &[u8]) -> Self {
//         Self(
//             <[u8; 32]>::try_from(<&[u8]>::clone(&pubkey_vec))
//                 .expect("Slice must be the same length as a Pubkey"),
//         )
//     }
// }
// impl fmt::Display for Pubkey {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", bs58::encode(self.0).into_string())
//     }
// }
//
#[derive(Debug)]
pub struct Keypair(ed25519_dalek::Keypair);

impl Keypair {
    pub fn generate<R>(csprng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        Self(ed25519_dalek::Keypair::generate(csprng))
    }

    pub fn new() -> Self {
        let mut rng = OsRng::default();
        Self::generate(&mut rng)
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<ed25519::Signature, SignatureError> {
        let expanded: ExpandedSecretKey = (&self.0.secret).into();

        Ok(expanded.sign(&message, &self.0.public).into())
    }

    pub fn verify(
        &self,
        message: &[u8],
        signature: &ed25519::Signature,
    ) -> Result<(), SignatureError> {
        self.0.public.verify(message, signature)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ed25519_dalek::SignatureError> {
        ed25519_dalek::Keypair::from_bytes(bytes).map(Self)
    }
}

pub fn keypair_from_seed(seed: &[u8]) -> Result<Keypair, Box<dyn Error>> {
    if seed.len() < ed25519_dalek::SECRET_KEY_LENGTH {
        return Err("Seed is too short".into());
    }
    let secret = ed25519_dalek::SecretKey::from_bytes(&seed[..ed25519_dalek::SECRET_KEY_LENGTH])
        .map_err(|e| e.to_string())?;
    let public = ed25519_dalek::PublicKey::from(&secret);
    let dalek_keypair = ed25519_dalek::Keypair { secret, public };
    Ok(Keypair(dalek_keypair))
}

pub struct Signature(Vec<u8>);

impl Signature {
    pub fn new(signature_slice: &[u8]) -> Self {
        Self(signature_slice.to_owned())
    }

    fn verify_verbose(
        &self,
        pubkey_bytes: &[u8],
        message_bytes: &[u8],
    ) -> Result<(), SignatureError> {
        let publickey = ed25519_dalek::PublicKey::from_bytes(pubkey_bytes)?;
        let signature = self.0.as_slice().try_into()?;
        publickey.verify_strict(message_bytes, &signature)
    }

    pub fn verify(&self, pubkey_bytes: &[u8], message_bytes: &[u8]) -> bool {
        self.verify_verbose(pubkey_bytes, message_bytes).is_ok()
    }
}

fn main() {
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let message = "hello".as_bytes();

    let expanded: ExpandedSecretKey = (&keypair.secret).into();
    let signature: ed25519_dalek::Signature = expanded.sign(&message, &keypair.public).into();

    println!("Secret key: {:?}", keypair.public.as_bytes());
    println!("Public key: {:?}", keypair.secret.as_bytes());
    println!("Message: {:?}", message);
    println!("Signature: {:?}", signature.to_bytes());

    {
        let sig = Signature::new(&signature.to_bytes());
        let is_valid = sig.verify(keypair.public.as_bytes(), message);
        match is_valid {
            true => println!("Signature has been proven!"),
            false => println!("Signature has NOT been proven!"),
        }
    }

    //     let mnemonic_type = MnemonicType::for_word_count(12)?;
    //     let mnemonic = Mnemonic::new(mnemonic_type, Language::English);
    //     let passphrase: &str = "";
    //     let seed = Seed::new(&mnemonic, &passphrase);
    //     let keypair = keypair_from_seed(seed.as_bytes())?;
    //     println!("pubkey: {}", keypair.pubkey().to_string());
    //
    //     let signed = keypair.sign_message("hello".as_bytes()).unwrap();
    //     println!("signed: {}", signed);
    //
    //     let sig = Signature::new(signed.to_string().as_bytes());
    //     match sig.verify(keypair.pubkey().to_string().as_bytes(), "hello".as_bytes()) {
    //         Ok(_) => println!("okkkkkkkkkkkkkkkk"),
    //         Err(_) => println!("Errrrrrrrrrrrrrrrrrrr"),
    //     };
    //
    //     println!(
    //         "\npubkey: {}\n keypair: {:?}",
    //         keypair.pubkey(),
    //         hash_to_hex(keypair.0.to_bytes())
    //     );
    //
    //     Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sign() {
        let message_bytes = "hello".as_bytes();
        let secret = hex::decode(
            "7c60b0a205667ede90d57e8c526d96bdb080bb0bb6a79b12e7e60703c15271d7".as_bytes(),
        )
        .unwrap();
        println!("{:?}", secret);

        let keypair = Keypair::from_bytes(&secret).unwrap();
        let signature = keypair.sign_message(message_bytes).unwrap();

        let pubkey_bytes = hex::decode(
            "b73657d720bf4bc498ddfb350bbfd6052ef9b3bc2e61e3eeaf0c7947f5febad9".as_bytes(),
        )
        .unwrap();

        let sig = Signature::new(&signature.to_bytes());
        let is_valid = sig.verify(&pubkey_bytes, message_bytes);
        assert!(is_valid);
    }
    #[test]
    fn test_signature_fromstr() {
        let message_bytes = "hello".as_bytes();
        let pubkey_bytes = &[
            248, 193, 148, 30, 173, 70, 141, 182, 87, 18, 216, 11, 61, 166, 5, 221, 197, 83, 126,
            24, 35, 66, 251, 210, 15, 72, 120, 125, 23, 126, 202, 13,
        ];
        let signature_bytes = &[
            165, 73, 149, 3, 108, 164, 213, 37, 199, 58, 121, 167, 33, 24, 196, 222, 86, 221, 48,
            120, 65, 26, 255, 238, 173, 124, 128, 154, 40, 47, 78, 159, 48, 49, 198, 229, 39, 252,
            221, 95, 86, 157, 74, 30, 234, 43, 228, 251, 68, 43, 136, 215, 9, 128, 127, 76, 140,
            124, 200, 87, 146, 178, 68, 13,
        ];
        let sig = Signature::new(signature_bytes);
        let is_valid = sig.verify(pubkey_bytes, message_bytes);
        assert!(is_valid);
    }
}
