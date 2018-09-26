use cas::{
    content::{Address, AddressableContent},
    storage::ContentAddressableStorage,
};
use error::HolochainError;
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, MAIN_SEPARATOR},
};

pub struct FileContentAddressableStorage {
    path: String,
}

impl FileContentAddressableStorage {
    pub fn new(path: &str) -> FileContentAddressableStorage {
        FileContentAddressableStorage {
            path: path.to_string(),
        }
    }

    fn address_to_path(&self, address: &Address) -> String {
        format!("{}{}{}.json", self.path, MAIN_SEPARATOR, address)
    }
}

impl ContentAddressableStorage for FileContentAddressableStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        create_dir_all(&self.path)?;
        Ok(write(
            self.address_to_path(&content.address()),
            content.content(),
        )?)
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(Path::new(&self.address_to_path(address)).is_file())
    }

    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        if self.contains(&address)? {
            Ok(Some(C::from_content(&read_to_string(
                self.address_to_path(address),
            )?)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::{
            tests::{ExampleAddressableContent, OtherExampleAddressableContent},
            AddressableContent,
        },
        file::FileContentAddressableStorage,
        storage::ContentAddressableStorage,
    };
    use tempfile::{tempdir, TempDir};

    pub fn test_file_cas() -> (FileContentAddressableStorage, TempDir) {
        let dir = tempdir().unwrap();
        (
            FileContentAddressableStorage::new(dir.path().to_str().unwrap()),
            dir,
        )
    }

    #[test]
    /// show that content of different types can round trip through the same storage
    /// this is copied straight from the example with a file CAS
    fn file_content_round_trip_test() {
        let content = ExampleAddressableContent::from_content(&"foo".to_string());
        let other_content = OtherExampleAddressableContent::from_content(&"bar".to_string());
        let (mut cas, _dir) = test_file_cas();

        assert_eq!(Ok(false), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));

        // round trip some AddressableContent through the FileContentAddressableStorage
        assert_eq!(Ok(()), cas.add(&content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));

        // multiple types of AddressableContent can sit in a single FileContentAddressableStorage
        // the safety of this is only as good as the hashing algorithm(s) used
        assert_eq!(Ok(()), cas.add(&other_content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(true), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));
        assert_eq!(
            Ok(Some(other_content.clone())),
            cas.fetch(&other_content.address())
        );
    }

}