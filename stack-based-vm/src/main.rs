use std::collections::HashMap;
use std::fs::read_to_string;
use std::env;
use std::process;

struct VMState {
    ip: usize,
    reg: i32,
    stack: Vec<i32>,
}

impl VMState {
    fn new() -> Self {
        Self {
            ip : 0,
            reg : 0,
            stack : Vec::new()
        }
    }
}

fn ex_print(_: Option<&&str>, vm_state: &mut VMState) {
    println!("{}", vm_state.reg);
    vm_state.ip += 1;
}
fn ex_push(_: Option<&&str>, vm_state: &mut VMState) {
    vm_state.stack.push(vm_state.reg);
    vm_state.ip += 1;
}
fn ex_pop(_: Option<&&str>, vm_state: &mut VMState) {
    if let Some(value) = vm_state.stack.pop() {
        vm_state.reg = value;
        vm_state.ip += 1;
    } else {
        panic!(format!(
            "[POP:{}] Tried to pop from empty stack!",
            vm_state.ip
        ));
    }
}
fn ex_top(_: Option<&&str>, vm_state: &mut VMState) {
    if let Some(value) = vm_state.stack.get(0) {
        vm_state.reg = *value;
        vm_state.ip += 1;
    } else {
        panic!(format!(
            "[TOP:{}] Tried to peek empty stack!",
            vm_state.ip
        ));
    }
}
fn ex_add(_: Option<&&str>, vm_state: &mut VMState) {
    if let Some(value) = vm_state.stack.pop() {
        vm_state.reg += value;
        vm_state.ip += 1;
    } else {
        panic!(format!(
            "[ADD:{}] Tried to pop from empty stack!",
            vm_state.ip
        ));
    }
}
fn ex_sub(_: Option<&&str>, vm_state: &mut VMState) {
    if let Some(value) = vm_state.stack.pop() {
        vm_state.reg = value - vm_state.reg;
        vm_state.ip += 1;
    } else {
        panic!(format!(
            "[SUB:{}] Tried to pop from empty stack!",
            vm_state.ip
        ));
    }
}
fn ex_div(_: Option<&&str>, vm_state: &mut VMState) {
    if let Some(value) = vm_state.stack.pop() {
        vm_state.reg = value / vm_state.reg;
        vm_state.ip += 1;
    } else {
        panic!(format!(
            "[DIV:{}] Tried to pop from empty stack!",
            vm_state.ip
        ));
    }
}
fn ex_mul(_: Option<&&str>, vm_state: &mut VMState) {
    if let Some(value) = vm_state.stack.pop() {
        vm_state.reg *= value;
        vm_state.ip += 1;
    } else {
        panic!(format!(
            "[MUL:{}] Tried to pop from empty stack!",
            vm_state.ip
        ));
    }
}
fn ex_load(arg: Option<&&str>, vm_state: &mut VMState) {
    if let Some(arg) = arg {
        vm_state.stack.push(
            arg.parse()
                .expect(&format!("[LOAD:{}] Could not parse '{}'", vm_state.ip, arg)),
        );
        vm_state.ip += 1;
    } else {
        panic!(format!("[LOAD:{}] Requires one int argument!", vm_state.ip));
    }
}
fn ex_jmp(arg: Option<&&str>, vm_state: &mut VMState) {
    if let Some(arg) = arg {
        vm_state.ip = arg
            .parse()
            .expect(&format!("[JMP:{}] Could not parse '{}'", vm_state.ip, arg));
    } else {
        panic!(format!("[JMP:{}] Requires one uint argument!", vm_state.ip));
    }
}
fn ex_jz(arg: Option<&&str>, vm_state: &mut VMState) {
    if let Some(arg) = arg {
        let n_ip = arg
            .parse()
            .expect(&format!("[JZ:{}] Could not parse '{}'", vm_state.ip, arg));
        if vm_state.reg == 0 {
            vm_state.ip = n_ip;
        } else {
            vm_state.ip += 1;
        }
    } else {
        panic!(format!("[JZ:{}] Requires one int argument!", vm_state.ip));
    }
}
fn ex_jnz(arg: Option<&&str>, vm_state: &mut VMState) {
    if let Some(arg) = arg {
        let n_ip = arg
            .parse()
            .expect(&format!("[JNZ:{}] Could not parse '{}'", vm_state.ip, arg));
        if vm_state.reg != 0 {
            vm_state.ip = n_ip;
        } else {
            vm_state.ip += 1;
        }
    } else {
        panic!(format!("[JNZ:{}] Requires one int argument!", vm_state.ip));
    }
}


fn build_commands() -> HashMap<String, fn(Option<&&str>, &mut VMState)> {
    let mut commands: HashMap<String, fn(Option<&&str>, &mut VMState)> = HashMap::new();
    commands.insert("PUSH".to_string(), ex_push);
    commands.insert("POP".to_string(), ex_pop);
    commands.insert("TOP".to_string(), ex_top);
    commands.insert("ADD".to_string(), ex_add);
    commands.insert("SUB".to_string(), ex_sub);
    commands.insert("DIV".to_string(), ex_div);
    commands.insert("MUL".to_string(), ex_mul);
    commands.insert("LOAD".to_string(), ex_load);
    commands.insert("JMP".to_string(), ex_jmp);
    commands.insert("JZ".to_string(), ex_jz);
    commands.insert("JNZ".to_string(), ex_jnz);
    commands.insert("PRINT".to_string(), ex_print);

    commands
}

fn read_program(file_name : &str) -> Vec<String> {
    let content = read_to_string(file_name).expect("Could not open the specified file!");
    content.trim().split("\n").map(|x| x.to_ascii_uppercase()).collect()
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Require program file! Please specify a file.");
        process::exit(1);
    }

    let commands = build_commands();
    let program = read_program(&args[1]);

    let mut vm_state = VMState::new();
    while vm_state.ip < program.len() {
        let instruction : Vec<&str> = program[vm_state.ip].split_ascii_whitespace().collect();
        let arg = instruction.get(1);
        if let Some(cmd) = commands.get(&instruction[0].to_string()) {
            cmd(arg, &mut vm_state);
        } else {
            panic!(format!("Invalid instruction at line {}: '{}'", vm_state.ip, instruction[0]));
        }
    }
}
