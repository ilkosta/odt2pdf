extern crate params;

use std::fmt::{self};
use std::error::Error;
use std::io::ErrorKind;
use std::fs::{File};
use std::process::Command;

use iron::prelude::*;
use iron::status;

use param_rules::{get_param};

use hash::sha1sum;

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
 
pub fn same_sha1sum_of_param_req(req: &mut Request, file_param : &self::params::File) -> IronResult<()> {

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



pub fn odt_extension(_: &mut Request, file_param : &self::params::File) -> IronResult<()> {

  let fname = String::from(file_param.filename().expect("filename parameter not present! it must be have checked by BeforeMiddleware"));
//   let fname = fname.toUpper();
  if !fname.ends_with(".odt") {
    Err(IronError::new(UploadError(String::from("Wrong file extension.")), status::PreconditionRequired))
  }
  else {
    Ok(())
  }
}



pub fn correct_magic_number(_: &mut Request, file_param : &self::params::File) -> IronResult<()> {

  let ref file_path = format!("{}",file_param.path().display());
  let cmd_name  = "file -b --mime-type {file}";
  let cmd_name  = cmd_name.replace("{file}", &file_path);
  debug!("executing: {}", cmd_name);  
  let mut cmd_args: Vec<&str> = cmd_name.split_whitespace()
                                        .collect();
                                        
  let output = Command::new(cmd_args.remove(0))
                        .args(&cmd_args)
                        .output()
                        .unwrap_or_else(|e| {
                          panic!("failed to execute file process: {}", e);
                        });

  if !output.status.success() {
    Err(IronError::new(UploadError(String::from("Error determining file type.")), status::InternalServerError))
  } 
  else {
    let file_type = String::from_utf8_lossy(&output.stdout);
    if  file_type != "application/vnd.oasis.opendocument.text\n" && 
        file_type != "application/zip\n"
    {
        Err(IronError::new(
            UploadError(format!("wrong type: {}", file_type)), status::PreconditionRequired))
    }
    else {
//         Err(IronError::new(UploadError(String::from("ok, ma stoppoooooooooo.")), status::PreconditionRequired))
        Ok(())
    }
  }
  
}
