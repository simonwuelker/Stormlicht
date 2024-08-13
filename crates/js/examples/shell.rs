use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let Some(filename) = env::args().nth(1) else {
        eprintln!("No filename provided");
        return ExitCode::FAILURE;
    };

    let Ok(script) = fs::read_to_string(&filename) else {
        return ExitCode::FAILURE;
    };

    let executable = match script.parse::<js::Executable>() {
        Ok(executable) => executable,
        Err(error) => {
            eprintln!("Failed to parse program {error:?}");
            return ExitCode::FAILURE;
        },
    };

    println!("{executable:#?}");

    let mut vm = js::Vm::default();
    vm.execute(executable);
    println!("{vm:#?}");

    ExitCode::SUCCESS
}
