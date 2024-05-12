use uuid::{uuid, Uuid};

pub mod objects {
    use super::*;

    pub const BUNNY: Uuid = uuid!("21b700d0-724f-4642-8f50-c5b8fbffe6c9");
    pub const SAND: Uuid = uuid!("49b870e4-1b21-4feb-9a3f-4a3ffd101204");
}

pub mod posts {
    use super::*;

    // Programming Languages
    pub const C: Uuid = uuid!("9bb115bc-9a32-4d13-9f0b-f38ce3a18bc0");
    pub const CPP: Uuid = uuid!("e0fac289-d8b9-4435-b73e-d11007d879da");
    pub const JAVA: Uuid = uuid!("21cfd03e-be9c-4549-9758-c0696088122b");
    pub const JS: Uuid = uuid!("4a2c7232-8a70-46ba-957b-50293b08bb73");
    pub const RUST: Uuid = uuid!("06b8413b-5dd7-4b87-a183-34552608ddeb");

    // Media
    pub const BUNNY: Uuid = uuid!("5cf6bc10-db1f-4159-ba96-5c075ce3a072");
    pub const SAND: Uuid = uuid!("80e0c042-beef-4a30-99ac-07063327d01a");
}

pub mod tags {
    use super::*;

    pub const LANGUAGES: Uuid = uuid!("7d0f4f2c-a08e-4a70-b71a-ae9ea54ffc6e");
    pub const PHOTOS: Uuid = uuid!("ceaa85e6-6a89-4053-b64b-dde3e817f45a");
    pub const VIDEOS: Uuid = uuid!("04a7b298-3d23-4916-a2c7-62fb201fc40d");
}

pub mod users {
    use super::*;

    pub const MINTY: Uuid = uuid!("99786976-95bd-49ff-892e-cd76580aec5a");
}
