extern crate ini as _ini;

use std::io::{self, Write, Read};
use std::error::Error as ErrorT;
use std::num::ParseIntError;
use _ini::ini;
use std::fmt;


#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlaylistElement {
    path: String,
    title: Option<String>,
    len: ElementLength,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElementLength {
    Seconds(u64),
    Unknown,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidVersion(u64),
    MissingPlaylistSection,
    MissingKey(String),
    InvalidInteger(ParseIntError),
    Ini(ini::Error),
}


pub fn parse<R: Read>(what: &mut R) -> Result<Vec<PlaylistElement>, ParseError> {
    let p = try!(ini::Ini::read_from(what));
    let play = try!(p.section(Some("playlist")).ok_or(ParseError::MissingPlaylistSection));

    if let Some(v) = play.get("Version") {
        let v = try!(v.parse());
        if v != 2 {
            return Err(ParseError::InvalidVersion(v));
        }
    }

    if let Some(e) = play.get("NumberOfEntries") {
        let e: u64 = try!(e.parse());
        let mut elems = Vec::with_capacity(e as usize);
        for i in 1..e + 1 {
            elems.push(PlaylistElement {
                path: try!(play.get(&format!("File{}", i)).ok_or_else(|| ParseError::MissingKey(format!("File{}", i)))).clone(),
                title: play.get(&format!("Title{}", i)).map(Clone::clone),
                len: try!(ElementLength::parse(play.get(&format!("Length{}", i)))),
            })
        }
        Ok(elems)
    } else {
        Err(ParseError::MissingKey("NumberOfEntries".to_string()))
    }
}

pub fn write<'i, I: Iterator<Item = &'i PlaylistElement>, W: Write>(what: I, to: &mut W) -> io::Result<()> {
    try!(writeln!(to, "[playlist]"));

    let mut ent = 0u64;
    for (i, &PlaylistElement { ref path, ref title, ref len }) in what.into_iter().enumerate() {
        try!(writeln!(to, "File{}={}", i + 1, path));

        if let Some(title) = title.as_ref() {
            try!(writeln!(to, "Title{}={}", i + 1, title));
        }

        if let ElementLength::Seconds(s) = *len {
            try!(writeln!(to, "Length{}={}", i + 1, s));
        }

        try!(writeln!(to, ""));
        ent += 1;
    }

    try!(writeln!(to, "NumberOfEntries={}", ent));
    try!(writeln!(to, "Version=2"));

    Ok(())
}


impl ElementLength {
    fn parse<S: AsRef<str>>(what: Option<S>) -> Result<ElementLength, ParseError> {
        if let Some(what) = what {
            let what = what.as_ref();
            if what == "-1" {
                Ok(ElementLength::Unknown)
            } else {
                Ok(ElementLength::Seconds(try!(what.parse())))
            }
        } else {
            Ok(ElementLength::Unknown)
        }
    }
}


impl From<ini::Error> for ParseError {
    fn from(e: ini::Error) -> ParseError {
        ParseError::Ini(e)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> ParseError {
        ParseError::InvalidInteger(e)
    }
}

impl ErrorT for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidVersion(_) => "invalid version specified",
            ParseError::MissingPlaylistSection => "[playlist] section missing",
            ParseError::MissingKey(_) => "required key missing",
            ParseError::InvalidInteger(ref e) => e.description(),
            ParseError::Ini(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&ErrorT> {
        match *self {
            ParseError::InvalidInteger(ref e) => Some(e),
            ParseError::Ini(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidVersion(v) => write!(f, "Invalid version {} specified", v),
            ParseError::MissingPlaylistSection => write!(f, "Missing [playlist] section"),
            ParseError::MissingKey(ref k) => write!(f, "Key \"{}\" missing", k),
            ParseError::InvalidInteger(ref e) => e.fmt(f),
            ParseError::Ini(ref e) => e.fmt(f),
        }
    }
}

impl Clone for ParseError {
    fn clone(&self) -> ParseError {
        match *self {
            ParseError::InvalidVersion(v) => ParseError::InvalidVersion(v),
            ParseError::MissingPlaylistSection => ParseError::MissingPlaylistSection,
            ParseError::MissingKey(ref k) => ParseError::MissingKey(k.clone()),
            ParseError::InvalidInteger(ref e) => ParseError::InvalidInteger(e.clone()),
            ParseError::Ini(ref e) => ParseError::Ini(ini::Error { msg: e.msg.clone(), ..*e }),
        }
    }
}

impl PartialEq for ParseError {
    fn eq(&self, rhs: &ParseError) -> bool {
        match (self, rhs) {
            (&ParseError::InvalidVersion(v), &ParseError::InvalidVersion(rv)) => v == rv,
            (&ParseError::MissingPlaylistSection, &ParseError::MissingPlaylistSection) => true,
            (&ParseError::MissingKey(ref k), &ParseError::MissingKey(ref rk)) => k == rk,
            (&ParseError::InvalidInteger(ref e), &ParseError::InvalidInteger(ref re)) => e == re,
            (&ParseError::Ini(ref e), &ParseError::Ini(ref re)) => e.line == re.line && e.col == re.col && e.msg == re.msg,
            (_, _) => false,
        }
    }
}
