pub fn parse_file_extension(base64: &String) -> Option<String> {
    let start_index = base64.find('/')? + 1; // Find the index of the slash and add 1 to skip it.
    let end_index = base64.find(';')?; // Find the index of the semicolon.
    let file_extension = &base64[start_index..end_index];

    Some(file_extension.to_string())
}

pub fn parse_file_mimetype(base64: &String) -> Option<String> {
    let start_index = base64.find(':')? + 1; // Find the index of the slash and add 1 to skip it.
    let end_index = base64.find(';')?; // Find the index of the semicolon.
    let file_extension = &base64[start_index..end_index];

    Some(file_extension.to_string())
}
