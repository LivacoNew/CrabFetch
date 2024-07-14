use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager};

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
    if user_override.is_some() {
        let mut length: u16 = 0;
        user_override.as_ref().unwrap().split('\n').for_each(|x| {
            let len: usize = x.chars().count();
            if len > length as usize { length = len as u16 }
        });
        return (user_override.unwrap(), length)
    }
    let os: &str = &os.replace('"', "");

    match os {
        "arch" => arch(),
        "debian" => debian(),
        "ubuntu" => ubuntu(),
        "fedora" => fedora(),
        "void" => void(),
        "endeavouros" => endeavour(),
        "linuxmint" => mint(),
        _ => ("".to_string(), 0)
    }
}

// Define art down below here
// All distro ASCII's are generated from here; https://www.text-image.com/convert/ascii.html
// I suck at ASCII art, and want to use smaller ones than the other fetch defaults.
fn arch() -> (String, u16) { 
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
fn debian() -> (String, u16) { 
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
fn ubuntu() -> (String, u16) { 
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
fn fedora() -> (String, u16) { 
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
fn void() -> (String, u16) {
("             ..::::::::::..             
           ::::::::::::::::::.          
            .:::::....::::::::::.       
      !^      .          ..::::::.      
     ?5Y?^                  .::::::     
    75YY5J.       ...         .::::.    
J555B5YYY:.~7!7Y5YY5PP?::Y557^5PP5YY55Y!
^B@@@#555?7!G@@&?!!B@@@!G@@#7#@@B7!Y@@@@
 .G@@@#GY. J@@@J!7P@@#JG@@#7#@@#7!J@@@B!
  .YBP5YY: :?YYJY5PY7^!5Y5^75Y5YYY55?~  
    75YY5J.        ...        .::::.    
    .?5YY5Y~                  .::::     
      !Y5YY5J!:.        .:.     ..      
       :?Y5555YY?7!!!!7?YYY!.           
         :!JY55555555555555Y?.          
            :~!?JYYYYYYJ?!^:            ".to_string(), 40)
}
fn endeavour() -> (String, u16) {
("                            
              .!J^          
             ^J555J^        
           :755555557:      
         :!J555555555Y!.    
       :~7Y55555555555Y?^   
     .~7?55555555555555Y?!. 
   .^!7J5555555555555555J?7.
  ^!77Y555555555555555555??!
:!77?5555555555555555555Y???
..:7JJJYYYYYYYYY55555YYJ??7^
  ~777777777777!!!!!~~~^:.  ".to_string(), 28)
}
fn mint() -> (String, u16) {
("              .::^^^^^^::.              
          .^!7????????????7!^.          
        ^!????????????????????!^        
      :7?!!!7???????????????????7:      
     ~???.  7????!~^^~!7~^^~!?????~     
    ~????.  7??7:  ..  . ..  .!????~    
   :?????.  7??^  ^??^  :7?~  .?????:   
   !?????.  7??:  ~??~  ^??7  .?????!   
   7?????.  7??^  ~??~  ^??7  .?????7   
   !?????.  7??:  ~??~  :??7  .?????!   
   :?????.  !??!!!7??7!!!??!  .?????:   
    ~????~  .~!7777777777!~.  ~????~    
     ~????!:.              .:!????~     
      :7????77!!!!!!!!!!!!77????7:      
        ^!????????????????????!^        
          .^!7????????????7!^.          
              .::^^^^^^::.              ".to_string(), 40)
}
