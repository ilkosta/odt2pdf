extern crate params;

use iron::prelude::*;
use iron::status;

use params::Params;
use params::FromValue;



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


pub trait RequiredParam<T : params::FromValue = Self> {

  fn get_value(value: &params::Value) -> Option<T> {
    T::from_value(value)
  }

  fn check(req: &mut Request, name: &str) -> IronResult<()> {

    macro_rules! bad_request {
      // `()` indicates that the macro takes no argument
      ($msg:expr) => (
        Err(IronError::new($msg, status::BadRequest))
      )
    }

    let param_res = req.get_ref::<Params>();
    return match param_res {
      Err(why) => {
        bad_request!(why)
      },
      Ok(ref params) => {
        match params.find(&[name]) {
          None => bad_request!(StringError(format!("the request doesn't contains the parameter {}", name).to_string())),
          Some(p) => if let Some(_) = Self::get_value(p) {
            Ok(())
          } else {
            bad_request!(StringError(format!("the '{}' parameter is not of the correct type",name).to_string()))
          }

        }
      }
    }
  }

  fn get_param_value(req: &mut Request, name: &str) -> Option<T> {
    match req.get_ref::<Params>() {
      Err(_) => None,
      Ok(ref params) => {
        match params.find(&[name]) {
          None => None,
          Some(p) => Self::get_value(p)
        }
      }
    }
  }
}



macro_rules! impl_required {
  ($t:ty) => (
    impl RequiredParam for $t {}
  )
}

impl_required!(String);
impl_required!(params::File);
impl_required!(bool);

impl<T: FromValue>  RequiredParam for Vec<T> {}
use std::collections::BTreeMap;
impl<T: FromValue>  RequiredParam for BTreeMap<String, T> {}
//impl_required!(Vec<T>);
//impl_required!(BTreeMap<String, T>);

impl_required!(u8);
impl_required!(u16);
impl_required!(u32);
impl_required!(u64);
impl_required!(usize);
impl_required!(i8);
impl_required!(i16);
impl_required!(i32);
impl_required!(i64);
impl_required!(isize);
impl_required!(f32);
impl_required!(f64);


#[macro_export]
macro_rules! require_param {
  ($name:ident, $param_name:expr, $ty:ty) => {
    struct $name;

    impl BeforeMiddleware for $name {
      fn before(&self, req: &mut Request) -> IronResult<()> {
        <$ty>::check(req, $param_name)
      }
    }
  }
}
