use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ImageViewer {
    pub img_index: usize,
}

impl Display for ImageViewer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let query = serde_urlencoded::to_string(self)
            .expect("serialization should always succeed");

        f.write_str(&query)
    }
}
