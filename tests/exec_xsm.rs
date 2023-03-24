use std::process::{Command, Stdio};

#[test]
fn exec_xsm() -> Result<(), ()> {
    log::trace!("Running test exec_xsm...");

    let mut output = Command::new("zsh")
        .arg("xsm_expl/test")
        .arg("/src/input.xsm")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|msg| log::error!("{}", msg))?;

    let _ = output.wait();
    Ok(())
}
