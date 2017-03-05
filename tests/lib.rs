extern crate pls;

use std::fs::File;


#[test]
fn tst() {
    let p = pls::parse(&mut File::open("playlist.pls").unwrap());
    println!("{:#?}", p);
    pls::write(p.unwrap().iter(), &mut File::create("p.pls").unwrap()).unwrap();
    assert!(false);
}
