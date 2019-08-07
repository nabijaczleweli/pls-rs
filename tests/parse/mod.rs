use pls::{PlaylistElement, ElementLength, parse};

mod incorrect;


#[test]
fn correct() {
    for number_of_entries in &["NumberOfEntries", "numberofentries", "NumberOfEvents"] {
        assert_eq!(parse(&mut format!("[playlist]\n\
                                       File1=S:/M J U Z I K/pobrany/A-F-R-O & NGHTMRE - Stronger.mp3\n\
                                       \n\
                                       File2=S:/M J U Z I K/Z plyt/A-F-R-O - Tales From The Basement/01 Activated Trap Locks.mp3\n\
                                       Length2=79\n\
                                       \n\
                                       File3=S:/M J U Z I K/Z plyt/A-F-R-O - Tales From The Basement/02 Animal Kingdom.mp3\n\
                                       Title3=A-F-R-O - Animal Kingdom\n\
                                       Length3=124\n\
                                       \n\
                                       File4=http://127.0.0.1:8002/%D0%BC%D1%83%D0%B7%D1%8B%D0%BA%D0%B0/Z%20p%C5%82yt/\
                                             A-F-R-O%20-%20Tales%20From%20The%20Basement/03%20%23CODE%20829.mp3\n\
                                       Title4=A-F-R-O - CODE 829\n\
                                       Length4=-1\n\
                                       \n\
                                       {}=4\n\
                                       Version=2\n",
                                      number_of_entries)
                       .as_bytes()),
                   Ok(vec![PlaylistElement {
                               path: "S:/M J U Z I K/pobrany/A-F-R-O & NGHTMRE - Stronger.mp3".to_string(),
                               title: None,
                               len: ElementLength::Unknown,
                           },
                           PlaylistElement {
                               path: "S:/M J U Z I K/Z plyt/A-F-R-O - Tales From The Basement/01 Activated Trap Locks.mp3".to_string(),
                               title: None,
                               len: ElementLength::Seconds(79),
                           },
                           PlaylistElement {
                               path: "S:/M J U Z I K/Z plyt/A-F-R-O - Tales From The Basement/02 Animal Kingdom.mp3".to_string(),
                               title: Some("A-F-R-O - Animal Kingdom".to_string()),
                               len: ElementLength::Seconds(124),
                           },
                           PlaylistElement {
                               path: "http://127.0.0.1:8002/%D0%BC%D1%83%D0%B7%D1%8B%D0%BA%D0%B0/Z%20p%C5%82yt/\
                                      A-F-R-O%20-%20Tales%20From%20The%20Basement/03%20%23CODE%20829.mp3"
                                   .to_string(),
                               title: Some("A-F-R-O - CODE 829".to_string()),
                               len: ElementLength::Unknown,
                           }]));
    }
}
