use pls::{ParseError, parse};
use ini::ini::Error as IniError;


#[test]
fn invalid_version() {
    assert_eq!(parse(&mut &b"[playlist]\n\
                             Version=-1\n"[..]),
               Err(ParseError::InvalidInteger(u64::from_str_radix("-1", 10).unwrap_err())));
    assert_eq!(parse(&mut &b"[playlist]\n\
                             Version=0\n"[..]),
               Err(ParseError::InvalidVersion(0)));
    assert_eq!(parse(&mut &b"[playlist]\n\
                             Version=1\n"[..]),
               Err(ParseError::InvalidVersion(1)));
    assert_eq!(parse(&mut &b"[playlist]\n\
                             Version=3\n"[..]),
               Err(ParseError::InvalidVersion(3)));
    assert_eq!(parse(&mut &b"[playlist]\n\
                             Version=999\n"[..]),
               Err(ParseError::InvalidVersion(999)));
}

#[test]
fn missing_playlist_section() {
    assert_eq!(parse(&mut &b"File1=S:/M J U Z I K/pobrany/A-F-R-O & NGHTMRE - Stronger.mp3\n\
                             NumberOfEntries=1\n"[..]),
               Err(ParseError::MissingPlaylistSection));
}

#[test]
fn missing_number_of_entries() {
    assert_eq!(parse(&mut &b"[playlist]\n\
                             File1=S:/M J U Z I K/pobrany/A-F-R-O & NGHTMRE - Stronger.mp3\n"[..]),
               Err(ParseError::MissingKey("NumberOfEntries".to_string())));
}

#[test]
fn missing_file_entry() {
    assert_eq!(parse(&mut &b"[playlist]\n\
                             File1=S:/M J U Z I K/pobrany/A-F-R-O & NGHTMRE - Stronger.mp3\n\
                             File2=S:/M J U Z I K/Z plyt/A-F-R-O - Tales From The Basement/01 Activated Trap Locks.mp3\n\
                             NumberOfEntries=3"
                               [..]),
               Err(ParseError::MissingKey("File3".to_string())));
}

#[test]
fn invalid_number_of_entries() {
    assert_eq!(parse(&mut &b"[playlist]\n\
                             NumberOfEntries=-1"[..]),
               Err(ParseError::InvalidInteger(u64::from_str_radix("-1", 10).unwrap_err())));
}

#[test]
fn invalid_length() {
    assert_eq!(parse(&mut &b"[playlist]\n\
                             File1=S:/M J U Z I K/pobrany/A-F-R-O & NGHTMRE - Stronger.mp3\n\
                             Length1=Abolish the Burgeoisie!\n\
                             NumberOfEntries=1"
                               [..]),
               Err(ParseError::InvalidInteger(u64::from_str_radix("Abolish the Burgeoisie!", 10).unwrap_err())));
}

#[test]
fn ini() {
    assert_eq!(parse(&mut &b"[playlist\n"[..]),
               Err(ParseError::Ini(IniError {
                   line: 1,
                   col: 0,
                   msg: r#"Expecting "[Some(']')]" but found EOF."#.to_string(),
               })));
}
