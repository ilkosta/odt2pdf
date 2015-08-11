
extern crate iron;
extern crate router;
extern crate params;
extern crate hyper;


use std::process::Command;
use std::path::Path;

use iron::prelude::*;
use router::{Router};
use iron::status;
use iron::{BeforeMiddleware};


mod md5sum;

use md5sum::md5sum;


#[macro_use]
mod param_rules;


use param_rules::RequiredParam;


fn submit_form_file(req: &mut Request) -> IronResult<Response> {

  use std::fs::{rename,File};

  let passed_md5sum = &String::get_param_value(req, "md5sum").unwrap();
  let file_param = params::File::get_param_value(req, "filename").unwrap();

  let file_path = format!("{}.ods" , file_param.path().display());

  macro_rules! status_from_io_err {
    ($why:expr) => (

      match $why.kind() {
        std::io::ErrorKind::NotFound => status::NotFound,
        std::io::ErrorKind::PermissionDenied => status::Forbidden,
        _ => status::InternalServerError,
      }

    )
  }


  match rename(&file_param.path(), &file_path) {
    Err(why) => {
      let status = status_from_io_err!(why);
      Ok(Response::with((status, format!("failed to elaborate the received file: {}", why))))
    },
    Ok(_) => {
      println!("elaborating the file {}", file_path);

      match File::open(&file_path) {
        Err(why) => {
          let status = status_from_io_err!(why);
          Ok(Response::with((status, format!("failed to opening the received file: {}", why))))
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

          // it seems good
          let res = Command::new("timeout")
          .arg("--kill-after")
          .arg("10s")
          .arg("1m")

          // ----------------------- time it
          .arg("time")
          .arg("-v")
          .arg("-a")
          .arg("-o")
          .arg(format!("{}",file_param.path().parent().unwrap().join("time.log").display()))

          // ----------------------- convert
          .arg("libreoffice")
          .arg("--headless")
          .arg("--convert-to")
          .arg("pdf:writer_pdf_Export")
          .arg("--outdir")
          .arg(format!("{}",file_param.path().parent().unwrap().display()))
          .arg(file_path)
          .status().unwrap_or_else(|e| {
            println!("failed to execute process: {}", e);
            panic!("failed to execute process: {}", e);
          });

          println!("process exited with: {}", res);

          let ref file_path = format!("{}.pdf" , file_param.path().display());
          println!("file_path: {}", file_path);
          Ok(Response::with((status::Ok, Path::new(file_path))))
        }
      }
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


fn main() {
    println!("Hello, world!");
    start_server();
}
