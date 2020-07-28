mod simple_db {
    use std::collections::BTreeMap;
    use std::fs;
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;
    use uuid::Uuid;

    pub enum Errors {
        NotFound,
    }

    pub(crate) struct Db {
        name: String,
        data_map: BTreeMap<String, (u32, u32)>,
        file: File,
        // indexes: HashMap<T, dyn Fn(&str) -> T>,
    }

    impl Db {
        pub fn new(name: String) -> Self {
            Self {
                name: name.to_string(),
                data_map: BTreeMap::new(),
                file: OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(Path::new(&format!("{}.sdb", name.as_str())))
                    .unwrap(),
            }
        }

        #[allow(dead_code)]
        pub fn post<T: serde::ser::Serialize>(&mut self, obj: T) -> Result<String, Errors> {
            let new_id = Uuid::new_v4().to_string();
            let data_location = self.file.metadata().unwrap().len();
            println!("data location {:?}", data_location);
            self.file.seek(SeekFrom::Start(data_location));
            let serialised_value = bincode::serialize(&obj).unwrap();
            println!("Serialised value {:?}", serialised_value);
            bincode::serialize_into(&mut self.file, &serialised_value).unwrap();
            self.data_map.insert(
                new_id.to_string(),
                (data_location as u32, serialised_value.len() as u32),
            );
            // self.file.flush();
            self.file.seek(SeekFrom::Start(0));
            println!(
                "After Writing - file bytes contents {:?}",
                fs::read(&format!("{}.sdb", self.name.as_str()))
            );
            Ok(new_id)
        }

        #[allow(dead_code)]
        pub fn get<T: serde::de::DeserializeOwned>(&mut self, id: &String) -> Result<T, Errors> {
            match self.data_map.get(id) {
                Some((position, size)) => {
                    let offset_position = position + 8;
                    let offset_size = size;
                    self.file.seek(SeekFrom::Start(0));
                    println!(
                        "Before reading - file bytes contents {:?}",
                        fs::read(&format!("{}.sdb", self.name.as_str()))
                    );

                    println!(
                        "GET: position {} size {} offset position {} offset size {}",
                        position, size, offset_position, offset_size,
                    );
                    let mut raw_data: Vec<u8> = Vec::with_capacity(*offset_size as usize);
                    println!("raw data size {}", raw_data.len());
                    raw_data.resize(*offset_size as usize, 0);
                    println!("raw data size {}", raw_data.len());
                    self.file.seek(SeekFrom::Start(offset_position as u64));
                    self.file.read_exact(raw_data.as_mut()).unwrap();
                    println!("raw data size {}", raw_data.len());
                    Ok(bincode::deserialize(raw_data.as_slice()).unwrap())
                }
                _ => Err(Errors::NotFound),
            }
        }

        #[allow(dead_code)]
        pub fn nuke(&self) -> Result<(), Errors> {
            Err(Errors::NotFound)
        }

        #[allow(dead_code)]
        pub fn delete<T: serde::de::DeserializeOwned>(&self, id: &String) -> Result<(), Errors> {
            Err(Errors::NotFound)
        }

        #[allow(dead_code)]
        pub fn find<T: serde::de::DeserializeOwned>(
            &self,
            predicate: fn(&T) -> bool,
            limit: usize,
        ) -> Result<Option<Vec<T>>, Errors> {
            Err(Errors::NotFound)
        }

        #[allow(dead_code)]
        pub fn find_one<T: serde::de::DeserializeOwned>(
            &self,
            predicate: fn(&T) -> bool,
        ) -> Result<Option<T>, Errors> {
            Err(Errors::NotFound)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::simple_db::*;
        use serde::Deserialize;
        use serde::Serialize;
        use std::collections::HashMap;
        use std::fs;
        use std::panic;
        use uuid::Uuid;

        fn seeded_db() -> Db {
            Db::new(Uuid::new_v4().to_string())
        }

        fn nuke_db(db: Db) {
            fs::remove_file(format!("{}.sdb", db.name));
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
                // let actual_output = to_safe_filename(input);
                // assert_eq!(actual_output, expected_output);
            }
        }

        #[test]
        fn create_db() {
            let db = seeded_db();
            nuke_db(db);
        }

        #[test]
        fn post() {
            let mut db = seeded_db();
            db.post("hello").ok().unwrap();
            nuke_db(db);
        }

        #[test]
        fn get() {
            let mut db = seeded_db();
            let id = db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db.get::<String>(&id).ok().unwrap();
            assert_eq!(actual, "hello");
            nuke_db(db);
        }

        #[test]
        fn multiple_get() {
            let mut db = seeded_db();
            let id1 = db.post::<String>("hello1".to_string()).ok().unwrap();
            let actual1 = db.get::<String>(&id1).ok().unwrap();
            assert_eq!(actual1, "hello1");
            let id2 = db.post::<String>("hello2".to_string()).ok().unwrap();
            let actual2 = db.get::<String>(&id2).ok().unwrap();
            assert_eq!(actual2, "hello2");
            nuke_db(db);
        }

        #[test]
        fn nuke() {
            let mut db = seeded_db();
            let id = db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db.nuke();
            assert!(actual.is_ok());
            assert!(db.get::<String>(&id).is_err());
            nuke_db(db);
        }

        #[test]
        fn delete() {
            let mut db = seeded_db();
            let id = db.post::<String>("hello".to_string()).ok().unwrap();
            assert!(db.get::<String>(&id).is_ok());
            db.delete::<String>(&id).ok().unwrap();
            assert!(db.get::<String>(&id).is_err());
            nuke_db(db);
        }

        #[test]
        fn delete_non_existing_id() {
            let db = seeded_db();
            let result = db.delete::<String>(&"made_up".to_string());
            assert!(result.is_err());
            nuke_db(db);
        }

        #[test]
        fn complex_object_workflow() {
            #[derive(Serialize, Deserialize, PartialEq, Debug)]
            struct Complex {
                name: String,
                x: i32,
            }
            let complex = Complex {
                name: "Stefano".to_string(),
                x: 34,
            };
            let mut db = seeded_db();
            let id = db.post(&complex).ok().unwrap();
            let retrieved_complex = db.get::<Complex>(&id).ok().unwrap();
            assert_eq!(retrieved_complex, complex);
            db.delete::<Complex>(&id).ok().unwrap();
            assert!(db.get::<Complex>(&id).is_err());
            nuke_db(db);
        }

        #[test]
        fn find() {
            let mut db = seeded_db();
            db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db
                .find_one::<String>(|x: &String| x.starts_with("hell"))
                .ok()
                .unwrap()
                .unwrap();
            assert_eq!(actual, "hello");
            nuke_db(db);
        }

        #[test]
        fn not_found() {
            let mut db = seeded_db();
            db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db
                .find_one::<String>(|x: &String| x.starts_with("hellllooo"))
                .ok()
                .unwrap();
            assert!(actual.is_none());
            nuke_db(db);
        }
    }
}
