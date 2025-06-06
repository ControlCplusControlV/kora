use super::{solana_signer::SolanaMemorySigner, vault_signer::VaultSigner};
use crate::error::KoraError;
use privy_rust::PrivySigner;
use solana_sdk::signature::Signature as SolanaSignature;
use std::error::Error;
use tk_rs::TurnkeySigner;

#[derive(Debug, Clone)]
pub struct Signature {
    /// The raw bytes of the signature
    pub bytes: Vec<u8>,
    /// Whether this is a partial signature or a complete signature
    pub is_partial: bool,
}

/// A trait for signing arbitrary messages
pub trait Signer {
    /// The error type returned by signing operations
    type Error: Error + Send + Sync + 'static;

    fn sign(
        &self,
        message: &[u8],
    ) -> impl std::future::Future<Output = Result<Signature, Self::Error>> + Send;

    fn sign_solana(
        &self,
        message: &[u8],
    ) -> impl std::future::Future<Output = Result<SolanaSignature, Self::Error>> + Send;
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum KoraSigner {
    Memory(SolanaMemorySigner),
    Privy(PrivySigner),
    Turnkey(TurnkeySigner),
    Vault(VaultSigner),
}

impl KoraSigner {
    pub fn solana_pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        match self {
            KoraSigner::Memory(signer) => signer.solana_pubkey(),
            KoraSigner::Privy(signer) => signer.solana_pubkey(),
            KoraSigner::Turnkey(signer) => signer.solana_pubkey(),
            KoraSigner::Vault(signer) => signer.solana_pubkey(),
        }
    }
}

impl super::Signer for KoraSigner {
    type Error = KoraError;

    async fn sign(&self, message: &[u8]) -> Result<super::Signature, Self::Error> {
        match self {
            KoraSigner::Memory(signer) => signer.sign(message).await,
            KoraSigner::Privy(signer) => {
                let sig = signer.sign_solana(message).await?;
                Ok(super::Signature { bytes: sig.as_ref().to_vec(), is_partial: false })
            }
            KoraSigner::Turnkey(signer) => {
                let sig = signer.sign(message).await?;
                Ok(super::Signature { bytes: sig, is_partial: false })
            }
            KoraSigner::Vault(signer) => signer.sign(message).await,
        }
    }

    async fn sign_solana(
        &self,
        message: &[u8],
    ) -> Result<solana_sdk::signature::Signature, Self::Error> {
        match self {
            KoraSigner::Memory(signer) => signer.sign_solana(message).await,
            KoraSigner::Privy(signer) => Ok(signer.sign_solana(message).await?),
            KoraSigner::Turnkey(signer) => {
                signer.sign_solana(message).await.map_err(KoraError::from)
            }
            KoraSigner::Vault(signer) => signer.sign_solana(message).await,
        }
    }
}
