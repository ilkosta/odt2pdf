﻿
extern crate iron;
extern crate router;
extern crate params;
extern crate config;

use std::process::Command;
use std::path::Path;

use iron::prelude::*;
use router::{Router};
use iron::status;
use iron::{BeforeMiddleware};


mod md5sum;

use md5sum::md5sum;


use param_rules::RequiredParam;


macro_rules! status_from_io_err {
  ($why:expr) => (

    match $why.kind() {
      std::io::ErrorKind::NotFound => status::NotFound,
      std::io::ErrorKind::PermissionDenied => status::Forbidden,
      _ => status::InternalServerError,
    }

  )
}

use config::reader::from_file;
use config::reader::from_str;
// use config::error::ConfigErrorKind;
fn submit_form_file(req: &mut Request) -> IronResult<Response> {

  use std::fs::{rename,File};

  // thanks to BeforeMiddleware rules we can do unwrap safely
  let passed_md5sum = &String::get_param_value(req, "md5sum").unwrap();
  let file_param = params::File::get_param_value(req, "filename").unwrap();

  let file_path = format!("{}.ods" , file_param.path().display());




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


          let config_fname = "./conversion.conf";

          // read the configuration from the config_fname file,
          // if not found or not correct, load a default configuration string
          let conf = match config::reader::from_file(Path::new(config_fname)) {
            Ok(conf) => conf,
            Err(why) => {
              println!("error reading the configuration file '{}': {:?}", config_fname,why);

              // go to fallback

              let ref default_configuration_str = format!(
                "transformation_cmd=\"{}\";\n",

                "timeout --kill-after 10s 1m \
                 time -v -a -o {working_dir}/time.log \
                 libreoffice --headless --convert-to pdf:writer_pdf_Export --outdir {working_dir} {file}");

              match from_str(default_configuration_str) {
                Err(why) => panic!(why),
                Ok(conf) => conf
              }
            }
          };


          // run the cmd

          let cmd_name = conf.lookup_str("transformation_cmd").unwrap();
          let cmd_name = cmd_name.replace("{working_dir}", &format!("{}",file_param.path().parent().unwrap().display()));
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
            return Ok(Response::with(status::InternalServerError))


            // copy the file in an error dirctory for further investigations

          }


        }
      }
    }
  }

}

#[macro_use]
mod param_rules;

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
