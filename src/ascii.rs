// Return type is the ascii & the maximum length of it
pub fn get_ascii(os: &str) -> (String, u64) {
    match os {
        "arch" => arch(),
        _ => ("".to_string(), 0)
    }
}

// Define art down below here
fn arch() -> (String, u64) {
("                  .o+`
                 `ooo/
                `+oooo:
               `+oooooo:
               -+oooooo+:
             `/:-:++oooo+:
            `/++++/+++++++:
           `/++++++++++++++:
          `/+++ooooooooooooo/`
         ./ooosssso++osssssso+`
        .oossssso-````/ossssss+`
       -osssssso.      :ssssssso.
      :osssssss/        osssso+++.
     /ossssssss/        +ssssooo/-
   `/ossssso+/:-        -:/+osssso+-
  `+sso+:-`                 `.-/+oso:
 `++:.                           `-/+/
 .`                                 `/"
 .to_string(), 40)
}
