use redb::TableDefinition;

pub const OBJECTS_TABLE: TableDefinition<[u8; 4], Vec<u8>> = TableDefinition::new("objects");
pub const OBJECTS_HASH_TABLE: TableDefinition<[u8; 4], Vec<u64>> = TableDefinition::new("objects-hash");
pub const OBJECTS_CHUNK_TABLE: TableDefinition<[u8; 8], Vec<u8>> = TableDefinition::new("objects-chunks");