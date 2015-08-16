extern crate time;
extern crate log;
extern crate fern;

use iron::prelude::*;
use iron::{Handler, AroundMiddleware};

struct LogEnabler;

struct HandlerWithLog<H: Handler> {
  handler: H
}

impl AroundMiddleware for LogEnabler {

    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(HandlerWithLog { handler: handler } ) as Box<Handler>
    }
}
 
impl <H: Handler> Handler for HandlerWithLog<H> {

    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        debug!("{:?}", req);
        let entry = self::time::precise_time_ns();        
        let res = self.handler.handle(req);
        trace!("{:?} - rt: {:?}", res, self::time::precise_time_ns() - entry);
        res
    }
}


pub fn init() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            format!("[{}][{}] {}", self::time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout(), fern::OutputConfig::file("output.log")],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}

pub fn get_log_enabled_handler(handler : Box<Handler>) -> Box<Handler> {
  
  let use_log = LogEnabler;
    
  use_log.around(handler)
}




