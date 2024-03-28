use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    if let Some(filename) = env::args().nth(1) {
        let Ok(script) = fs::read_to_string(&filename) else {
            return ExitCode::FAILURE;
        };

        let program = match script.parse::<js::bytecode::Program>() {
            Ok(program) => program,
            Err(error) => {
                error.get_context(&script).dump();
                return ExitCode::FAILURE;
            },
        };

        println!("{program:#?}");

        let mut vm = js::bytecode::Vm::default();
        vm.execute_program(&program);
        vm.dump();

        ExitCode::SUCCESS
    } else {
        match run_shell() {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        }
    }
}

fn run_shell() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut vm = js::bytecode::Vm::default();
    loop {
        buffer.clear();
        let mut stdout = io::stdout();
        write!(stdout, ">>> ")?;
        stdout.flush()?;

        stdin.read_line(&mut buffer)?;

        match buffer.parse::<js::bytecode::Program>() {
            Ok(program) => {
                writeln!(stdout, "{program:#?}")?;

                vm.execute_program(&program);
                vm.dump();
            },
            Err(error) => error.get_context(&buffer).dump(),
        }
    }
}
