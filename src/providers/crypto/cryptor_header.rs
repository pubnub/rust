use crate::{
    core::PubNubError,
    lib::alloc::{vec, vec::Vec},
};

/// Maximum cryptor identifier length.
const IDENTIFIER_LENGTH: usize = 4;

/// Legacy cryptor identifier.
const NULL_IDENTIFIER: &[u8; 4] = &[0x00u8; 4];

const SENTINEL: &[u8; 4] = b"PNED";

/// Maximum known header version.
///
/// It should be possible to process headers up to this version.
const MAX_VERSION: u8 = 1;

/// Cryptor data header.
///
/// This instance used to parse header from received data and encode into binary
/// for sending.
#[derive(Debug)]
pub(crate) enum CryptorHeader {
    V1 {
        /// Unique cryptor identifier.
        ///
        /// Identifier of the cryptor which has been used to encrypt data.
        identifier: [u8; 4],

        /// Size of cryptor-defined data.
        data_size: usize,
    },

    /// Cryptor data doesn't have header.
    None,
}

impl CryptorHeader {
    /// Create new crypto header.
    pub fn new(identifier: [u8; 4], cryptor_metadata: &Option<Vec<u8>>) -> Self {
        Self::V1 {
            data_size: cryptor_metadata
                .as_ref()
                .map(|metadata| metadata.len())
                .unwrap_or(0),
            identifier,
        }
    }

    /// Overall header size.
    ///
    /// Full header size which includes:
    /// * sentinel
    /// * version
    /// * cryptor identifier
    /// * cryptor data size
    /// * cryptor-defined fields size.
    pub fn len(&self) -> usize {
        match self.data_size() {
            Some(size) => {
                SENTINEL.len() + 1 + IDENTIFIER_LENGTH + if size < 255 { 1 } else { 3 } + size
            }
            None => 0,
        }
    }

    /// Header version value.
    pub fn version(&self) -> u8 {
        match self {
            CryptorHeader::V1 { .. } => 1,
            _ => 0,
        }
    }

    /// Cryptor-defined data size.
    pub fn data_size(&self) -> Option<usize> {
        match self {
            CryptorHeader::V1 { data_size, .. } => Some(*data_size),
            _ => None,
        }
    }

    /// Cryptor identifier.
    pub fn identifier(&self) -> Option<[u8; 4]> {
        match self {
            CryptorHeader::V1 { identifier, .. } => {
                identifier.ne(NULL_IDENTIFIER).then_some(*identifier)
            }
            _ => None,
        }
    }
}

/// Encode composed header into binary array.
impl From<CryptorHeader> for Vec<u8> {
    fn from(value: CryptorHeader) -> Self {
        // Creating header only if specified.
        let Some(identifier) = value.identifier() else {
            return vec![];
        };

        let mut data = vec![0; value.len()];
        // Adding sentinel into header.
        data.splice(0..SENTINEL.len(), SENTINEL.to_vec());
        let mut pos = SENTINEL.len();

        // Adding header version.
        data[pos] = value.version();
        pos += 1;

        // Add cryptor identifier if it is not for legacy cryptor.
        identifier.ne(NULL_IDENTIFIER).then(|| {
            data.splice(pos..(pos + identifier.len()), identifier);
            pos += identifier.len();
        });

        // Adding cryptor header size.
        let header_size = value.data_size().unwrap_or(0);
        if header_size < 255 {
            data[pos] = header_size as u8;
        } else {
            data.splice(
                pos..(pos + 3),
                vec![255, (header_size >> 8) as u8, (header_size & 0xFF) as u8],
            );
        }

        data
    }
}

/// Decode and parse header from binary array.
impl TryFrom<&Vec<u8>> for CryptorHeader {
    type Error = PubNubError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        // Data is too short to be encrypted. Assume legacy cryptor without
        // header.
        let Some(sentinel) = value.len().ge(&4).then(|| &value[0..4]) else {
            return Ok(Self::None);
        };

        // There is no sentinel. Assume legacy cryptor without header.
        if sentinel.ne(SENTINEL) {
            return Ok(Self::None);
        }

        let Some(version) = value.len().ge(&5).then(|| &value[4]) else {
            return Err(PubNubError::Decryption {
                details: "Decrypted data header is malformed.".into(),
            });
        };

        // Check whether version is within known range.
        if *version == 0 || *version > MAX_VERSION {
            return Err(PubNubError::UnknownCryptor {
                details: "Decrypting data created by unknown cryptor.".into(),
            });
        }

        // Retrieving cryptor identifier.
        let mut pos = 5 + IDENTIFIER_LENGTH;
        let Some(identifier) = value.len().ge(&pos).then(|| {
            let mut identifier: [u8; 4] = [0; 4];
            identifier.copy_from_slice(&value[5..pos]);
            identifier
        }) else {
            return Err(PubNubError::Decryption {
                details: "Decrypted data header is malformed.".into(),
            });
        };

        // Retrieving cryptor-defined data size.
        let Some(mut header_size) = value.len().ge(&(pos + 1)).then(|| value[pos] as usize) else {
            return Ok(Self::None);
        };
        pos += 1;
        if header_size == 255 {
            let Some(size_bytes) = value.len().ge(&(pos + 2)).then(|| {
                let mut size_bytes: [u8; 2] = [0; 2];
                size_bytes.clone_from_slice(&value[pos..(pos + 2)]);
                size_bytes
            }) else {
                return Ok(Self::None);
            };
            header_size = u16::from_be_bytes(size_bytes) as usize;
        }

        // Construct header basing on version passed in payload.
        Ok(match version {
            &1 => Self::V1 {
                data_size: header_size,
                identifier,
            },
            _ => Self::None,
        })
    }
}
