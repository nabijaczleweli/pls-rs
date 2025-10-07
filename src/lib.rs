//! Parser and writer for the [`PLS` playlist format](https://en.wikipedia.org/wiki/PLS_(file_format)).
//!
//! # Examples
//!
//! Reading PLS':
//!
//! ```
//! # use pls::{PlaylistElement, ElementLength};
//! assert_eq!(pls::parse(&mut &b"[playlist]\n\
//!                               File1=Track 1.mp3\n\
//!                               Title1=Unknown Artist - Track 1\n\
//!                               \n\
//!                               File2=Track 2.mp3\n\
//!                               Length2=420\n\
//!                               \n\
//!                               File3=Track 3.mp3\n\
//!                               Length3=-1\n\
//!                               \n\
//!                               NumberOfEntries=3\n"[..]).unwrap(),
//!            vec![PlaylistElement {
//!                path: "Track 1.mp3".to_string(),
//!                title: Some("Unknown Artist - Track 1".to_string()),
//!                len: ElementLength::Unknown,
//!            },
//!            PlaylistElement {
//!                path: "Track 2.mp3".to_string(),
//!                title: None,
//!                len: ElementLength::Seconds(420),
//!            },
//!            PlaylistElement {
//!                path: "Track 3.mp3".to_string(),
//!                title: None,
//!                len: ElementLength::Unknown,
//!            }]);
//! ```
//!
//! Writing PLS':
//!
//! ```
//! # use pls::{PlaylistElement, ElementLength};
//! let mut buf = Vec::new();
//! pls::write(&[PlaylistElement {
//!                path: "Track 1.mp3".to_string(),
//!                title: Some("Unknown Artist - Track 1".to_string()),
//!                len: ElementLength::Unknown,
//!            },
//!            PlaylistElement {
//!                path: "Track 2.mp3".to_string(),
//!                title: None,
//!                len: ElementLength::Seconds(420),
//!            },
//!            PlaylistElement {
//!                path: "Track 3.mp3".to_string(),
//!                title: None,
//!                len: ElementLength::Unknown,
//!            }],
//!            &mut buf).unwrap();
//! assert_eq!(String::from_utf8(buf).unwrap(),
//!            "[playlist]\n\
//!             File1=Track 1.mp3\n\
//!             Title1=Unknown Artist - Track 1\n\
//!             \n\
//!             File2=Track 2.mp3\n\
//!             Length2=420\n\
//!             \n\
//!             File3=Track 3.mp3\n\
//!             \n\
//!             NumberOfEntries=3\n\
//!             Version=2\n")
//! ```

use ini::ini;
use std::error::Error as ErrorT;
use std::fmt;
use std::io::{self, Read, Write};
use std::num::ParseIntError;

/// A single element of a playlist
///
/// # Examples
///
/// ```
/// # use pls::{PlaylistElement, ElementLength};
/// # use std::io;
/// # struct File { d: &'static [u8] };
/// # impl File {
/// #     fn open(_: &str) -> File { File { d: &b"[playlist]\n\
/// #                                             File1=Track 1.mp3\n\
/// #                                             Title1=Unknown Artist - Track 1\n\
/// #                                             Length1=420\n\
/// #                                             \n\
/// #                                             NumberOfEntries=1\n\
/// #                                             Version=2\n"[..] } }
/// # }
/// # impl io::Read for File {
/// #     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.d.read(buf) }
/// # }
/// let elements = pls::parse(&mut File::open("Unknown Artist.pls")).unwrap();
/// # assert_eq!(elements,
/// #            vec![PlaylistElement {
/// #                path: "Track 1.mp3".to_string(),
/// #                title: Some("Unknown Artist - Track 1".to_string()),
/// #                len: ElementLength::Seconds(420),
/// #            }]);
/// ```
///
/// ```
/// # use pls::{PlaylistElement, ElementLength};
/// # use std::io;
/// # struct File { f: () };
/// # impl File {
/// #     fn create(_: &str) -> File { File { f: () } }
/// # }
/// # impl io::Write for File {
/// #     fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
/// #     fn flush(&mut self) -> io::Result<()> { Ok(()) }
/// # }
/// pls::write(&[PlaylistElement {
///                path: "Track 1.mp3".to_string(),
///                title: Some("Unknown Artist - Track 1".to_string()),
///                len: ElementLength::Seconds(420),
///            }],
///            &mut File::create("Unknown Artist.pls")).unwrap();
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlaylistElement {
    /// Path specified in the `File#` key, unconstrained
    pub path: String,
    /// Title specified by the `Title#` key or `None` if omitted
    pub title: Option<String>,
    /// Length specified by the `Length#` key or `Unknown` if omitted
    pub len: ElementLength,
}

/// Playlist element's length
///
/// `Unknown` if omitted or set to `-1`
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElementLength {
    /// Length was specified in `Length#` field
    Seconds(u64),
    /// Length was omitted or set to `-1`
    Unknown,
}

/// All ways parsing can fail
#[derive(Debug)]
pub enum ParseError {
    /// Specified version was not `2`
    InvalidVersion(u64),
    /// The whole `[playlist]` section's missing
    MissingPlaylistSection,
    /// Some required key is missing
    MissingKey(String),
    /// An integer was not actually an integer
    InvalidInteger(ParseIntError),
    /// Other `.ini` parsing errors
    Ini(ini::Error),
}

/// Parse a playlist
///
/// The parser is very lenient and allows pretty much everything as long as the
/// required stuff's in.
///
/// # Examples
///
/// ```
/// # use pls::{PlaylistElement, ElementLength};
/// assert_eq!(pls::parse(&mut &b"[playlist]\n\
///                               File1=Track 1.mp3\n\
///                               Title1=Unknown Artist - Track 1\n\
///                               \n\
///                               File2=Track 2.mp3\n\
///                               Length2=420\n\
///                               \n\
///                               File3=Track 3.mp3\n\
///                               Length3=-1\n\
///                               \n\
///                               NumberOfEntries=3\n"[..]).unwrap(),
///            vec![PlaylistElement {
///                path: "Track 1.mp3".to_string(),
///                title: Some("Unknown Artist - Track 1".to_string()),
///                len: ElementLength::Unknown,
///            },
///            PlaylistElement {
///                path: "Track 2.mp3".to_string(),
///                title: None,
///                len: ElementLength::Seconds(420),
///            },
///            PlaylistElement {
///                path: "Track 3.mp3".to_string(),
///                title: None,
///                len: ElementLength::Unknown,
///            }]);
/// ```
pub fn parse<R: Read>(what: &mut R) -> Result<Vec<PlaylistElement>, ParseError> {
    let p = ini::Ini::read_from(what)?;
    let play = p.section(Some("playlist")).ok_or(ParseError::MissingPlaylistSection)?;

    if let Some(v) = play.get("Version") {
        let v = v.parse()?;
        if v != 2 {
            return Err(ParseError::InvalidVersion(v));
        }
    }

    // Some major radio stations have malformed pls files, handle without error:
    // "numberofentries" http://newmedia.kcrw.com/legacy/pls/kcrwsimulcast.pls
    // "NumberOfEvents" http://www.abc.net.au/res/streaming/audio/mp3/classic_fm.pls
    if let Some(e) = play
        .get("NumberOfEntries")
        .or_else(|| play.get("numberofentries"))
        .or_else(|| play.get("NumberOfEvents"))
    {
        let e: u64 = e.parse()?;
        let mut elems = Vec::with_capacity(e as usize);
        for i in 1..e + 1 {
            elems.push(PlaylistElement {
                path: play
                    .get(&format!("File{}", i))
                    .ok_or_else(|| ParseError::MissingKey(format!("File{}", i)))?
                    .clone(),
                title: play.get(&format!("Title{}", i)).cloned(),
                len: ElementLength::parse(play.get(&format!("Length{}", i)))?,
            })
        }
        Ok(elems)
    } else {
        Err(ParseError::MissingKey("NumberOfEntries|numberofentries|NumberOfEvents".to_string()))
    }
}

/// Write a playlist to the specified output stream
///
/// # Examples
///
/// ```
/// # use pls::{PlaylistElement, ElementLength};
/// let mut buf = Vec::new();
/// pls::write(&[PlaylistElement {
///                path: "Track 1.mp3".to_string(),
///                title: Some("Unknown Artist - Track 1".to_string()),
///                len: ElementLength::Unknown,
///            },
///            PlaylistElement {
///                path: "Track 2.mp3".to_string(),
///                title: None,
///                len: ElementLength::Seconds(420),
///            },
///            PlaylistElement {
///                path: "Track 3.mp3".to_string(),
///                title: None,
///                len: ElementLength::Unknown,
///            }],
///            &mut buf).unwrap();
/// assert_eq!(String::from_utf8(buf).unwrap(),
///            "[playlist]\n\
///             File1=Track 1.mp3\n\
///             Title1=Unknown Artist - Track 1\n\
///             \n\
///             File2=Track 2.mp3\n\
///             Length2=420\n\
///             \n\
///             File3=Track 3.mp3\n\
///             \n\
///             NumberOfEntries=3\n\
///             Version=2\n")
/// ```
pub fn write<'i, I: IntoIterator<Item = &'i PlaylistElement>, W: Write>(what: I, to: &mut W) -> io::Result<()> {
    writeln!(to, "[playlist]")?;

    let mut ent = 0u64;
    for (i, PlaylistElement { path, title, len }) in what.into_iter().enumerate() {
        writeln!(to, "File{}={}", i + 1, path)?;

        if let Some(title) = title.as_ref() {
            writeln!(to, "Title{}={}", i + 1, title)?;
        }

        if let ElementLength::Seconds(s) = *len {
            writeln!(to, "Length{}={}", i + 1, s)?;
        }

        writeln!(to)?;
        ent += 1;
    }

    writeln!(to, "NumberOfEntries={}", ent)?;
    writeln!(to, "Version=2")?;

    Ok(())
}

impl ElementLength {
    fn parse<S: AsRef<str>>(what: Option<S>) -> Result<ElementLength, ParseError> {
        if let Some(what) = what {
            let what = what.as_ref();
            if what == "-1" {
                Ok(ElementLength::Unknown)
            } else {
                Ok(ElementLength::Seconds(what.parse()?))
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
    fn source(&self) -> Option<&(dyn ErrorT + 'static)> {
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
            (ParseError::InvalidVersion(v), &ParseError::InvalidVersion(rv)) => *v == rv,
            (ParseError::MissingPlaylistSection, &ParseError::MissingPlaylistSection) => true,
            (ParseError::MissingKey(k), ParseError::MissingKey(rk)) => k == rk,
            (ParseError::InvalidInteger(e), ParseError::InvalidInteger(re)) => e == re,
            (ParseError::Ini(e), ParseError::Ini(re)) => e.line == re.line && e.col == re.col && e.msg == re.msg,
            (_, _) => false,
        }
    }
}
