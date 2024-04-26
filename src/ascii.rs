use serde::Deserialize;

use crate::config_manager::{self, CrabFetchColor};

#[derive(Deserialize)]
pub struct AsciiConfiguration {
    pub display: bool,
    pub colors: Vec<CrabFetchColor>,
    pub margin: u16,
}
impl Default for AsciiConfiguration {
    fn default() -> Self {
        AsciiConfiguration {
            display: true,
            colors: vec![CrabFetchColor::BrightMagenta],
            margin: 4
        }
    }
}

// Return type is the ascii & the maximum length of it
pub fn get_ascii(os: &str) -> (String, u16) {
    // Will first confirm if theres a ascii override file
    let user_override: Option<String> = config_manager::check_for_ascii_override();
    if user_override != None {
        let mut length: u16 = 0;
        user_override.as_ref().unwrap().split("\n").for_each(|x| {
            // x.len() hated ascii art using weird characters, this works fine tho
            let len: usize = x.chars().collect::<Vec<char>>().len();
            if len > length as usize { length = len as u16 }
        });
        return (user_override.unwrap(), length)
    }

    match os {
        "arch" => arch(),
        "debian" => debian(),
        "ubuntu" => ubuntu(),
        "fedora" => fedora(),
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
fn ubuntu() -> (String, u16) { // https://www.text-image.com/convert/ascii.html
("           .^~7?JJYYYYJJ?7~^.
        :!?JYYYYYYYYYYYYY55YY?!:
     .~?YYYYYYYYYYYYYYYYY?7?JYYY?~.
    ~JYYYYYYYYYYYYYJJYYY^   .?YYYYJ~
  .?YYYYYYYYYYY!::.....7!.  :?YYYYYY?.
 .JYYYYYYYY?^.7?. .::.  ^~~~?YYYYYYYYJ.
 ?YYYYYYYY~   .JYJYYYYJ?~.   ~YYYYYYYY?
~YYYYYYYY^   ~YYYYYYYYYYYY~   ^YYYYYYYY~
JYYY7^:^7~  ~5YYYYYYYYYYYY5~   7YYYYYYYJ
JYYJ     J: ?YYYYYYYYYYYYYYJ!77?YYYYYYYJ
JYYY7^:^7~  ~5YYYYYYYYYYYY5~   7YYYYYYYJ
~YYYYYYYY^   ~YYYYYYYYYYYY~   ^YYYYYYYY~
 ?YYYYYYYY~   .JYJYYYYJ?~.   ~YYYYYYYY?
 .JYYYYYYYY?^.7?. .::.  ^~~~?YYYYYYYYJ.
  .?YYYYYYYYYYY!::.....7!.  :?YYYYYY?.
    ~JYYYYYYYYYYYYYJJYYY^   .?YYYYJ~
     .~?YYYYYYYYYYYYYYYYY?7?JYYY?~.
        :!?JYYYYYYYYYYYYY55YY?!:
           .^~7?JJYYYYJJ?7~^.           ".to_string(), 40) // fatty
}
fn fedora() -> (String, u16) { // https://www.text-image.com/convert/ascii.html
("      .^7J5PGGGGGPY7~.
    ^JPGGGGGGGP5J??J5PJ^
  ^YGBGGGGGGP!.   . ^5PGY^
 !GGGGGGGGGP:  !5PPPP5Y5GG!
~GGGGGGGGGB?  ~BGGGGGPYY5GG~
5GGGGGGGGGB?  ~BGGGGP5Y5PGG5
GGGGPP57:::.  .:::!5Y55PGGGG
GGP5Y55J~~~:  :~~~?PPGGGGGG5
GP5Y5PGGBGB?  ~BGGGGGGGGGGG~
GPY55GGGGGB!  !BGGGGGGGGGG!
GG5Y55Y55J~  :5GGGGGGGBGY^
PGGP5!     :7PGGGGGGGPJ^
~YGGGPYJJY5GGGGGP5J7^.      ".to_string(), 28)
}
