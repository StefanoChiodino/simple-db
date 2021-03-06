mod simple_db {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::fs;
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Seek, SeekFrom};
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    pub enum Errors {
        NotFound,
    }

    struct Table {
        // GUID -> (position, length)
        data_map: HashMap<String, (u32, u32)>,
        indexes: Vec<&'static dyn Fn(dyn Any) -> dyn Any>,
    }

    impl Table {
        fn new() -> Self {
            Self {
                data_map: HashMap::new(),
                indexes: Vec::new(),
            }
        }
    }

    pub struct Db {
        name: String,
        file: File,
        tables: HashMap<TypeId, Table>,
    }

    impl Db {
        #[allow(dead_code)]
        pub fn new(name: String) -> Self {
            let root_folder_path: PathBuf = Path::new("data").to_owned();
            let db_filename = &format!("{}.sdb", name.as_str());
            match fs::create_dir(&root_folder_path) {
                _ => (),
            };
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&root_folder_path.join(db_filename))
                .unwrap();
            Self {
                name: name.to_string(),
                file,
                tables: HashMap::new(),
                // indexes: Default::default(),
            }
        }

        #[allow(dead_code)]
        pub fn post<T: 'static + serde::ser::Serialize + Sized>(
            &mut self,
            obj: T,
        ) -> Result<String, Errors> {
            let new_id = Uuid::new_v4().to_string();
            let data_location = self.file.metadata().unwrap().len();
            self.file.seek(SeekFrom::Start(data_location)).unwrap();
            let serialised_value = bincode::serialize(&obj).unwrap();
            println!(
                "POST: writing to location {:?} value {:?}",
                data_location, serialised_value
            );
            bincode::serialize_into(&mut self.file, &serialised_value).unwrap();
            if self.tables.contains_key(&obj.type_id()) == false {
                self.tables.insert(obj.type_id(), Table::new());
            }
            let table = self.tables.get_mut(&obj.type_id()).unwrap();
            table.data_map.insert(
                new_id.to_string(),
                (data_location as u32, serialised_value.len() as u32),
            );
            Ok(new_id)
        }

        #[allow(dead_code)]
        pub fn get<T: 'static + serde::de::DeserializeOwned>(
            &mut self,
            id: &String,
        ) -> Result<T, Errors> {
            match self
                .tables
                .get(&TypeId::of::<T>())
                .unwrap()
                .data_map
                .get(id)
            {
                Some((position, size)) => {
                    let offset_position = position + 8;
                    let offset_size = size;

                    println!(
                        "GET: id {} type {:?} position {} size {} offset position {} offset size {}",
                        id, TypeId::of::<T>(), position, size, offset_position, offset_size,
                    );
                    let mut raw_data: Vec<u8> = Vec::with_capacity(*offset_size as usize);
                    raw_data.resize(*offset_size as usize, 0);
                    self.file
                        .seek(SeekFrom::Start(offset_position as u64))
                        .unwrap();
                    self.file.read_exact(raw_data.as_mut()).unwrap();
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
        pub fn delete<T: 'static + serde::de::DeserializeOwned>(
            &mut self,
            id: &String,
        ) -> Result<(), Errors> {
            match self.tables.get_mut(&TypeId::of::<T>()) {
                Some(table) => match table.data_map.remove(id) {
                    Some(_) => Ok(()),
                    None => Err(Errors::NotFound),
                },
                None => Err(Errors::NotFound),
            }
        }

        // #[allow(dead_code)]
        // pub fn find<T: serde::de::DeserializeOwned>(
        //     &mut self,
        //     predicate: fn(&T) -> bool,
        //     limit: usize,
        // ) -> Result<Option<Vec<T>>, Errors> {
        //     let matches: Vec<T> = self
        //         .data_map
        //         .keys()
        //         .map(|k| self.get::<T>(k))
        //         .filter_map(|r| match r {
        //             Ok(item) => {
        //                 if predicate(&item) {
        //                     Some(item)
        //                 } else {
        //                     None
        //                 }
        //             }
        //             _ => None,
        //         })
        //         .take(limit)
        //         .collect();
        //     return if matches.is_empty() {
        //         Ok(None)
        //     } else {
        //         Ok(Some(matches))
        //     };
        //     // Err(Errors::NotFound)
        // }

        // #[allow(dead_code)]
        // pub fn find_one<T: serde::de::DeserializeOwned>(
        //     &mut self,
        //     predicate: fn(&T) -> bool,
        // ) -> Result<Option<T>, Errors> {
        //     match self.find(predicate, 1) {
        //         Ok(items_option) => match items_option {
        //             Some(items) => Ok(Some(items.into_iter().nth(0).unwrap())),
        //             None => Ok(None),
        //         },
        //         Err(_) => Err(Errors::NotFound),
        //     }
        // }
    }

    #[cfg(test)]
    mod tests {
        use crate::simple_db::*;
        use serde::Deserialize;
        use serde::Serialize;
        use std::fs;
        use std::panic;
        use uuid::Uuid;

        fn seeded_db() -> Db {
            Db::new(Uuid::new_v4().to_string())
        }

        fn nuke_db(db: Db) {
            match fs::remove_file(format!("{}.sdb", db.name)) {
                _ => (),
            };
        }

        // use crate::simple_db::Crud;
        // #[test]
        // fn safe_filename() {
        //     let mut pairs: HashMap<&str, &str> = HashMap::new();
        //     pairs.insert("test", "test");
        //     pairs.insert("a1", "a1");
        //     // pairs.insert("!@£^@!$£", "_");
        //     pairs.insert("test!@£$123", "test123");
        //     pairs.insert(std::any::type_name::<str>(), "str");
        //
        //     for (input, expected_output) in pairs {
        //         let actual_output = to_safe_filename(input);
        //         assert_eq!(actual_output, expected_output);
        //     }
        // }

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
            let mut db = seeded_db();
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
            let id = db.post(complex).ok().unwrap();
            let retrieved_complex = db.get::<Complex>(&id).ok().unwrap();
            assert_eq!(retrieved_complex.name, "Stefano");
            assert_eq!(retrieved_complex.x, 34);
            db.delete::<Complex>(&id).ok().unwrap();
            assert!(db.get::<Complex>(&id).is_err());
            nuke_db(db);
        }

        // #[test]
        // fn find() {
        //     let mut db = seeded_db();
        //     db.post::<String>("hello".to_string()).ok().unwrap();
        //     let actual = db
        //         .find_one::<String>(|x: &String| x.starts_with("hell"))
        //         .ok()
        //         .unwrap()
        //         .unwrap();
        //     assert_eq!(actual, "hello");
        //     nuke_db(db);
        // }
        //
        // #[test]
        // fn not_found() {
        //     let mut db = seeded_db();
        //     db.post::<String>("hello".to_string()).ok().unwrap();
        //     let actual = db
        //         .find_one::<String>(|x: &String| x.starts_with("hellllooo"))
        //         .ok()
        //         .unwrap();
        //     assert!(actual.is_none());
        //     nuke_db(db);
        // }
    }
}
