use crate::config_manager;

// Return type is the ascii & the maximum length of it
pub fn get_ascii(os: &str) -> (String, u16) {
    // Will first confirm if theres a ascii override file
    let use_me_instead: Option<String> = config_manager::check_for_ascii_override();
    if use_me_instead != None {
        let mut length: u16 = 0;
        use_me_instead.as_ref().unwrap().split("\n").for_each(|x| if x.len() > length as usize { length = x.len() as u16 });
        return (use_me_instead.unwrap(), length)
    }

    match os {
        "arch" => arch(),
        _ => ("".to_string(), 0)
    }
}

// Define art down below here
fn arch() -> (String, u16) {
("             ..
             -=.
            -==-
           -====-
          -======-
         -========-
        -==========-
       -============-
      -====-:  .-====-
     -=====.     =====-
    -=====-      -=====-.
  .-====--:      :--====-.
 .-=-:.              .:-==.
.:.                      .:."
 .to_string(), 28)
}
