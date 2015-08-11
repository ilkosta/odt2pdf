
extern crate crypto;
extern crate iron;
extern crate router;
extern crate params;
extern crate hyper;
extern crate logger;


use iron::prelude::*;
use router::{Router};
use iron::status;
use params::Params;
use logger::Logger;
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

use params::FromValue;

macro_rules! bad_request {
  // `()` indicates that the macro takes no argument
  ($msg:expr) => (
    Err(IronError::new($msg, status::BadRequest))
  )
}

use std::error::Error;
use std::fmt::{self, Debug};

#[derive(Debug)]
struct StringError(String);


impl fmt::Display for StringError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    Debug::fmt(self, f)
  }
}

impl Error for StringError {
  fn description(&self) -> &str { &*self.0 }
}

struct RequireMd5sumParam;

impl RequireMd5sumParam {
  fn value(params: &params::Map) -> String {
    let s = String::from_value(params.find(&["md5sum"]).unwrap()).unwrap();
    println!("'md5sum': {}", s);
    s
  }
}

impl BeforeMiddleware for RequireMd5sumParam {

  fn before(&self, req: &mut Request) -> IronResult<()> {
    let param_res = req.get_ref::<Params>();
    return match param_res {
      Err(why) => {
        bad_request!(why)
      },
      Ok(ref params) => {
        match params.find(&["md5sum"]) {
          None => bad_request!(StringError("the request doesn't contains the parameter 'md5sum'".to_string())),
          Some(passed_md5sum) => {
            // check the type
            match *passed_md5sum {
              params::Value::String(ref p) => Ok(()),
              _ => bad_request!(StringError("the 'md5sum' parameter is not of type 'string'".to_string()))
            }
          }
        }
      }
    }
  }
}

struct CheckFileParam;
impl BeforeMiddleware for CheckFileParam {

  fn before(&self, req: &mut Request) -> IronResult<()> {
    let param_res = req.get_ref::<Params>();
    return match param_res {
      Err(why) => {
        bad_request!(why)
      },
      Ok(ref params) => {
        match params.find(&["file"]) {
          None => bad_request!(StringError("the request doesn't contains the parameter 'file'".to_string())),
          Some(passed_md5sum) => {
            // check the type
            match *passed_md5sum {
              params::Value::File(ref p) => Ok(()),
              _ => bad_request!(StringError("the 'file' parameter is not of type 'file'".to_string()))
            }
          }
        }
      }
    }
  }
}


macro_rules! check_parameter {
  // `()` indicates that the macro takes no argument
  ($name:ident, $pname:expr, $ptype:pat, $rtype:ty) => (
    struct $name;


    impl BeforeMiddleware for $name {

      fn before(&self, req: &mut Request) -> IronResult<()> {
        let param_res = req.get_ref::<Params>();
        return match param_res {
          Err(why) => {
            bad_request!(why)
          },
          Ok(ref params) => {
            match params.find(&[$pname]) {
              None => bad_request!(StringError(format!("the request doesn't contains the parameter '{}'", $pname))),
              Some(passed_md5sum) => {
                // check the type
                match *passed_md5sum {
                  $ptype => Ok(()),
                  _ => bad_request!(StringError(format!("the '{}' parameter is not of the correct type", $pname)))
                }
              }
            }
          }
        }
      }
    }
  )
}
check_parameter!(RequireMd5sumParam2, "md5sum", params::Value::String(_), String);


fn submit_form_file2(req: &mut Request) -> IronResult<Response> {
  // parameters must exists...
  let param_res = req.get_ref::<Params>();
  let md5sum = RequireMd5sumParam::value(param_res.unwrap());
  Ok(Response::with(status::Ok))
}



// curl -i "http://localhost:3000/openact"  -F "filename=@/home/costa/test.csv" -F "name=jason" -F
// "age=2"
fn submit_form_file(req: &mut Request) -> IronResult<Response> {
//   use params::FromValue;
  
  let param_res = req.get_ref::<Params>(); // Result<&params::Map, params::ParamsError>
    println!("DEBUG - received request: {:?}", &param_res);
    
    match param_res {
      Err(why) => {
        let msg = format!("{:?}", why);
        println!("{}", msg); 
        Ok(Response::with((status::InternalServerError, msg)))
      },
      Ok(ref params) => match params.find(&["md5sum"]) {
        Some(ref passed_md5sum) => {
          
          match **passed_md5sum {
            params::Value::String(ref passed_md5sum) => {
              match params.find(&["filename"]) {
                None => {
                  let msg = format!("missing 'filename' field: {:?}", params);
                  println!("{}", msg);
                  Ok(Response::with((status::BadRequest, msg)))
                },
                Some(ref f) => { 
                  match **f {
                    params::Value::File(ref file_param) => { 
                      let file_path = file_param.path().display();
                      let msg = format!("the path is: {:?}", file_path);
                      println!("{}", msg);
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
//                           use std::process::Command;
                          
//                           match Command::new("file") 
//                             .arg("-b")
//                             .arg("--mime-type")
//                             .arg(file_path)
//                             .output() {
//                               Err(why) => 
//                             }
//                             .unwrap_or_else(|e| { Ok(Response::with((status::PreconditionRequired, 
//                               format!("error finding the type of the uploaded file {:?}", file_param.filename()) )) });
                          
                        }
                      }
                      
                      
                    },
                    _ => {
                      let msg = format!("The request parameter 'filename' doesn't correspond to a real file: {:?}", f);
                      println!("{}", msg);
                      Ok(Response::with((status::PreconditionRequired, msg)))
                    }
                  }
                }
                
              }
            }
            _ => {
              let msg = format!(
                "The request parameter 'md5sum':'{:?}'' in the request {:?} doesn't correspond to a string", 
                passed_md5sum, param_res);
              println!("{}", msg);
              Ok(Response::with((status::BadRequest, msg)))
            }
          }

        },
        _ => {
          let msg = format!("The request parameter 'md5sum' is required");
          println!("{}", msg);
          Ok(Response::with((status::BadRequest, msg)))
        }
      },
        
    }
  
}





fn start_server() {
   let (logger_before, logger_after) = Logger::new(None);
   let mut router = Router::new();
   let mut chain_form_file = Chain::new(submit_form_file2);
   chain_form_file.link_before(logger_before);
   chain_form_file.link_before(RequireMd5sumParam);
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
