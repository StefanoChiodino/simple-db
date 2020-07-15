mod simple_db {
    use regex::Regex;
    use std::fs;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    pub struct Client {
        name: String,
    }

    pub enum Errors {
        InitialisationError,
        NotFound,
    }

    impl Client {
        #[allow(dead_code)]
        pub fn new(name: String) -> Client {
            println!("Initialised client with name '{}'", name);
            Client { name }
        }

        #[allow(dead_code)]
        pub fn post<T: serde::ser::Serialize>(&self, obj: T) -> Result<usize, Errors> {
            let folder_path = self.get_folder::<T>();
            let new_index = std::fs::read_dir(&folder_path).unwrap().count();
            let file_path = folder_path.join(Path::new(&new_index.to_string()));
            let mut file = File::create(file_path).unwrap();

            bincode::serialize_into(&mut file, &obj).unwrap();
            Ok(new_index)
        }

        #[allow(dead_code)]
        pub fn get<T: serde::de::DeserializeOwned>(&self, index: usize) -> Result<T, Errors> {
            let folder_path = self.get_folder::<T>();
            let file_path = folder_path.join(Path::new(&index.to_string()));
            let read_results = &fs::read(&file_path);
            match read_results {
                Ok(read_bytes) => {
                    let result = bincode::deserialize(&read_bytes);
                    let obj = result.unwrap();
                    Ok(obj)
                }
                Err(_) => Err(Errors::NotFound),
            }
        }

        #[allow(dead_code)]
        pub fn nuke(&self) -> Result<(), Errors> {
            let seeded_folder = Path::new(self.name.as_str());
            fs::remove_dir_all(seeded_folder).unwrap();
            Ok(())
        }

        pub fn get_seeded_folder(&self) -> PathBuf {
            let seed_folder = Path::new(self.name.as_str());
            let base_of_data_path = seed_folder.join("base_of_data");
            if base_of_data_path.exists() == false {
                fs::create_dir_all(&base_of_data_path);
            }
            base_of_data_path
        }
        pub fn get_folder<T>(&self) -> PathBuf {
            let base_of_data_path = self.get_seeded_folder();
            let type_name = std::any::type_name::<T>();
            let safe_type_name = to_safe_filename(type_name);
            let folder_path = base_of_data_path.join(safe_type_name);
            if folder_path.exists() == false {
                fs::create_dir_all(&folder_path).unwrap();
            }
            println!("Got folder with path '{}'", folder_path.display());
            folder_path
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
    use std::panic;
    use uuid::Uuid;

    fn seeded_client() -> Client {
        Client::new(Uuid::new_v4().to_string())
    }

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
        let client = seeded_client();
    }

    #[test]
    fn post() {
        let client = seeded_client();
        client.post("hello");
        let actual = client.nuke();
    }

    #[test]
    fn get() {
        let client = seeded_client();
        let index = client.post::<String>("hello".to_string()).ok().unwrap();
        let actual = client.get::<String>(index).ok().unwrap();
        assert_eq!(actual, "hello");
        let actual = client.nuke();
    }

    #[test]
    fn multiple_get() {
        let client = seeded_client();
        let index1 = client.post::<String>("hello1".to_string()).ok().unwrap();
        let actual1 = client.get::<String>(index1).ok().unwrap();
        assert_eq!(actual1, "hello1");
        let index2 = client.post::<String>("hello2".to_string()).ok().unwrap();
        let actual2 = client.get::<String>(index2).ok().unwrap();
        assert_eq!(actual2, "hello2");
        let actual = client.nuke();
    }

    #[test]
    fn nuke() {
        let client = seeded_client();
        let index = client.post::<String>("hello1".to_string()).ok().unwrap();
        let actual = client.nuke();
        assert!(actual.is_ok());
        assert!(client.get::<String>(index).is_err());
        let actual = client.nuke();
    }
}
