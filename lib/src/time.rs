use clap::ValueEnum;
use chrono::{Local, Utc};

const DEFAULT_FORMAT: &str = "%F %T";

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TimestampFormats {
    ISO8601,
    RFC2822,
    RFC3339
}

pub fn create_timestamp_utc(format: Option<TimestampFormats>) -> Vec<u8> {
    let now = Utc::now();
    let timestamp = match format {
        Some(TimestampFormats::RFC2822) => now.to_rfc2822().to_string(),
        Some(TimestampFormats::RFC3339) | Some(TimestampFormats::ISO8601) => now.to_rfc3339().to_string(),
        None => now.format(DEFAULT_FORMAT).to_string()
    };

    timestamp.into_bytes()
}

pub fn create_timestamp_local(format: Option<TimestampFormats>) -> Vec<u8> {
    let now = Local::now();
    let timestamp = match format {
        Some(TimestampFormats::RFC2822) => now.to_rfc2822().to_string(),
        Some(TimestampFormats::RFC3339) | Some(TimestampFormats::ISO8601) => now.to_rfc3339().to_string(),
        None => now.format(DEFAULT_FORMAT).to_string()
    };

    timestamp.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn creates_timestamp_utc() {
        let timestamp = create_timestamp_utc(None);
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_iso8601_timestamp_utc() {
        let timestamp = create_timestamp_utc(Some(TimestampFormats::ISO8601));
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6,9}[+-]\d{2}:\d{2}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_rfc2822_timestamp_utc() {
        let timestamp = create_timestamp_utc(Some(TimestampFormats::RFC2822));
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"[a-zA-Z]{3}, \d{1,2} [a-zA-Z]{3} \d{4} \d{2}:\d{2}:\d{2} [+-]\d{4}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_rfc3339_timestamp_utc() {
        let timestamp = create_timestamp_utc(Some(TimestampFormats::RFC3339));
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6,9}[+-]\d{2}:\d{2}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_timestamp_local() {
        let timestamp = create_timestamp_local(None);
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_iso8601_timestamp_local() {
        let timestamp = create_timestamp_local(Some(TimestampFormats::ISO8601));
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6,9}[+-]\d{2}:\d{2}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_rfc2822_timestamp_local() {
        let timestamp = create_timestamp_local(Some(TimestampFormats::RFC2822));
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"[a-zA-Z]{3}, \d{1,2} [a-zA-Z]{3} \d{4} \d{2}:\d{2}:\d{2} [+-]\d{4}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }

    #[test]
    fn creates_rfc3339_timestamp_local() {
        let timestamp = create_timestamp_local(Some(TimestampFormats::RFC3339));
        let string = String::from_utf8(timestamp).unwrap();
        let pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6,9}[+-]\d{2}:\d{2}";
        let regex = Regex::new(pattern).unwrap();

        assert!(regex.is_match(&string), "assertion 'matches regular expression' failed\n\tpattern: {pattern}\n\thaystack: {string}");
    }
}
