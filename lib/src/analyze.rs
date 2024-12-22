use std::fmt::Display;

use bytesize::ByteSize;
use digest::Digest;
use hex::encode;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use tabled::builder::Builder;

pub struct Report {
    pub size: String,
    pub shannon_entropy: f64,
    pub absolute_entropy: f64,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String
}

impl Display for Report {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut builder = Builder::new();

        builder.push_record(["Size", &self.size]);
        builder.push_record(["Entropy (Sh)", &self.shannon_entropy.to_string()]);
        builder.push_record(["Entropy (So)", &self.absolute_entropy.to_string()]);
        builder.push_record(["MD5", &self.md5]);
        builder.push_record(["SHA1", &self.sha1]);
        builder.push_record(["SHA2-256", &self.sha256]);
        builder.push_record(["SHA2-512", &self.sha512]);

        let table = builder.build();

        write!(formatter, "{table}")
    }
}

pub fn analyze(buffer: Vec<u8>) -> Report {
    let length = buffer.len();
    let mut md5 = Md5::new();
    let mut sha1 = Sha1::new();
    let mut sha256 = Sha256::new();
    let mut sha512 = Sha512::new();

    md5.update(&buffer);
    sha1.update(&buffer);
    sha256.update(&buffer);
    sha512.update(&buffer);

    Report {
        size: ByteSize::b(length as u64).to_string(),
        shannon_entropy: shannon_entropy(&buffer),
        absolute_entropy: normalized_absolute_entropy(&buffer),
        md5: encode(md5.finalize()),
        sha1: encode(sha1.finalize()),
        sha256: encode(sha256.finalize()),
        sha512: encode(sha512.finalize())
    }
}

/// Calculate the Shannon entropy.
fn shannon_entropy(buffer: &[u8]) -> f64 {
    let length = buffer.len();
    let mut entropy = 0.0_f64;
    let mut counts = [0_u64; 256];

    // Create a histogram of the number of times each symbol occurred.
    for byte in buffer { counts[*byte as usize] += 1; }

    // Calculate the Shannon entropy.
    for count in counts {
        if count == 0 { continue; }

        let value = (count as f64) / (length as f64);

        entropy -= value * value.log2();
    }

    entropy
}

/// Calculate the normalized absolute entropy.
fn normalized_absolute_entropy(buffer: &[u8]) -> f64 {
    let length = buffer.len();

    // Calculate the Shannon entropy.
    let entropy = shannon_entropy(buffer);

    // Calculate and return the normalized absolute entropy.
    (length as f64) * entropy / 8.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_bytes_has_zero_shannon_entropy() {
        assert_eq!(shannon_entropy(b""), 0.0);
    }

    #[test]
    fn equal_distribution_has_full_shannon_entropy() {
        let mut bytes = [0_u8; 256];

        for index in 0..256 { bytes[index] = index as u8; }

        assert_eq!(shannon_entropy(&bytes), 8.0);
    }

    #[test]
    fn zero_bytes_has_zero_absolute_entropy() {
        assert_eq!(normalized_absolute_entropy(b""), 0.0);
    }

    #[test]
    fn equal_distribution_has_full_absolute_entropy() {
        let mut bytes = [0_u8; 256];

        for index in 0..256 { bytes[index] = index as u8; }

        assert_eq!(normalized_absolute_entropy(&bytes), 256.0);
    }
}
