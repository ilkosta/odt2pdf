#![feature(plugin)]

#![plugin(clippy)]

#[macro_use]
extern crate log;
extern crate fern;

extern crate iron;
extern crate router;
extern crate params;
extern crate staticfile;
extern crate mount;

use std::process::Command;
use std::path::Path;
use std::error::Error;

use std::fmt::{self};

use iron::prelude::*;
use router::{Router};
use iron::status;
use iron::{BeforeMiddleware};
use std::fs::{rename,File};

mod hash;

#[macro_use]
mod config;

use hash::sha1sum;


use param_rules::{RequiredParam, get_param};

macro_rules! status_from_io_err {
  ($why:expr) => (

    match $why.kind() {
      std::io::ErrorKind::NotFound => status::NotFound,
      std::io::ErrorKind::PermissionDenied => status::Forbidden,
      _ => status::InternalServerError,
    }

  )
}

#[macro_use]
mod param_rules;


#[derive(Debug)]
struct UploadError(String);

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt("upload error", f)
    }
}

impl Error for UploadError {
    fn description(&self) -> &str {
        "upload error"
    }
}


fn compare_sha1sum_of_file(req: &mut Request, file_param : params::File) -> IronResult<()> {

  let file_path = file_param.path();
  let passed_sha1sum = &get_param::<String>(req,"sha1sum"); // order dependency from RequireMd5sumParam
  
  match File::open( &file_path ) {
  
    Err(why) => {
      let status = status_from_io_err!(why);
      let msg = format!("failed to opening the received file: {}", why);
      Err(IronError::new(UploadError(msg), status))
    },
    Ok(ref mut file) => {
      let calculated_sha1sum = &sha1sum(file);
      if calculated_sha1sum != passed_sha1sum {
        let msg = format!(
          "the sha1sum '{}' calculate for the file {} doesn't correspond to the parameter '{}'",
          calculated_sha1sum, file_path.display(), passed_sha1sum
        );
        Err(IronError::new(UploadError(msg), status::PreconditionRequired))
      }
      else {
        Ok(())
      }
    }
    
  }
}

require_param!(RequireMd5sumParam, "sha1sum", String);
require_param!(RequireFileParam, "filename", params::File, compare_sha1sum_of_file);


#[allow(needless_return)]
fn submit_form_file(req: &mut Request) -> IronResult<Response> {

  // thanks to BeforeMiddleware rules we can do unwrap safely
  let file_param = get_param::<params::File>(req, "filename");

  // add .odt extension to tempfile
  let file_path = format!("{}.odt" , file_param.path().display());

  match rename(&file_param.path(), &file_path) {
    Err(why) => {
      let status = status_from_io_err!(why);
      Ok(Response::with((status, format!("failed to elaborate the received file: {}", why))))
    },
    Ok(_) => {
      println!("elaborating the file {}", file_path);
      
      let conf = config::get_config();

      // run the cmd
      let working_dir = format!("{}",file_param.path().parent().expect("cannot access the directory of the uploaded temporary file").display());
      let cmd_name = get_config_parameter!(conf,"transformation.cmd");
      let cmd_name = cmd_name.replace("{working_dir}", &working_dir);
      let cmd_name = cmd_name.replace("{file}", &file_path);

      println!("cmd: {}", cmd_name);

      let mut cmd_args: Vec<&str> = cmd_name.split_whitespace()
                                        .collect();


      let res = Command::new(cmd_args.remove(0))
                        .args(&cmd_args)
                        .status()
                        .unwrap_or_else(|e| {
                          println!("failed to execute process: {}", e);
                          panic!("failed to execute process: {}", e);
                        });

      println!("process exited with: {}", res);

      if res.success() {
        let ref file_path = format!("{}.pdf" , file_param.path().display());
        println!("file_path: {}", file_path);
        return Ok(Response::with((status::Ok, Path::new(file_path))))
      }
      else {
        // copy the file in an error dirctory for further investigations
        let err_destination_dir = get_config_parameter!(conf,"transformation.err_destination_dir");
        //             let res = Command::new("mkdir")
        //                               .args(&["-p",err_destination_dir])
        //                               .status()
        //                               .unwrap_or_else(|e| {
        //                                 println!("failed to execute process: {}", e);
        //                                 panic!("failed to execute process: {}", e);
        //                               });

        Command::new("nice")
        .args(&["-n","20","cp","-R", &working_dir, err_destination_dir])
        .spawn()
        .unwrap_or_else(|e| {
          println!("failed to execute process: {}", e);
          panic!("failed to execute process: {}", e);
        });

        return Ok(Response::with(status::InternalServerError))

      }
      
    }
  }

}



mod logger;

fn start_server() {
    logger::init();
    
   let mut router = Router::new();
   let mut chain_form_file = Chain::new(submit_form_file);

   chain_form_file.link_before(RequireMd5sumParam);
   chain_form_file.link_before(RequireFileParam);

   router.post("/odt2pdf", logger::get_log_enabled_handler(Box::new(chain_form_file)));
//    router.get("/openact/", staticfile::Static::new(Path::new("src/asset/html/")));
   
   let mut mount = mount::Mount::new();
   mount.mount("/", router)
        .mount("/openact/", staticfile::Static::new(Path::new("src/asset/html/")));


   println!("started server at http://localhost:3000/");
//    Iron::new(router).http("localhost:3000").unwrap();
   Iron::new(mount).http("localhost:3000").unwrap();
}


fn main() {
    println!("Hello, world!");
    start_server();
}
