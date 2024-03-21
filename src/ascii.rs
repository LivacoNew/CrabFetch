use crate::config_manager;

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
