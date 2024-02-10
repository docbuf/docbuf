use crate::{error, vtable::VTable};

#[cfg(feature = "crypto")]
use crate::crypto::{ed25519, digest::{self, Digest, HashMarker, FixedOutputReset, FixedOutput}};


pub trait DocBuf {
    // inner type for the Document Buffer
    type Doc;

    // Document buffer struct with additional fields for options and validation
    type DocBuf: DocBuf;

    /// Consumes the document buffer and returns the inner document type
    fn to_doc(self) -> Self::Doc;

    /// Returns a reference to the inner document type
    fn as_doc(&self) -> &Self::Doc;

    /// From the document type, return the document buffer
    fn from_doc(doc: Self::Doc) -> Self;

    /// Convert the document to a document buffer
    fn to_docbuf(&self) -> Result<Vec<u8>, error::Error>;

    /// Convert the document buffer to a document
    fn from_docbuf(buf: &[u8]) -> Result<Self::DocBuf, error::Error>;

    /// Return the virtual table (vtable) for the document buffer
    fn vtable() -> Result<VTable, error::Error>;
}

#[cfg(feature = "crypto")]
pub trait DocBufCrypto: DocBuf {

    #[cfg(feature = "ed25519")]
    fn sign<D>(
        &self, 
        digest: &mut D,
        signer: impl ed25519::signature::Signer<ed25519::Signature>,
    ) -> Result<ed25519::Signature, error::Error>
        where D: 
            Default + Digest + Clone + FixedOutput + FixedOutputReset + HashMarker
    {
        let data = self.hash(&mut digest.clone())?;
        let signature = signer.try_sign(&data)?;
        Ok(signature)
    }

    #[cfg(feature = "ed25519")]
    fn verify<D>(
        &self,
        digest: &mut D,
        signature: &ed25519::Signature, 
        verifier: impl ed25519::signature::Verifier<ed25519::Signature>
    ) -> Result<(), error::Error> 
     where D: 
        Default + Digest + Clone + FixedOutput + FixedOutputReset + HashMarker
    {
        // Re-compute the data hash message that was signed.
        let data = self.hash(&mut digest.clone())?;

        // Verify the signature against the hashed payload.
        verifier.verify(&data, signature)?;

        Ok(())
    }

    #[cfg(feature = "digest")]
    fn hash<D>(&self, digest: &mut D) -> Result<Vec<u8>, error::Error>
        where D: Default + Digest + Clone + FixedOutput + FixedOutputReset + HashMarker
    {
        // Hash the document buffer contents
        let bytes = self.to_docbuf()?;

        // Create a buffer for the hash result
        let mut result = Vec::new();

        Digest::update(digest, &bytes);

        // Reset the digest after finalizing the hash
        // This allows the digest to be re-usable
        Digest::finalize_into_reset(digest, result.as_mut_slice().into());

        // Return the hash result
        Ok(result.to_vec())
    }
}