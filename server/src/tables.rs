use redb::TableDefinition;

pub const OBJECTS_TABLE: TableDefinition<[u8; 4], Vec<u8>> = TableDefinition::new("objects");
