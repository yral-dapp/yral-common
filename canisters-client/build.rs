use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsStr,
    fs,
    io::BufReader,
    path::PathBuf,
    sync::LazyLock,
};

use anyhow::Result;
use candid_parser::Principal;
use convert_case::{Case, Casing};
use serde::Deserialize;

static DID_WHITELIST: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    #[allow(unused_mut)]
    let mut whitelist = HashSet::new();

    #[cfg(feature = "individual-user")]
    whitelist.insert("individual_user_template");
    #[cfg(feature = "platform-orchestrator")]
    whitelist.insert("platform_orchestrator");
    #[cfg(feature = "post-cache")]
    whitelist.insert("post_cache");
    #[cfg(feature = "user-index")]
    whitelist.insert("user_index");

    #[cfg(feature = "sns-governance")]
    whitelist.insert("sns_governance");
    #[cfg(feature = "sns-ledger")]
    whitelist.insert("sns_ledger");
    #[cfg(feature = "sns-root")]
    whitelist.insert("sns_root");
    #[cfg(feature = "sns-swap")]
    whitelist.insert("sns_swap");
    #[cfg(feature = "sns-index")]
    whitelist.insert("sns_index");

    whitelist
});

#[derive(Deserialize)]
struct CanId {
    ic: Principal,
    local: Principal,
}

fn read_candid_ids() -> Result<HashMap<String, CanId>> {
    let can_ids_file = fs::File::open("did/canister_ids.json")?;
    let reader = BufReader::new(can_ids_file);
    Ok(serde_json::from_reader(reader)?)
}

fn generate_canister_id_mod(can_ids: Vec<(String, Principal)>) -> String {
    let mut canister_id_mod = String::new();
    for (canister, can_id) in can_ids {
        let can_upper = canister.to_case(Case::UpperSnake);
        // CANISTER_NAME_ID: Principal = Principal::from_slice(&[..]);
        canister_id_mod.push_str(&format!(
            "pub const {can_upper}_ID: candid::Principal = candid::Principal::from_slice(&{:?});\n",
            can_id.as_slice()
        ));
    }
    canister_id_mod
}

fn build_canister_ids(out_dir: &str) -> Result<()> {
    let can_ids = read_candid_ids()?;
    let mut local_can_ids = Vec::<(String, Principal)>::new();
    let mut ic_can_ids = Vec::<(String, Principal)>::new();
    let whitelist = DID_WHITELIST.clone();
    for (canister, can_id) in can_ids {
        if !whitelist.contains(canister.as_str()) {
            continue;
        }

        local_can_ids.push((canister.clone(), can_id.local));
        ic_can_ids.push((canister, can_id.ic));
    }

    let local_canister_id_mod = generate_canister_id_mod(local_can_ids);
    let ic_canister_id_mod = generate_canister_id_mod(ic_can_ids);

    let canister_id_mod_contents = format!(
        r#"
    pub mod local {{
        {local_canister_id_mod}
    }}

    pub mod ic {{
        {ic_canister_id_mod}
    }}
"#
    );
    let canister_id_mod_path = PathBuf::from(out_dir).join("canister_ids.rs");
    fs::write(canister_id_mod_path, canister_id_mod_contents)?;

    Ok(())
}

fn build_did_intfs(out_dir: &str) -> Result<()> {
    println!("cargo:rerurn-if-changed=./did/*");

    let mut candid_config = candid_parser::bindings::rust::Config::new();
    candid_config.set_target(candid_parser::bindings::rust::Target::Agent);
    candid_config
        .set_type_attributes("#[derive(CandidType, Deserialize, Debug, PartialEq)]".into());
    let mut did_mod_contents = String::new();
    let whitelist = DID_WHITELIST.clone();

    // create $OUT_DIR/did directory
    let did_dir = PathBuf::from(&out_dir).join("did");
    fs::create_dir_all(&did_dir)?;

    // Auto generate bindings for did files
    for didinfo in fs::read_dir("did")? {
        let didpath = didinfo?.path();
        if didpath.extension() != Some(OsStr::new("did")) {
            continue;
        }
        let file_name = didpath.file_stem().unwrap().to_str().unwrap();
        if !whitelist.contains(file_name) {
            continue;
        }

        // compile bindings from did
        let service_name: String = file_name.to_case(Case::Pascal);
        candid_config.set_service_name(service_name);
        let (type_env, actor) = candid_parser::check_file(&didpath).unwrap_or_else(|e| {
            panic!(
                "invalid did file: {}, err: {e}",
                didpath.as_os_str().to_string_lossy()
            )
        });
        let bindings = candid_parser::bindings::rust::compile(&candid_config, &type_env, &actor);

        // write bindings to $OUT_DIR/did/<did file>.rs
        let mut binding_file = did_dir.clone();
        binding_file.push(file_name);
        binding_file.set_extension("rs");
        fs::write(&binding_file, bindings)?;

        // #[path = "$OUT_DIR/did/<did file>.rs"] pub mod <did file>;
        did_mod_contents.push_str(&format!(
            "#[path = \"{}\"] pub mod {};\n",
            binding_file.to_string_lossy(),
            file_name
        ));
    }

    // create mod file for did bindings
    let binding_mod_file = PathBuf::from(&out_dir).join("did").join("mod.rs");
    fs::write(binding_mod_file, did_mod_contents)?;

    Ok(())
}

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();

    build_did_intfs(&out_dir)?;
    build_canister_ids(&out_dir)?;

    Ok(())
}
