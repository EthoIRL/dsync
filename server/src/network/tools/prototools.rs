use std::error::Error;

pub fn parse_object_id(object_id_vec: &Vec<u8>) -> Result<[u8; 4], Box<dyn Error>> {
    if object_id_vec.len() != 4 {
        return Err(format!("Object_id is an incorrect length. (Expected: 4, Got: {})", object_id_vec.len()).into());
    }

    Ok(object_id_vec[0..4].try_into()?)
}

pub fn object_id_hex(object_id: &[u8; 4]) -> String {
    object_id.iter()
        .map(|byte| format!("{:02X}", byte))
        .collect()
}