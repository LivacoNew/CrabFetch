use std::collections::HashMap;

pub fn get_ascii(desired: &str) -> String {
    arch().get(desired).unwrap().to_string()
}

// Define art down below here
fn arch() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("default".to_string(), "
                  .o+`
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
 .`                                 `/".to_string());

    map
}
