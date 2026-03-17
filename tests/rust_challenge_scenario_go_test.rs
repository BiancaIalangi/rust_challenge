use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn rust_challenge_go() {
    world().run("scenarios/rust_challenge.scen.json");
}
