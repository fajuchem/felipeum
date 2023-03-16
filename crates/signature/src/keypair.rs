use bip39::{Language, Mnemonic, MnemonicType, Seed};
use ed25519_dalek::Verifier;
use ed25519_dalek::{ed25519, ExpandedSecretKey, SignatureError};
use std::error::Error;

use rand::{rngs::OsRng, CryptoRng, RngCore};

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

    pub fn secret(&self) -> &[u8] {
        self.0.secret.as_bytes()
    }

    pub fn public_key(&self) -> &[u8] {
        self.0.public.as_bytes()
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

pub fn new_keypair() -> Result<Keypair, Box<dyn Error>> {
    let mnemonic_type = MnemonicType::for_word_count(12)?;
    let mnemonic = Mnemonic::new(mnemonic_type, Language::English);
    let passphrase: &str = "";
    let seed = Seed::new(&mnemonic, &passphrase);
    let keypair = keypair_from_seed(seed.as_bytes())?;

    Ok(keypair)
}
