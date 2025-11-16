use std::path::PathBuf;
use bitcode::{Decode, Encode};
use redb::{Database, DatabaseError, TableDefinition};

const CHUNK_SIZE: usize = 1024 * 64;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Object {
    id: [u8; 8],
    name: String,
    
    r#type: ObjectType,
    
    modified_time: u64,
    creation_time: u64,
    
    size: Option<u64>,
    hash: Option<[u8; 32]>,
    chunks: Option<Vec<ChunkInfo>>,
    
    is_deleted: bool,
    
    parent: Option<[u8; 8]>,
    children: Vec<[u8; 8]>
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum ObjectType {
    File,
    Directory
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ChunkInfo {
    offset: u64,
    hash: [u8; 32],
    size: u64
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ChunkTable {
    chunks: Vec<Vec<u8>>,
    total_chunks: u64
}

pub struct ObjectDatabase<'a> {
    internal_database: Database,
    pub object_table: TableDefinition<'a, [u8; 8], Vec<u8>>,
    pub chunk_tables: TableDefinition<'a, [u8; 8], Vec<u8>>,
}

impl ObjectDatabase<'_> {
    pub fn open_or_create(path: &PathBuf) -> Result<ObjectDatabase, DatabaseError> {
        let file_database = Database::create(path)?;
     
        Ok(ObjectDatabase {
            internal_database: file_database,
            object_table: TableDefinition::new("OBJECTS"),
            chunk_tables: TableDefinition::new("CHUNKS")
        })
    }
    pub fn get_object(&self, id: &[u8; 4]) -> Object {
        todo!()
    }

    pub fn save_object(&self, object: Object) {
        todo!()
    }
    
    pub fn get_chunk(&self, object_id: &[u8; 4], chunk_offset: u64) -> &Vec<u8> {
        todo!()
    }
    
    pub fn save_chunk(&self, object_id: &[u8; 4], chunk_offset: u64) -> Result<(), ()> {
        todo!()
    }
}
