use crate::{
    error,
    vtable::{VTable, VTableFieldOffset, VTableFieldOffsets},
};

#[cfg(feature = "crypto")]
use crate::crypto::{
    digest::{self, Digest, FixedOutput, FixedOutputReset, HashMarker},
    ed25519,
};

pub trait DocBuf {
    // inner type for the Document Buffer
    type Doc: DocBuf + std::fmt::Debug;

    /// Return the virtual table (vtable) for the document buffer
    fn vtable() -> Result<&'static VTable<'static>, error::Error>;

    /// Convert the document to a document buffer
    fn to_docbuf<'a>(&self, buffer: &'a mut Vec<u8>) -> Result<VTableFieldOffsets, error::Error>;

    /// Convert the document buffer to a document
    fn from_docbuf<'a>(buffer: &'a mut Vec<u8>) -> Result<Self::Doc, error::Error>;

    /// Write the document buffer to a file
    #[cfg(feature = "std")]
    fn to_file(&self, path: impl Into<std::path::PathBuf>) -> Result<(), error::Error> {
        use std::fs::File;
        use std::io::Write;

        let mut buffer = Vec::with_capacity(1024);
        self.to_docbuf(&mut buffer)?;

        let mut file = File::create(path.into())?;
        file.write_all(&buffer)?;

        Ok(())
    }
}

#[cfg(feature = "crypto")]
pub trait DocBufCrypto: DocBuf {
    #[cfg(feature = "ed25519")]
    fn sign<D>(
        &self,
        digest: &mut D,
        signer: impl ed25519::signature::Signer<ed25519::Signature>,
    ) -> Result<ed25519::Signature, error::Error>
    where
        D: Default + Digest + Clone + FixedOutput + FixedOutputReset + HashMarker + 'static,
    {
        let data = self.hash(digest)?;
        let signature = signer.try_sign(&data)?;
        Ok(signature)
    }

    #[cfg(feature = "ed25519")]
    fn verify<D>(
        &self,
        digest: &mut D,
        signature: &ed25519::Signature,
        verifier: impl ed25519::signature::Verifier<ed25519::Signature>,
    ) -> Result<(), error::Error>
    where
        D: Default + Digest + Clone + FixedOutput + FixedOutputReset + HashMarker + 'static,
    {
        // Re-compute the data hash message that was signed.
        let data = self.hash(digest)?;

        // Verify the signature against the hashed payload.
        verifier.verify(&data, signature)?;

        Ok(())
    }

    #[cfg(feature = "digest")]
    fn hash<D>(&self, digest: &mut D) -> Result<Vec<u8>, error::Error>
    where
        D: Default + Digest + Clone + FixedOutput + FixedOutputReset + HashMarker + 'static,
    {
        // Hash the document buffer contents
        use digest::DynDigest;
        let mut bytes = Vec::with_capacity(1024);
        self.to_docbuf(&mut bytes)?;

        let output_size = digest.output_size();
        let mut result = vec![0u8; output_size];

        Digest::update(digest, &bytes);

        // Reset the digest after finalizing the hash
        // This allows the digest to be re-usable
        Digest::finalize_into_reset(digest, result.as_mut_slice().into());

        // Return the hash result
        Ok(result.to_vec())
    }
}

/// This trait is used by the vtable to read a field from the
/// document buffer, rather than deserializing the entire document.
pub trait DocBufMap<T> {
    // Read a field from the document buffer, given the field offset.
    fn docbuf_map(
        &self,
        buffer: &[u8],
        offset: &VTableFieldOffset,
    ) -> Result<T, crate::vtable::Error>;
}

/// DocBufEncodeField is a trait used to serialize a field to the document buffer.
pub trait DocBufEncodeField {
    fn encode(&self, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, crate::vtable::Error>;
}

/// DocBufDecodeField is a trait used to deserialize a field from the document buffer.
pub trait DocBufDecodeField<T> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<T, crate::vtable::Error>;
}
