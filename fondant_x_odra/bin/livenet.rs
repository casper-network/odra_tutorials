use fondant_x_odra::flipper::FlipperHostRef;
use odra::args::Maybe;
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader, NoArgs};
use odra::Address;
use std::str::FromStr;

fn main() {
    let env = odra_casper_livenet_env::env();

    // Deploy new contract.
    let mut flipper = deploy_contract(&env);
    println!("flipper current value: {}", flipper.get().to_string());

    // Uncomment to load existing contract.
    // let mut token = load_contract(&env, CASPER_CONTRACT_ADDRESS);
    // println!("Token name: {}", token.get_collection_name());

    env.set_gas(3_000_000_000u64);
    let owner = env.caller();
    let _ = flipper.flip();
    println!("flipper after flip value: {}", flipper.get().to_string());
}

pub fn load_contract(env: &HostEnv, address: &str) -> FlipperHostRef {
    let address = Address::from_str(address).expect("Should be a valid contract address");
    FlipperHostRef::load(env, address)
}

pub fn deploy_contract(env: &HostEnv) -> FlipperHostRef {
    env.set_gas(400_000_000_000u64);
    FlipperHostRef::deploy(env, NoArgs)
}
