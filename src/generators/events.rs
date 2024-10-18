use crate::{
    generators::common::{indent, make_defined_types_fields},
    types::Event,
    IDL,
};
use convert_case::{Case, Casing};

pub fn make_events(idl: &IDL) -> String {
    idl.events
        .iter()
        .map(|event| {
            let event_name_pascal = event.name.to_case(Case::Pascal);

            let fields = match event.fields.is_empty() {
                true => {
                    if let Some(matched_type) =
                        idl.types.iter().find(|t| t.name == event_name_pascal)
                    {
                        indent(make_defined_types_fields(matched_type.clone()))
                    } else {
                        make_event_props(event)
                    }
                }
                false => make_event_props(event),
            };

            format!(
                "#[cfg_attr(not(target_os=\"solana\"), derive(Debug))]
#[derive(AnchorDeserialize, serde::Serialize)]
pub struct {} {{
{}
}}

impl Discriminator for {} {{
    const DISCRIMINATOR: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    fn discriminator() -> [u8; 8] {{
        get_event_discriminator(\"{}\")
    }}
}}",
                event_name_pascal,
                indent(fields),
                event_name_pascal,
                event_name_pascal
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

pub fn make_events_enum(idl: &IDL) -> String {
    let name = idl.name.clone().unwrap().to_case(Case::Pascal);
    let values = idl
        .events
        .iter()
        .map(|event| {
            let event_name_pascal = event.name.to_case(Case::Pascal);
            format!("{}({})", &event_name_pascal, &event_name_pascal)
        })
        .collect::<Vec<String>>()
        .join(",\n");
    format!(
        "#[derive(Debug, serde::Serialize)]
        pub enum {}Event {{
        {},
        Unknown(String),
    }}",
        name, values
    )
}

pub fn make_introspect_helper(idl: &IDL) -> String {
    let name = idl.name.clone().unwrap().to_case(Case::Pascal);
    let snake_name = idl.name.clone().unwrap().to_case(Case::Snake);
    let values = idl
        .events
        .iter()
        .map(|event| {
            let event_name = event.name.to_case(Case::Pascal);
            format!(
                "if {}::discriminator().eq(discriminator) {{
                return Some((
                    \"{}.{}\".to_string(),
                    {}Event::{}(
                        {}::try_from_slice(buffer).expect(\"Invalid data of {}\"),
                    ),
                ));
            }}",
                event_name,
                snake_name.clone(),
                event_name,
                name,
                event_name,
                event_name,
                event_name
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "pub fn introspect_log(log: String) -> Option<(String, {})> {{
    if log.starts_with(\"Program data: \") {{
        msg!(\"Will try to introspect log {{}}\", log.clone());
        let data = log.strip_prefix(\"Program data: \").unwrap();
        const DISCRIMINATOR_SIZE: usize = 8;

        let bytes = base64::prelude::BASE64_STANDARD.decode(data).ok()?;

        let (discriminator, buffer) = bytes.split_at(DISCRIMINATOR_SIZE);

        {}

    }}
}}",
        name, values
    )
}

pub fn make_event_props(event: &Event) -> String {
    event
        .fields
        .iter()
        .map(|t| {
            format!(
                "        pub {}: {}",
                t.name.to_case(Case::Snake),
                t.kind.to_string()
            )
        })
        .collect::<Vec<String>>()
        .join(",\n")
}
