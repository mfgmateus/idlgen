use crate::{generators::common::make_ix_has_info, IDL};
use convert_case::{Case, Casing};

pub fn make_i11n_accounts(idl: &IDL) -> String {
    idl.instructions
        .iter()
        .map(|ix| {
            let ix_name_pascal = ix.name.to_case(Case::Pascal);
            format!(
                "    #[derive(TryFromAccountMetas, serde::Serialize)]
    pub struct {}AccountMetas{} {{
{}
    }}",
                ix_name_pascal,
                make_ix_has_info(ix),
                ix.accounts
                    .iter()
                    .map(|a| {
                        let kind = match a.isOptional {
                            true => "Option<&'info AccountMeta>",
                            false => "&'info AccountMeta",
                        };
                        format!("        pub {}: {},", a.name.to_case(Case::Snake), kind)
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

pub fn make_i11n_ctxs(idl: &IDL) -> String {
    format!(
        "pub mod i11n {{
    use anchor_lang::prelude::*;
    use anchor_i11n::prelude::*;
    use anchor_lang::Discriminator;
    use super::{{instructions::*, ID}};
{}

    //Accounts
{}

    //Helper
{}
}}",
        make_i11n_structs(idl),
        make_i11n_accounts(idl),
        make_i11n_decode_helper(idl),
    )
}

fn make_i11n_structs(idl: &IDL) -> String {
    idl.instructions
        .iter()
        .map(|ix| {
            let ix_name_pascal = ix.name.to_case(Case::Pascal);
            format!(
                "
    // {}
    #[derive(TryFromInstruction, serde::Serialize)]
    pub struct {}I11n<'info> {{
        pub accounts: {}AccountMetas{},
        pub args: {},
        pub remaining_accounts: Vec<&'info AccountMeta>,
    }}",
                ix_name_pascal,
                ix_name_pascal,
                ix_name_pascal,
                make_ix_has_info(ix),
                ix_name_pascal
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn make_i11n_decode_helper(idl: &IDL) -> String {
    let IDL {
        instructions, name, ..
    } = idl;
    let idl_name_pascal = name.clone().unwrap().to_case(Case::Pascal);

    let variants = instructions
        .iter()
        .map(|ix| {
            let ix_name = ix.name.clone();
            let ix_name_pascal = ix.name.to_case(Case::Pascal);

            format!(
                "
        #[serde(rename = \"{}\")]
        {}({}I11n<'info>),",
                ix_name, ix_name_pascal, ix_name_pascal
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let variants_ifs = instructions.iter().map(|ix| {
        let ix_name = ix.name.clone();
        let ix_name_pascal = ix.name.to_case(Case::Pascal);
        format!("
            if {}::discriminator().eq(discriminator) {{
                return (\"{}\".to_string(), {}I11n::{}({}I11n::try_from(ix).expect(\"Invalid instruction of {}\")))
            }}
        ", ix_name_pascal, ix_name, idl_name_pascal, ix_name_pascal, ix_name_pascal, ix_name_pascal)
    }).collect::<Vec<String>>().join("\n\t\t\t");

    format!("
    // {}
    #[derive(serde::Serialize)]
    pub enum {}I11n<'info> {{
        {}

        #[serde(rename = \"unknown\")]
        Unknown(String),
    }}




    //helper
    pub fn introspect(ix: &anchor_lang::solana_program::instruction::Instruction) -> (String, {}I11n) {{
        let disc_data: &[u8] = &ix.data.as_slice()[..8];
        let discriminator: &[u8] = disc_data;

        {}

        (\"unknown\".to_string(), {}I11n::Unknown(\"Failed to resolve command\".to_string()))
    }}
    ", idl_name_pascal, idl_name_pascal, variants, idl_name_pascal, variants_ifs, idl_name_pascal)
}
