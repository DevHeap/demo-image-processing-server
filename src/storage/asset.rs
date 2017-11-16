use rocket::http::ContentType;
use std::str::FromStr;
use std::io;
use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;

pub struct Asset {
    pub path: PathBuf,
    pub content_type: String,
    opened: Option<File>,
}

impl Asset {    
    pub fn new(path: String) -> Self {
        let content_type = path.rfind(".").map_or(ContentType::from_str("text/plain").unwrap(), |dot| {
            let ext = &path[dot+1..];
            ContentType::from_extension(ext).unwrap()
        });

        debug!("Asset {} have been given content type {:?}", path, content_type);

        Asset {
            path: path.into(),
            content_type: content_type.to_string(),
            opened: None
        }
    }

    pub fn open(&mut self) -> Result<(), io::Error> {
        debug!("Opening {:?}", self.path);
        if self.opened.is_none() {
            Ok(self.opened = Some(File::open(&self.path)?))
        } else {
            Ok(())
        }
    }

    pub fn create(&mut self) -> Result<(), io::Error> {
        debug!("Creating {:?}", self.path);
        if self.opened.is_none() {
            Ok(self.opened = Some(File::create(&self.path)?))
        } else {
            Ok(())
        }
    }

    pub fn to_vec(&mut self) -> Result<Vec<u8>, io::Error> {
        self.open()?;
        let mut file = self.opened.take().unwrap(); 
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        Ok(buffer)
    }

    pub fn file(&mut self) -> Result<File, io::Error> {
        self.open()?;
        Ok(self.opened.take().unwrap())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}