use crate::model::*;

pub trait ObjectExt {
    fn data_path(&self) -> String;
}

impl ObjectExt for Object {
    fn data_path(&self) -> String {
        object_data_path(self.id, self.extension.as_deref())
    }
}

impl ObjectExt for ObjectPreview {
    fn data_path(&self) -> String {
        object_data_path(self.id, self.extension.as_deref())
    }
}

fn object_data_path(id: Uuid, extension: Option<&str>) -> String {
    let ext = extension.map(|ext| format!(".{ext}")).unwrap_or_default();
    format!("/object/{id}/data{ext}")
}
