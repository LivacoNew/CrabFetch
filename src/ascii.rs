// Return type is the ascii & the maximum length of it
pub fn get_ascii(os: &str) -> (String, u64) {
    match os {
        "arch" => arch(),
        _ => ("".to_string(), 0)
    }
}

// Define art down below here
fn arch() -> (String, u64) {
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
