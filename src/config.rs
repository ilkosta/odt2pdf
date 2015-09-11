extern crate rustc_serialize;

use std::fs::File;
use std::io::Read;
use rustc_serialize::json::{self};

#[allow(dead_code)]
#[derive(RustcDecodable)]
pub struct Configuration {
  pub transformation: TransformationConf,
  pub file: FileConf,
  pub clean: CleanConf,
  pub logger: LoggerConf
}

#[allow(dead_code)]
#[derive(RustcDecodable)]
pub struct TransformationConf {
  pub cmd : String,
  pub error_dir: String
}

#[allow(dead_code)]
#[derive(RustcDecodable)]
pub struct FileConf {
  pub cmd: String,
  pub accepted_types: Vec<String>
}



#[derive(RustcDecodable)]
pub struct CleanConf {
  pub converted: CleanConvertedConf
}

#[derive(RustcDecodable)]
pub struct CleanConvertedConf {
  pub cmd: String
}

#[derive(RustcDecodable)]
pub struct LoggerConf {
  pub output: String,
  pub level: String
}




pub fn get_config() -> Configuration {
  let config_fname = "./conf.json";
  
  let default_conf = r#" {
    "transformation" : {
      "cmd": "timeout --kill-after 10s 1m time -v -a -o {working_dir}/time.log libreoffice --headless --convert-to pdf:writer_pdf_Export --outdir {working_dir} {file}",
      "error_dir": "./errors/"
    },
    "file" : {
      "cmd" : "file -b --mime-type {file}",
      "accepted_types": ["application/vnd.oasis.opendocument.text\n", "application/zip\n"]
    },
    
    "clean" : {
    "converted" : {
      "cmd" : "nice -n 20 rm -r {working_dir}"
    },
  
    "logger" : {
      "output" : "output.log",
      "level" : "info"
    }
    
  }"#.to_owned();
  
  let notify_error = |why| {
    trace!("Erorr opening the configuration file {}: {}", config_fname , why);
  };
  
  let conf_str = match File::open(config_fname) {
    
    Err(why) => {
      notify_error(why);
      default_conf
    },
    Ok(ref mut f) => {
      let mut s = String::new();
      match f.read_to_string(&mut s) {
        Err(why) => {
          notify_error(why);
          default_conf
        },
        Ok(size) => if size > 0 {s} else {default_conf}
      }
    }
  };
  
  json::decode(&conf_str).unwrap()
}

