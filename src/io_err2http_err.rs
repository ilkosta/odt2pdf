// use std::error::Error;
// use std::io::ErrorKind;
// use iron::status;

macro_rules! status_from_io_err {
  ($why:expr) => (

    match $why.kind() {
      ErrorKind::NotFound => status::NotFound,
      ErrorKind::PermissionDenied => status::Forbidden,
      _ => status::InternalServerError,
    }

  )
}