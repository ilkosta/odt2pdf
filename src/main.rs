
extern crate crypto;
extern crate iron;
extern crate router;
extern crate params;
extern crate hyper;
extern crate logger;


use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use iron::middleware::Handler;
use router::{Router};
use iron::status;
use params::Params;
use logger::Logger;
use std::error::Error;



/// generate the md5sum of a given file (io::Read)
pub fn md5sum<T : std::io::Read> (f: &mut T) -> String  {
  use crypto::digest::Digest;

  let mut digest = crypto::md5::Md5::new();
  let mut data: Vec<u8> = Vec::new();

  f.read_to_end(&mut data);
  digest.input(&data);
  digest.result_str()
}

// fn getFile(params: &params::Map) -> Option<&std::io::Read> 
// {
//   use std::fs::File;
//   use std::path::Path;
//   
// //   #[cfg(debug)]
// //   {
// //     for (key, value) in params.iter() {
// //       println!("{}: {:?}", key, value);
// //     }
// //   }
//   
//   
//   match params.find(&["filename"]) {
//     Some(ref f) => println!("{:?}", f),
//     
//                       { // Open the path in read-only mode, returns `io::Result<File>`
//       let path = Path::new(path);
//       match File::open(path) {
//           // The `description` method of `io::Error` returns a string that
//           // describes the error
//           Err(why) => panic!("couldn't open {}: {}", path.display(),
//                                                     Error::description(&why)),
//           Ok(file) => &file,
//       }
//     },
//     Nome => //None
//             println!("peccato"),
//   }
// }

// curl -i "http://localhost:3000/openact"  -F "filename=@/home/costa/test.csv" -F "name=jason" -F
// "age=2"
fn submit_form_file(req: &mut Request) -> IronResult<Response> {
  let param_res = req.get_ref::<Params>(); // Result<&params::Map, params::ParamsError>
    println!("{:?}", &param_res);
    
    let md5sum_check = match param_res {
        Ok(ref params) => match params.find(&["md5sum"]) {
          Some(ref passed_md5sum) => {
            match params.find(&["filename"]) {
              None => {
                println!("missing 'filename' field: {:?}", params);
                false
              },
              Some(f) => { 
                match *f {
                  params::Value::File(ref file) => println!("the path is: {:?}", file.path() ),
                  _ => println!("non si tratta di un file!!!!")
                }
                true
              }
//               match *f as params::Value {
//                 params::Value::File => {
//                   let file = f as &params::Value::File;
//                   md5sum(file.open()) == passed_md5sum
//                 }
//               }
            }
          },
            _ => false
        },
        Err(why) => {println!("{:?}", why); false},
    };
    Ok(Response::with(status::Ok))
}





fn start_server() {
   let (logger_before, logger_after) = Logger::new(None);
   let mut router = Router::new();
   let mut chain_form_file = Chain::new(submit_form_file);
   chain_form_file.link_before(logger_before);
   chain_form_file.link_after(logger_after);
   router.post("/openact", chain_form_file);
   println!("started server at http://localhost:3000/");
   Iron::new(router).http("localhost:3000").unwrap();
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

    #[test]
    fn simple_get() {
        use hyper::Client;
        use hyper::header::Connection;
        use std::io::Read;
        use super::start_server;

        // start_server();

        // Create a client.
        let client = Client::new();
        //
        // Creating an outgoing request.
        let mut res = client.get("http://localhost:3000/openact")
            // set a header
            .header(Connection::close())
            // let 'er go!
            .send().unwrap();

        // Read the Response.
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        println!("Response: {}", body);
        assert_eq!(body, "Hello World");
    }

}



fn main() {
    println!("Hello, world!");
    start_server();
}
