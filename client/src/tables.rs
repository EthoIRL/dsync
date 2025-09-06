use redb::TableDefinition;

pub const OBJECTS_LOCAL_TABLE: TableDefinition<[u8; 4], String> = TableDefinition::new("object_paths");