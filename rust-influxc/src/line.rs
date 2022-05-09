//!
//! Snippet Archive for Line Protocol Formatting/Parsing
//!


// pub fn from_line(line: &str) -> InfluxResult<Self>
// {
//     let parts = line.split(" ")
//         .collect::<Vec<&str>>();

//     if parts.len() != 3 {
//         return Err(format!("Invalid measurement line: '{}'. Please consult the InfluxDB line protocol documentation", line).into())
//     }

//     let part_msrmt  = parse_measurement(parts[0])?;
//     let part_fields = parse_fields(parts[1])?;
//     let part_tstamp = parts[2];

//     let mut this = Self::new(part_msrmt.0);

//     this.tags      = part_msrmt.1;
//     this.fields    = part_fields;
//     this.timestamp = Some(part_tstamp);

//     Ok(this)
// }


// fn parse_measurement<'m>(fragment: &'m str) -> InfluxResult<(&'m str, BTreeMap<String, String>)>
// {
//     let mut tags  = BTreeMap::new();
//     let mut parts = fragment.split(",");

//     if let Some(msrmt) = parts.next()
//     {
//         while let Some(tag) = parts.next()
//         {
//             let tag_parts = tag.split("=")
//                 .collect::<Vec<&str>>();

//             if tag_parts.len() != 2 {
//                 return Err("All tags must have a key=value format".into());
//             }

//             let key   = tag_parts[0].to_owned();
//             let value = tag_parts[1].to_owned();

//             tags.insert(key, value);
//         }

//         Ok((msrmt, tags))
//     }
//     else {
//         Err("Measurement is missing in line".into())
//     }
// }


// fn parse_fields(fragment: &str) -> InfluxResult<BTreeMap<String, String>>
// {
//     let mut fields = BTreeMap::new();
//     let mut parts  = fragment.split(",");

//     while let Some(field) = parts.next()
//     {
//         let field_parts = field.split("=")
//             .collect::<Vec<&str>>();

//         if field_parts.len() != 2 {
//             return Err("All fields must have a key=value format".into());
//         }

//         let key   = field_parts[0].to_owned();
//         let value = field_parts[1].to_owned();

//         fields.insert(key, value);
//     }

//     Ok(fields)
// }
