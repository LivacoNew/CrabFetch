use colored::ColoredString;
use serde::Deserialize;

use crate::{config_manager::{self, Configuration}, formatter::CrabFetchColor};

#[derive(Deserialize)]
pub struct AsciiConfiguration {
    pub display: bool,
    pub side: String,
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
    let os: &str = &os.replace('"', "").to_lowercase();

    let ascii: (&str, u16) = match os {
        "arch" => ARCH,
        "debian" => DEBIAN,
        "ubuntu" => UBUNTU,
        "fedora" => FEDORA,
        "void" => VOID,
        "endeavouros" => ENDEAVOUR,
        "linuxmint" => MINT,
        "elementary" => ELEMENTARY,
        "zorin" => ZORIN,
        "manjaro" => MANJARO,
        "pop" => POPOS,
        "opensuse-tumbleweed" => OPENSUSE,
        "opensuse-leap" => OPENSUSE,
        "bazzite" => BAZZITE,
        "rocky" => ROCKYLINUX,
        "kali" => KALI,
        "almalinux" => ALMA,
        "android" => ANDROID,
        "garuda" => GARUDA,
        "linux" => LINUX,
        _ => ("", 0)
    };

    // I blame rust not letting me make const strings
    let ascii_string: String = ascii.0.to_string();
    (ascii_string, ascii.1)
}

pub fn get_ascii_line(current_line: usize, ascii_split: &[&str], target_length: &u16, config: &Configuration) -> String {
    let percentage: f32 = current_line as f32 / ascii_split.len() as f32;
    let index: u8 = (((config.ascii.colors.len() - 1) as f32) * percentage).round() as u8;

    let mut line = String::new();
    if ascii_split.len() > current_line {
        line = ascii_split[current_line].to_string();
    }
    let remainder: u16 = target_length - (line.chars().count() as u16);
    for _ in 0..remainder {
        line.push(' ');
    }

    if current_line < ascii_split.len() {
        let colored: ColoredString = config.ascii.colors.get(index as usize).unwrap().color_string(&line);
        return colored.to_string();
    }
    line
}

// Define art down below here
// All distro ASCII's are generated from here; https://www.text-image.com/convert/ascii.html
// I suck at ASCII art, and want to use smaller ones than the other fetch defaults.
const ARCH: (&str, u16) =  (
"             ~!
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
^7~:                    :~7^", 28);

const DEBIAN: (&str, u16) = (
"         .^!7!~~~^:
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
         .^~~:..            ", 28);
const UBUNTU: (&str, u16) = (
"           .^~7?JJYYYYJJ?7~^.
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
           .^~7?JJYYYYJJ?7~^.           ", 40);
const FEDORA: (&str, u16) = ( 
"      .^7J5PGGGGGPY7~.
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
~YGGGPYJJY5GGGGGP5J7^.      ", 28);

const VOID: (&str, u16) = (
"             ..::::::::::..             
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
            :~!?JYYYYYYJ?!^:            ", 40);

const ENDEAVOUR: (&str, u16) = (
"                            
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
  ~777777777777!!!!!~~~^:.  ", 28);

const MINT: (&str, u16) = (
"              .::^^^^^^::.              
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
              .::^^^^^^::.              ", 40);

const ELEMENTARY: (&str, u16) = (
"         :!J5PPP555PPPP5?~.         
      :JPPY!^.:^~~~~^::~?5P5!.      
    ^PGJ^  .7YY?!!!7JPY.  .!5BJ.    
  .5#?   .YBJ:       .5&^    :5#!   
 :BG.   !&G:           #G      !&J  
.BG    ?@P             #P       ^&? 
Y&:   ~@#.            J@^        B@^
&5    5@?           .5#~       ^BG@J
@J    P@!          7B5.       ?&J.#5
#P    7@P       :?G5^       7BB~ :@?
?@^    5@Y.  ^7PGJ:      :?BB7   Y&.
 P#.   .Y@#5BGJ^     .^?P#P~    7@! 
  P#5PGGGPYG&BPYYYY5GBG57.     J&7  
   7&#7.    .~7?J??!~:       !BG^   
    .?G5!:                ^JGP!     
      .!YPPJ7~^:....:^!?YPP?^       
          ^!JY5PPPPPP5Y?~.          ", 36);

const ZORIN: (&str, u16) = (
"        !JJJJJJJJJJJJJJJJJJ!        
      .!JJJJJJJJJJJJJJJJJJJJ!.      
       ......................       
                                    
   :^^^^^^^^^^^^^^^^:         .^:   
  !JJJJJJJJJJJJJJ?!^.      .^7?JJ!  
.7JJJJJJJJJJJJ?!:.      .~7JJJJJJJ!.
7JJJJJJJJJJ?~:       :~7JJJJJJJJJJJ7
.7JJJJJJ7~:       :~?JJJJJJJJJJJJJ7.
  !JJ7~.       :!?JJJJJJJJJJJJJJJ!  
   ..         .:::...............   
                                    
      .^::::::::::::::::::::^.      
      .7JJJJJJJJJJJJJJJJJJJJ7.      
        !JJJJJJJJJJJJJJJJJJ!        ", 36);

const MANJARO: (&str, u16) = (
"???????????????????????. :J?????????
???????????????????????. :J?????????
???????????????????????. :J?????????
???????????JJJJJJJJJJJ?. :J?????????
??????????7777777777777. :J?????????
?????????J:              :J?????????
?????????J: .7777777777. :J?????????
?????????J: .?J??????J?. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????
?????????J: .??????????. :J?????????", 36);

const POPOS: (&str, u16) = (
"           .:~!!77777777!!~:.           
        :~!7????????7777????7!~:        
     .^7???7!~^::^~!7?7777777???7^.     
    ^7??7~:.        .~?777777777??7^    
  .!???7.    :!~.     ^?777????777??!.  
 .7?7777:    .7?7^     7??7!!!7?7777?7. 
.7?7777?7:    :7?7.   .7?7:   .~?7777?7.
~?777777?7:    .~^   .~??!     !?77777?~
777777777?7^       .^7?7?^    !?77777777
7777777777??^    .!7??77?:  .!?777777777
777777777777?~    ~?77777. :7?7777777777
~?77777777777?!    !?777?!~7?777777777?~
.7?77777777777?!.  .7??7^:!?777777777?7.
 .7?77777???????7~:^7??7:.!????77777?7. 
  .!??7777~~~~~~~~~~~~~~~~~~~~7777??!.  
    ^7???~                    ~???7^    
     .^7??7!!!!!!!!!!!!!!!!!!7??7^.     
        :~!7????????????????7!~:        
           .:~!!77777777!!~:.           ", 40);

const OPENSUSE: (&str, u16) = (
"           ^7YG#&@@@@@@&#GY7^           
       .~5#@&BPJ7!~~~~!7JPB&@#5~.       
     :J#@#Y~.              .~Y#@#J:     
   .J&@P~ ~!~^:..              !G@&J.   
  ^B@@#?!^G@@@@&#BPY?7~:.        ~B@B^  
 ^&@@@@@@@@@@@@@@@@@@@&#GPY!.     .P@&^ 
:#@@@@@@@@@@@@@@@@@&Y!7??7?B&Y.     P@#:
Y@@@@@@@@@@@@@@@@@&^!#@PYP5.5@G     :&@Y
&@@@@@@@@@@@@@@@@@P B@@Y75@~^@@5     P@&
@@@@@@@@@@@@&P#@@@&!^P&@@#J:P@@@7    Y@@
&@@@@@@@@@@@@Y:~JG&@P?7777J#@@@&Y    P@&
Y@@@@@@@@@@@@@#5?~^~?YPGB##BPJ!~:   :&@Y
:#@@@@@@@@@@@@@@@@&B5J7!~~!7J5B&!   P@#.
 ^&@@@@@@@@@@@@@@@@@@@@@@@@@&GJ^  .P@&^ 
  ^B@@@BPPPGGB######BBGPY?!^.    ~B@B^  
   .J&@G~      .....           !G@&J.   
     :J#@#Y!.              .~Y#@#J:     
       .~5#@&BPJ7!~~~~!7JPB&@#5~.       
           ^7YG#&@@@@@@&#GY7^           ", 40);

const ROCKYLINUX: (&str, u16) = (
"      :!YG#&@@@@&#GY!:      
   .!5#@@@@@@@@@@@@@@#5!.   
  ~G&@@@@@@@@@@@@@@@@@@&G~  
.?&@@@@@@@@@@@@@@@@@@@@@@&?.
7&@@@@@@@@@@@@@@&G5#@@@@@@&7
#@@@@@@@@@@@@@&P!. ^J#@@@@@#
@@@@@@@@@@@@&5~.     :?B@@@@
#@@@@@@@@@#Y^    :^.   :7G@#
7&@@@@@@#J^    :JB&P~.   .!!
.?&@@@B?:    ^Y#@@@@&P!.    
  ~GG7:   .~5#@@@@@@@@&P^   
   ..   .!P&@@@@@@@@@#5!.   
        ~P#&@@@@&#GY!:      ", 28);

const KALI: (&str, u16) = (
"   ....::^^~~!!77!~^:                   
   ..::^~~~!!!!!!7?Y55.                 
 .:::::....:^~!77??JJJ!                 
      .:^!!!~~^:..    PGJ??7!!^:.       
    .:^:..          7G?~^:^~!?YY57.     
                   ~&^         .~GP~.   
                   !@:            :7~   
                    JB7:.               
                     ^?YYJJJJJJJ?!:.    
                         ....::~7JY?!~. 
                                  .77:!^
                                    !7 ^
                                     !: 
                                     .: ", 40);

const ALMA: (&str, u16) = (
"       .5&@&G~ ..       ..   ~??~       
       ~@@@@@5J##B~  :JG##B~5@@@@G.     
       .JGBBPG@@@5. ?&@@@@@BP&@@@G.     
      !#&##&&&#@P  J@@P!~?#@#GGG5:      
      ?@@@#!:..:. :&@Y    ^@@@@@@P      
       5@@#^      :@&.     !7!!7Y!      
     :!.~P@@P7:    GG    .^~!7!~^.      
    J@@B~ :?PBBPJ!::? .75GBBBB#@@&BY^   
 ~??5#@@@B~   ..:..  .^:..    .:5@@@@7  
B@@@@#G@@5.      ^Y^ .YY:      .5@@#B!  
P@@@@GB@@^    :7G&7    P@J  .JG&@#5GBBP!
 ^!!~#@@@#5YPB@@5:      B@5  Y@@@5#@@@@@
     7G#&&&#B5!. ^7.    J@@~ ^@@@P!G##P!
       ....  !7YB@@#J~^7#@@! .7!~.  ..  
             Y@@&BPG#@@@@@B.            
              ^~7PGGPYG&BY:             
                B@@@@#..                
                7B&@B7                  ", 40);

const GARUDA: (&str, u16) = (
"             ^Y#&&&&&&&&&&#J:       
           ~5&@&PY55YY55YG@@#Y^     
         !G@@&Y^     .    ^Y&@@5~   
      .7B@@B?:    .!G#:     :J#@@G!.
    :J#@@G7.    :?#@@@#BBBBBBG#@@@@5
  ^5&@@P~      .?PP555PPPPPPPPP5B@@P
^P@@&Y^                         ?B?:
^5&@@P~  .75P5555555555555555PY:    
  :?#@@G7YGBBBBBBBBBBBBB@@@@@P!.    
    .7G@@#?:         .!G@@#Y^       
       ~P@@&P55555555B@@B?:         
         ^5#&&&&&&&&&&G7.           ", 36);

const BAZZITE: (&str, u16) = (
"     %%%%%%====%%%%%%%%%%            
   %%%%%%%%    %%%%%%%%%%%%%%        
  %%%%%%%%%    %%%%%%%%%%%%%%%%      
  %%%%%%%%%    %%%%%%%%%%%%%%%###    
  %%%%%%%%%    %%%%%%%%%%%%%######   
  ==                  =======######  
  ==                  =========##### 
  %%%%%%%%%    %%%%%%%####======#####
  %%%%%%%%%    %%%%%#######=====#####
  %%%%%%%%%    %%%#########=====#####
  %%%%%%%%%    %%##########=====#####
  %%%%%%%%%====###########=====######
   %%%%%%%%====#########======###### 
    %%%%%%%=====#####========######  
     %%%%###===============#######   
      %#######==========#########    
        #######################      
          ###################        
              ###########           ", 36);

const LINUX: (&str, u16) = (
"        #####        
       #######       
       ##O#O##       
       #######       
     ###########     
    #############    
   ###############   
   ################  
  #################  
#####################
#####################
  #################  ", 21);

const ANDROID: (&str, u16) = (
"          .^^  ........  ^:  
           .~~~!!!!!!!!~~~.   
          :~!!77!!!!!!!!7!!!~:  
      .~!7!!^  !!!!!!!  ^77!^. 
      .~7!!!!~!!!!!!!!!!~!!!!7~. 
      :7!!!!!!!!!!!!!!!!!!!!!!7:      
.:^:. :^^^^^^^^^^^^^^^^^^^^^^^^: .^^:.
!777!::!!!!!!!!!!!!!!!!!!!!!!!!::!77!!
!!!!7^^7!!!!!!!!!!!!!!!!!!!!!!7:^7!!!!
!!!!7^^7!!!!!!!!!!!!!!!!!!!!!!7:^7!!!!
!!!!7^:7!!!!!!!!!!!!!!!!!!!!!!7:^7!!!!
!!!!7^:7!!!!!!!!!!!!!!!!!!!!!!7:^7!!!!
!!!!7^:7!!!!!!!!!!!!!!!!!!!!!!7:^7!!!!
!7777:^7!!!!!!!!!!!!!!!!!!!!!!7::7777!
.^~^: ^7!!!!!!!!!!!!!!!!!!!!!!7: :^~^.
      :77!7!!!!!!!77!!!!!!!7!7!:      
      .:~~~!!!!!!~~~~!!!!!~~~^:.
           ^7!!!!.  .!!!!7:     
           ^7!!!!.  .!!!!7:  
           ^7!!!!.  .!!!!7: 
           .~!!!^   .^!!!^. ", 38);
