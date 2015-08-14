extern crate crypto;

use std::io::Read;
use self::crypto::digest::Digest;


macro_rules! hash_file {
  ($fname:ident, $hash_struct:ident) => {
    #[allow(dead_code)]
    pub fn $fname< F: Read> (f: &mut F) -> String {
      let mut digest = $hash_struct::new();
      let mut data: Vec<u8> = Vec::new();

      match f.read_to_end(&mut data) {
        Err(why) => { println!("Error reading the passed file to calculate the hash: {}", why); String::new() }
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
  }
}


use self::crypto::md5::Md5;
hash_file!(md5sum,  Md5);


use self::crypto::sha1::Sha1;
hash_file!(sha1,    Sha1);


#[test]
fn md5sum_of_file() {
  use std::fs::File;
  static FP : &'static str = "/home/costa/pkg/lav/regione/attiweb/attività/20150107/definizione procedura di riavvio/Procedure_Riavvio_omnia.odt";

  let mut odt = match File::open(FP) {
      Err(why) => panic!("Cannot open the test file: {}", why),
      Ok(file) => file,
  };

  assert_eq!(md5sum(&mut odt), "a381cbe043c38a94af0983ffa934f338");
}
