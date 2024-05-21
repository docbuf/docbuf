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

    type DocId: Into<[u8; 16]> + From<[u8; 16]> + std::fmt::Debug;

    #[cfg(feature = "uuid")]
    /// Return the Uuid for the document
    ///
    /// Set the docbuf `uuid = true` attribtute on the struct to enable it automatically.
    /// ```
    /// #[docbuf {
    ///    uuid = true
    /// }]
    /// struct MyStruct {
    ///    // ...
    /// }
    /// ```
    ///
    /// By default, this method will error on unimplemented structs.
    fn uuid(&self) -> Result<Self::DocId, error::Error> {
        Err(error::Error::UuidNotImplemented)
    }

    /// Return the virtual table (vtable) for the document buffer
    fn vtable() -> Result<&'static VTable, error::Error>;

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
        Ok(result)
    }
}

/// This trait is used by the vtable to read a field from the
/// document buffer, rather than deserializing the entire document.
pub trait DocBufMap<T> {
    /// Read a field from the document buffer, given the field offset.
    fn docbuf_map(
        &self,
        buffer: &[u8],
        offset: &VTableFieldOffset,
    ) -> Result<T, crate::vtable::Error>;

    /// Replace a field from the document buffer, given the field offset index, and return the new offets.
    /// This will shift the buffer to the right if the new field is larger than the old one. Conversely,
    /// this will shift the buffer to the left if the new field is smaller than the old one.
    fn docbuf_map_replace(
        &self,
        new_value: &T,
        offset: VTableFieldOffset,
        buffer: &mut Vec<u8>,
        offsets: &mut VTableFieldOffsets,
    ) -> Result<VTableFieldOffset, crate::vtable::Error>;
}

/// DocBufEncodeField is a trait used to serialize a field to the document buffer.
pub trait DocBufEncodeField<T> {
    fn encode(
        &self,
        data: &T,
        buffer: &mut Vec<u8>,
    ) -> Result<VTableFieldOffset, crate::vtable::Error>;
}

/// DocBufDecodeField is a trait used to deserialize a field from the document buffer.
pub trait DocBufDecodeField<T> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<T, crate::vtable::Error>;
}

/// DocBufValidateField is a trait used to validate a field from the document buffer.
pub trait DocBufValidateField<T> {
    fn validate(&self, value: &T) -> Result<(), crate::vtable::Error>;
}
