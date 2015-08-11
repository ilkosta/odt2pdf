extern crate crypto;

use std::io::Read;
use self::crypto::digest::Digest;

/// generate the md5sum of a given file (io::Read)
pub fn md5sum<T : Read> (f: &mut T) -> String  {


  let mut digest = crypto::md5::Md5::new();
  let mut data: Vec<u8> = Vec::new();

  match f.read_to_end(&mut data) {
    Err(why) => { println!("Error reading the passed file to calculate the md5sum: {}", why); String::new() }
    Ok(size) => {
      if size > 0 {
        digest.input(&data);
        digest.result_str()
      } else {
        String::new()
      }
    }
  }
}

#[test]
fn md5sum_of_file() {
    static FP : &'static str = "/home/costa/pkg/lav/regione/attiweb/attività/20150107/definizione procedura di riavvio/Procedure_Riavvio_omnia.odt";

    let mut odt = match File::open(FP) {
        Err(why) => panic!("Cannot open the test file: {}", why),
        Ok(file) => file,
    };

    assert_eq!(md5sum(&mut odt), "a381cbe043c38a94af0983ffa934f338");
}
