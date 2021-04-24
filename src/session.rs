use diesel_patches::models::User;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Session(Arc<Mutex<HashMap<u64, User>>>);

impl Session {
    pub fn new() -> Self {
        Session(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn get(&self, k: u64) -> Option<User> {
        self.0.lock().unwrap().get(&k).map(|u| u.clone())
    }

    pub fn put(&self, user: User) -> u64 {
        let mut map = self.0.lock().unwrap();
        loop {
            let id = rand::random::<u64>();

            if map.contains_key(&id) {
                continue;
            }

            map.insert(id, user);
            return id;
        }
    }
}
