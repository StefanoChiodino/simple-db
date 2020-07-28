mod simple_db {
    use std::collections::HashMap;
    use std::fs::File;

    pub enum Errors {
        NotFound,
    }

    pub(crate) struct Db {
        name: String,
        data: HashMap<String, u32>,
        // indexes: HashMap<T, dyn Fn(&str) -> T>,
    }

    impl Db {
        pub fn new(name: String) -> Self {
            Self {
                name,
                data: HashMap::new(),
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

        #[allow(dead_code)]
        pub fn get<T: serde::de::DeserializeOwned>(&self, id: &String) -> Result<T, Errors> {
            Err(Errors::NotFound)
        }

        #[allow(dead_code)]
        pub fn post<T: serde::ser::Serialize>(&self, obj: T) -> Result<String, Errors> {
            Err(Errors::NotFound)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::simple_db::*;
        use serde::Deserialize;
        use serde::Serialize;
        use std::collections::HashMap;
        use std::panic;
        use uuid::Uuid;

        fn seeded_db() -> Db {
            Db::new(Uuid::new_v4().to_string())
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
            let _db = seeded_db();
        }

        #[test]
        fn post() {
            let db = seeded_db();
            db.post("hello").ok().unwrap();
            db.nuke().ok().unwrap();
        }

        #[test]
        fn get() {
            let db = seeded_db();
            let id = db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db.get::<String>(&id).ok().unwrap();
            assert_eq!(actual, "hello");
            db.nuke().ok().unwrap();
        }

        #[test]
        fn multiple_get() {
            let db = seeded_db();
            let id1 = db.post::<String>("hello1".to_string()).ok().unwrap();
            let actual1 = db.get::<String>(&id1).ok().unwrap();
            assert_eq!(actual1, "hello1");
            let id2 = db.post::<String>("hello2".to_string()).ok().unwrap();
            let actual2 = db.get::<String>(&id2).ok().unwrap();
            assert_eq!(actual2, "hello2");
            db.nuke().ok().unwrap();
        }

        #[test]
        fn nuke() {
            let db = seeded_db();
            let id = db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db.nuke();
            assert!(actual.is_ok());
            assert!(db.get::<String>(&id).is_err());
            db.nuke().ok().unwrap();
        }

        #[test]
        fn delete() {
            let db = seeded_db();
            let id = db.post::<String>("hello".to_string()).ok().unwrap();
            assert!(db.get::<String>(&id).is_ok());
            db.delete::<String>(&id).ok().unwrap();
            assert!(db.get::<String>(&id).is_err());
            db.nuke().ok().unwrap();
        }

        #[test]
        fn delete_non_existing_id() {
            let db = seeded_db();
            let result = db.delete::<String>(&"made_up".to_string());
            assert!(result.is_err());
            db.nuke().ok().unwrap();
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
            let db = seeded_db();
            let id = db.post(&complex).ok().unwrap();
            let retrieved_complex = db.get::<Complex>(&id).ok().unwrap();
            assert_eq!(retrieved_complex, complex);
            db.delete::<Complex>(&id).ok().unwrap();
            assert!(db.get::<Complex>(&id).is_err());
            db.nuke().ok().unwrap();
        }

        #[test]
        fn find() {
            let db = seeded_db();
            db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db
                .find_one::<String>(|x: &String| x.starts_with("hell"))
                .ok()
                .unwrap()
                .unwrap();
            assert_eq!(actual, "hello");
            db.nuke().ok().unwrap();
        }

        #[test]
        fn not_found() {
            let db = seeded_db();
            db.post::<String>("hello".to_string()).ok().unwrap();
            let actual = db
                .find_one::<String>(|x: &String| x.starts_with("hellllooo"))
                .ok()
                .unwrap();
            assert!(actual.is_none());
            db.nuke().ok().unwrap();
        }
    }
}
