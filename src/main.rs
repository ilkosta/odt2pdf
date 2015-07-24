
extern crate crypto;


/// generate the md5sum of a given file
pub fn md5sum<T : std::io::Read> (f: &mut T) -> String  {
    use crypto::digest::Digest;

     let mut digest = crypto::md5::Md5::new();
     let mut data: Vec<u8> = Vec::new();

     f.read_to_end(&mut data);
     digest.input(&data);
     digest.result_str()
}



#[cfg(test)]
mod tests {

    use std::fs::File;
    use super::md5sum;

    #[test]
    fn md5sum_of_file() {
        static FP : &'static str = "/home/costa/pkg/lav/regione/attiweb/attività/20150107/definizione procedura di riavvio/Procedure_Riavvio_omnia.odt";

        let mut odt = match File::open(FP) {
            Err(why) => panic!("Cannot open the test file: {}", why),
            Ok(file) => file,
        };

        assert_eq!(md5sum(&mut odt), "a381cbe043c38a94af0983ffa934f338");
    }

}



fn main() {
    println!("Hello, world!");
}
