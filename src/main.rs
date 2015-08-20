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

use iron::prelude::*;
use router::{Router};
use iron::status;
use iron::{BeforeMiddleware};


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


#[allow(needless_return)]
fn submit_form_file(req: &mut Request) -> IronResult<Response> {

  use std::fs::{rename,File};

  // thanks to BeforeMiddleware rules we can do unwrap safely
  let passed_sha1sum = &get_param::<String>(req,"sha1sum");
  let file_param = get_param::<params::File>(req, "filename");

  let file_path = format!("{}.odt" , file_param.path().display());




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
          let calculated_sha1sum = &sha1sum(file);
          if calculated_sha1sum != passed_sha1sum {
            let msg = format!(
              "the sha1sum '{}' calculate for the file {} doesn't correspond to the parameter '{}'",
              calculated_sha1sum, file_path, passed_sha1sum
            );
            return Ok(Response::with((status::PreconditionRequired, msg)))
          }


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
  }

}

#[macro_use]
mod param_rules;

require_param!(RequireMd5sumParam, "sha1sum", String);
require_param!(RequireFileParam, "filename", params::File);

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


   println!("started server at http://10.6.9.145:8881/");
//    Iron::new(router).http("localhost:3000").unwrap();
   Iron::new(mount).http("10.6.9.145:8881").unwrap();
}


fn main() {
    println!("Hello, world!");
    start_server();
}
