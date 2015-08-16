extern crate config;



use self::config::reader::from_str;
// use self::config::error::ConfigErrorKind;

fn default_config() -> self::config::types::Config {

  let ref default_configuration_str = format!(
    "transformation:\n{}\ncmd=\"{}\";\nerr_destination_dir=\"{}\";\n{};\n",

    "{",

      // transformation.cmd
      "timeout --kill-after 10s 1m \
      time -v -a -o {working_dir}/time.log \
      libreoffice --headless --convert-to pdf:writer_pdf_Export --outdir {working_dir} {file}",

      // transformation.err_destination_dir
      "./errors",

      "}"
  );

  match from_str(default_configuration_str) {
    Err(why) => panic!(why),
    Ok(conf) => conf
  }

}

// use std::path::Path;
// use self::config::reader::from_file;
pub fn get_config() -> self::config::types::Config {

  default_config()


//   let config_fname = "./conversion.conf";
//
//   // read the configuration from the config_fname file,
//   // if not found or not correct, load a default configuration string
//   match config::reader::from_file(Path::new(config_fname)) {
//     Ok(conf) => conf,
//     Err(why) => {
//       println!("error reading the configuration file '{}': {:?}", config_fname,why);
//
//       // go to fallback
//       default_config()
//     }
//   }

}


#[macro_export]
macro_rules! get_config_parameter {
  ($conf:expr,$param:expr) => ($conf.lookup_str($param).expect(&format!("failed to load the configuration parameter '{}'",$param)) )
}
