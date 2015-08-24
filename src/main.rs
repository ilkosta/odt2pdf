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

extern crate rustc_serialize;
extern crate docopt;


use std::fs::{rename,File};
use std::process::Command;
use std::path::Path;
use std::error::Error;
use std::io::ErrorKind;

use std::net::{SocketAddrV4};
use std::net::Ipv4Addr;
use std::str::FromStr;



use iron::prelude::*;
use router::{Router};
use iron::status;
use iron::{BeforeMiddleware};

use docopt::Docopt;

mod hash;

#[macro_use]
mod config;

#[macro_use]
mod io_err2http_err;


use param_rules::{RequiredParam, get_param};


#[macro_use]
mod param_rules;

mod upload_checks;


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
      let cmd_name = conf.transformation.cmd;
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
        let err_destination_dir = &conf.transformation.error_dir;
        
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

fn start_server(args : &Args) {
  logger::init();

  let mut router = Router::new();
  let mut chain_form_file = Chain::new(submit_form_file);

  required_param!(sha1sum, String);
  required_param!(filename, params::File; rules [
    upload_checks::odt_extension 
  , upload_checks::correct_magic_number  
  , upload_checks::same_sha1sum_of_param_req]);
  
  chain_form_file.link_before(sha1sum);
  chain_form_file.link_before(filename);

  router.post("/odt2pdf", logger::get_log_enabled_handler(Box::new(chain_form_file)));
  //    router.get("/openact/", staticfile::Static::new(Path::new("src/asset/html/")));

  let mut mount = mount::Mount::new();
  mount.mount("/", router)
      .mount("/openact/", staticfile::Static::new(Path::new("src/asset/html/")));



  println!("started server at http://{}:{}/", args.flag_host, args.flag_port);
  Iron::new(mount).http( SocketAddrV4::new(Ipv4Addr::from_str(&args.flag_host).unwrap(), args.flag_port) ).unwrap();
}



const USAGE: &'static str = "
Naval Fate.

Usage:
  odt2pdf [--host=<host> --port=<port>]

Options:
  -h --help       Show this screen.
  --version       Show version.
  --port=<kn>     Port of the service [default: 3333].
  --host=<host>   Host of the service [default: 0.0.0.0].
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_port: u16,
    flag_host: String
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
                            
    start_server(&args);
}
