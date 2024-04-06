use serde::Deserialize;

use crate::config_manager::{self, CrabFetchColor};

#[derive(Deserialize)]
pub struct AsciiConfiguration {
    pub display: bool,
    pub colors: Vec<CrabFetchColor>,
    pub margin: u16,
}

// Return type is the ascii & the maximum length of it
pub fn get_ascii(os: &str) -> (String, u16) {
    // Will first confirm if theres a ascii override file
    let user_override: Option<String> = config_manager::check_for_ascii_override();
    if user_override != None {
        let mut length: u16 = 0;
        user_override.as_ref().unwrap().split("\n").for_each(|x| if x.len() > length as usize { length = x.len() as u16 });
        return (user_override.unwrap(), length)
    }

    match os {
        "arch" => arch(),
        "debian" => debian(),
        _ => ("".to_string(), 0)
    }
}

// Define art down below here
fn arch() -> (String, u16) { // Generated from https://www.text-image.com/convert/ascii.html
("             ~!
            ^YY^
           :JYYY^
          :JYYYYY^
         :JYYYYYYY^
        ^JYYYYYYYYY^
       ~YYYYYYYYYYYY~
      ~YYYYY?!!?YYYYY!
     !YYYYY!    ~YYYYY7.
   .7YYYYYJ      ?YYYYY?.
  .?YYYYYJ7      7JJYYYYJ:
 ^JY?7~^..        ..^~7?YY^
^7~:                    :~7^"
 .to_string(), 28)
}
fn debian() -> (String, u16) { // Generated from https://www.text-image.com/convert/ascii.html
("         .^!7!~~~^:
      :!JPPP55YY5555Y?!:
    ^YPG5?!:.    .:~?PGPJ:
   7G57^.            .7PG5~
 .YP7.       :^^^:.    !PY^
 JG7       .!~:....     YP:
.PY       .?.       .   YP^
.P?       :Y       .   :P7
.P?       .??.   .   .~Y!
 JP.      ..~?7~^:^~77!.
 ^PY~        .^^~~^:.
  ~PP^
   :YP^
     !Y?^
       ~??~^.
         .^~~:..            "
 .to_string(), 28)
}
