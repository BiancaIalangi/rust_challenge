use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/rust_challenge.mxsc.json", rust_challenge::ContractBuilder);
    blockchain
}

#[test]
fn rust_challenge_rs() {
    world().run("scenarios/rust_challenge.scen.json");
}
