mod simple_db {
    use regex::Regex;
    use std::fs::File;
    use std::path::Path;
    // pub trait Crud {
    //     // fn new() -> Self;
    //     fn post<T>(Self, obj: T) -> Result<(), Errors>;
    // }

    pub struct Client {}
    // pub struct Helper {}

    pub enum Errors {
        InitialisationError,
    }

    impl Client {
        pub fn post<T: serde::ser::Serialize>(&self, obj: T) -> Result<usize, Errors> {
            let type_name = std::any::type_name::<T>();
            let safe_type_name = to_safe_filename(type_name);
            let base_of_data_path = Path::new("base_of_data");
            if base_of_data_path.exists() == false {
                std::fs::create_dir(base_of_data_path);
            }
            let folder_path = base_of_data_path.join(safe_type_name);
            if folder_path.exists() == false {
                std::fs::create_dir(&folder_path);
            }

            let new_index = std::fs::read_dir(&folder_path).unwrap().count();

            let mut file =
                File::create(folder_path.join(Path::new(&new_index.to_string()))).unwrap();

            bincode::serialize_into(&mut file, &obj).unwrap();
            Ok(new_index)
        }
    }

    pub fn to_safe_filename(input: &str) -> String {
        let re = Regex::new(r"[^\w\d]").unwrap();
        re.replace_all(input, "").to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::simple_db::*;
    use std::collections::HashMap;

    // use crate::simple_db::Crud;
    #[test]
    fn safe_filename() {
        let mut pairs: HashMap<&str, &str> = HashMap::new();
        pairs.insert("test", "test");
        pairs.insert("a1", "a1");
        // pairs.insert("!@£^@!$£", "_");
        pairs.insert("test!@£$123", "test123");
        pairs.insert(std::any::type_name::<str>(), "str");

        for (input, expected_output) in pairs {
            let actual_output = to_safe_filename(input);
            assert_eq!(actual_output, expected_output);
        }
    }

    #[test]
    fn create_client() {
        let client = Client {};
        // assert!(!client.is_err());
    }

    #[test]
    fn post() {
        let client = Client {};
        client.post("hello");
    }
}
