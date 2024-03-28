use std::{
    env, fs,
    io::{self, Write},
};

fn main() -> io::Result<()> {
    if let Some(filename) = env::args().nth(1) {
        let script = fs::read_to_string(&filename)?;

        let program: js::bytecode::Program = script.parse().unwrap();

        println!("{program:#?}");

        let mut vm = js::bytecode::Vm::default();
        vm.execute_program(&program);
        vm.dump();
    } else {
        run_shell()?;
    }

    Ok(())
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
