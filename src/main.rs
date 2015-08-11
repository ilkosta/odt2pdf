
extern crate crypto;
extern crate iron;
extern crate router;
extern crate params;
extern crate hyper;


use iron::prelude::*;
use router::{Router};
use iron::status;
use iron::{BeforeMiddleware};


/// generate the md5sum of a given file (io::Read)
pub fn md5sum<T : std::io::Read> (f: &mut T) -> String  {
  use crypto::digest::Digest;

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


#[macro_use]
mod param_rules;


use param_rules::RequiredParam;




fn submit_form_file(req: &mut Request) -> IronResult<Response> {

  let passed_md5sum = &String::get_param_value(req, "md5sum").unwrap();
  let file_param = params::File::get_param_value(req, "filename").unwrap();
  let file_path = file_param.path().display();
  match file_param.open() {
    Err(why) => {
      let msg = format!("cannot open the uploaded file '{:?}' in path '{}' : '{}'", file_param.filename(), file_path, why);
      println!("{}", msg);
      Ok(Response::with((status::InternalServerError, msg)))
    },
    Ok(ref mut file) => {
      let calculated_md5sum = &md5sum(file);
      if calculated_md5sum != passed_md5sum {
        let msg = format!(
          "the md5sum '{}' calculate for the file {} doesn't correspond to the parameter '{}'",
          calculated_md5sum, file_path, passed_md5sum
        );
        return Ok(Response::with((status::PreconditionRequired, msg)))
      }

      return Ok(Response::with((status::Ok, "ciao mondo")))
    }
  }
}

require_param!(RequireMd5sumParam, "md5sum", String);
require_param!(RequireFileParam, "filename", params::File);

mod logger;

use logger::Logger;
use iron::{AroundMiddleware};
fn start_server() {
   let mut router = Router::new();
   let mut chain_form_file = Chain::new(submit_form_file);

   chain_form_file.link_before(RequireMd5sumParam);
   chain_form_file.link_before(RequireFileParam);

   router.post("/openact", Logger::default().around(Box::new(chain_form_file)));

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
