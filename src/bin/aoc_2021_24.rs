#![allow(dead_code)]

// Make the 4 register names globally available.
use Register::*;

fn main() {
    let input_program = &advent_of_code::env::get_puzzle_input_path("2021-12-24.txt");
    let program = parse_program(std::fs::read_to_string(&input_program).unwrap().lines());

    println!("Converting program to SSA form.");
    let (sym_prog, _reg_states) = execute_symbolic(&program, Z);
    assert_eq!(program.num_inputs(), sym_prog.num_inputs());

    // By printing the optimized program, and then setting pairs of values as below iteratively to
    // simplify the resulting code, we obtain the highest model number:
    //
    // Set 4 and 5 as high as possible while respecting in[4]-2 == in[5]
    let sym_prog = sym_prog.substitute_input_value(4, 9);
    let sym_prog = sym_prog.substitute_input_value(5, 7);
    // Set 3 and 6 as high as possible while respecting in[3] + 7 == in[6]
    let sym_prog = sym_prog.substitute_input_value(3, 2);
    let sym_prog = sym_prog.substitute_input_value(6, 9);
    // Set 7 and 8 as high as possible while respecting in[7] + 14 -10 == in[8]
    let sym_prog = sym_prog.substitute_input_value(7, 5);
    let sym_prog = sym_prog.substitute_input_value(8, 9);
    // Set 9 and 10 as high as possible while respecting in[9] + 6 - 12 == in[10]
    let sym_prog = sym_prog.substitute_input_value(9, 9);
    let sym_prog = sym_prog.substitute_input_value(10, 3);
    // Set 2 and 11 as high as possible while respecting in[2] + 8 - 3 == in[11]
    let sym_prog = sym_prog.substitute_input_value(2, 4);
    let sym_prog = sym_prog.substitute_input_value(11, 9);
    // Set 1 and 12 as high as possible while respecting in[1] + 4 - 11 == in[12]
    let sym_prog = sym_prog.substitute_input_value(1, 9);
    let sym_prog = sym_prog.substitute_input_value(12, 2);
    // Set 0 and 13 as high as possible while respecting in[0] + 2 - 2 == in[13]
    let sym_prog = sym_prog.substitute_input_value(0, 9);
    let sym_prog = sym_prog.substitute_input_value(13, 9);
    // Model number: 99429795993929

    // Similarly, looking for the lowest:
    // Model number: 18113181571611

    println!("Optimizing...");

    let opt_pass = |name: &str,
                    sym_prog: &SymbolicProgram,
                    opt_pass: &dyn Fn(&SymbolicProgram) -> SymbolicProgram| {
        let before_vars = sym_prog.num_vars();
        let before_consts = sym_prog.num_constants();
        let result = opt_pass(sym_prog);
        let after_vars = result.num_vars();
        let after_consts = result.num_constants();
        println!(
            "[{:>20}] variables: {} -> {}, constants: {} -> {}",
            name, before_vars, after_vars, before_consts, after_consts
        );
        result
    };

    let mut optimized_prog = sym_prog.clone();

    loop {
        let opt = opt_pass(
            "constant folding",
            &optimized_prog,
            &optimize_constant_results,
        );
        let opt = opt_pass("value ranges", &opt, &optimize_based_on_input_value_range);
        let opt = opt_pass("common subexpr", &opt, &optimize_common_subexpressions);
        let opt = opt_pass("prune dead code", &opt, &optimize_prune_dead_code);
        let opt = opt_pass("modulos", &opt, &optimize_modulos);
        let opt = opt_pass("divisions", &opt, &optimize_divisions);

        if opt == optimized_prog {
            break;
        }
        optimized_prog = opt;
    }

    // println!("Optimized program:\n{}", optimized_prog);

    display_what_inputs_affect_what_vars(&optimized_prog);

    fn batch_eval(
        mut model: ModelNumber,
        num_models: usize,
        sym_prog: &SymbolicProgram,
    ) {
        let mut num_valid = 0;
        let mut num_tested = 0;
        let mut var_values = vec![0i64; sym_prog.num_vars()];
        for _ in 0..num_models {
            for (var_id, expr) in sym_prog.vars.iter().enumerate() {
                let val = match expr {
                    SymbolicExpr::Int(x) => *x,
                    SymbolicExpr::Input(i) => model.0[*i] as i64,
                    SymbolicExpr::Op(binop) => {
                        let a = var_values[binop.a];
                        let b = var_values[binop.b];
                        binop.op.apply(a, b).unwrap()
                    }
                };
                var_values[var_id] = val;
            }

            let z_val = *var_values.last().unwrap();
            if z_val == 0 {
                num_valid += 1;
            }
            num_tested += 1;

            if !model.increment() {
                println!("Reached the final model number, stopping.");
                break;
            }
        }

        println!("Found {} / {} valid model numbers", num_valid, num_tested);
    }
    let model = ModelNumber([0;14]);
    batch_eval(model, 1, &optimized_prog);

    // TODOs:
    // - Optimization passes over the symbolic ops:
    //      - Discard MOD ops after EQL ops.
    //      - Look for more examples.
    // - Display SymbolicProgram better: show 2 levels of ops.
    // - Start from the result (Z), set it to 0 (good model number), and work backwards.
    // - For each value 1..=9 of the highest digit: is there any way to get a result of 0?
    //      - Once the highest such digit is found, try for the next less significant digit.
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Program {
    instructions: Vec<Instruction>,
}

/// A single instruction in a program.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Instruction {
    Input(Register),
    Op(BinaryOp),
}

/// A binary operation involving the source & destination register a, and the second operand b
/// (either a literal or a register).
#[derive(Clone, Debug, Eq, PartialEq)]
struct BinaryOp {
    op: Op,
    a: Register,
    b: Value,
}

/// The 5 types of binary operations supported by the ALU.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Op {
    Add,
    Mul,
    Div,
    Mod,
    Eql,
}

/// A register in the ALU.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Register {
    W = 0,
    X = 1,
    Y = 2,
    Z = 3,
}

/// A part of an operation, either a literal integer, or the contents of a register.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Value {
    Int(i64),
    Reg(Register),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegisterState([i64; 4]);

impl RegisterState {
    fn new() -> Self {
        Self([0; 4])
    }

    fn get(&self, reg: Register) -> i64 {
        self.0[reg as usize]
    }

    fn set(&mut self, reg: Register, value: i64) {
        self.0[reg as usize] = value;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ExecutionError {
    InvalidInput,
    DivideByZero,
    NegativeModulo,
}

impl Program {
    fn execute<T>(&self, mut inputs: &[T], result_reg: Register) -> Result<i64, ExecutionError>
    where
        T: Copy + Into<i64>,
    {
        if inputs.len() != self.num_inputs() {
            return Err(ExecutionError::InvalidInput);
        }

        let mut state = RegisterState::new();

        for instr in &self.instructions {
            match instr {
                Instruction::Input(reg) => {
                    let (x, rest) = inputs.split_first().unwrap();
                    state.set(*reg, (*x).into());
                    inputs = rest;
                }
                Instruction::Op(binop) => {
                    let a = state.get(binop.a);
                    let b: i64 = match binop.b {
                        Value::Int(x) => x,
                        Value::Reg(reg) => state.get(reg),
                    };
                    state.set(binop.a, binop.op.apply(a, b)?);
                }
            }
        }
        Ok(state.get(result_reg))
    }

    fn execute_with_logging(
        &self,
        mut inputs: &[i64],
    ) -> Result<Vec<RegisterState>, ExecutionError> {
        if inputs.len() != self.num_inputs() {
            return Err(ExecutionError::InvalidInput);
        }

        let mut state = RegisterState::new();
        let mut all_states = vec![state.clone()];

        for instr in &self.instructions {
            match instr {
                Instruction::Input(reg) => {
                    let (x, rest) = inputs.split_first().unwrap();
                    state.set(*reg, *x);
                    inputs = rest;
                }
                Instruction::Op(binop) => {
                    let a = state.get(binop.a);
                    let b: i64 = match binop.b {
                        Value::Int(x) => x,
                        Value::Reg(reg) => state.get(reg),
                    };
                    state.set(binop.a, binop.op.apply(a, b)?);
                }
            }
            all_states.push(state.clone());
        }

        Ok(all_states)
    }

    /// Returns the number of inputs that the program expects.
    fn num_inputs(&self) -> usize {
        self.instructions
            .iter()
            .filter(|i| matches!(i, Instruction::Input(_)))
            .count()
    }
}

impl Register {
    fn from_char(c: char) -> Register {
        match c {
            'w' => W,
            'x' => X,
            'y' => Y,
            'z' => Z,
            _ => panic!("Invalid register: {}", c),
        }
    }
}

impl Op {
    fn apply(self, lhs: i64, rhs: i64) -> Result<i64, ExecutionError> {
        match self {
            Op::Add => Ok(lhs + rhs),
            Op::Mul => Ok(lhs * rhs),
            Op::Div => {
                if rhs == 0 {
                    Err(ExecutionError::DivideByZero)
                } else {
                    Ok(lhs / rhs)
                }
            }
            Op::Mod => {
                if lhs < 0 || rhs <= 0 {
                    Err(ExecutionError::NegativeModulo)
                } else {
                    Ok(lhs % rhs)
                }
            }
            Op::Eql => {
                if lhs == rhs {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModelNumber([u8; 14]);

impl ModelNumber {
    fn zero() -> Self {
        Self([0; 14])
    }

    fn increment(&mut self) -> bool {
        for digit in self.0.iter_mut().rev() {
            if *digit == 9 {
                *digit = 1;
            } else {
                *digit += 1;
                // Produced a new model number.
                return true;
            }
        }

        // Reached the final model number and looped around.
        false
    }
}

/// A program where all the register assignments are in SSA form.
/// I.e., instead of "add x 2" mutating "x", it creates a new variable v (a virtual register)
/// containing the result, and makes the physical register "x" point to v.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SymbolicProgram {
    /// The list of variables that the original program defines (implicitly).
    /// Variables containing binary operations depend on other variables, which creates a dataflow
    /// graph.
    /// The last variable in the graph contains the value of the output register (Z).
    vars: Vec<SymbolicExpr>,
}

impl SymbolicProgram {
    fn push_var(&mut self, expr: SymbolicExpr) -> VarId {
        let var_id = self.vars.len();
        self.vars.push(expr);
        var_id
    }

    /// Returns the number of inputs that the program expects.
    fn num_inputs(&self) -> usize {
        let set: std::collections::HashSet<usize> = self
            .vars
            .iter()
            .filter_map(|v| {
                if let SymbolicExpr::Input(input_idx) = v {
                    Some(*input_idx)
                } else {
                    None
                }
            })
            .collect();
        set.len()
    }

    fn num_vars(&self) -> usize {
        self.vars.len()
    }

    fn num_constants(&self) -> usize {
        self.vars
            .iter()
            .filter(|expr| matches!(expr, SymbolicExpr::Int(_)))
            .count()
    }

    fn substitute_input_value(&self, input_idx: usize, value: i64) -> Self {
        SymbolicProgram {
            vars: self
                .vars
                .iter()
                .map(|expr| match expr {
                    SymbolicExpr::Input(x) if *x == input_idx => SymbolicExpr::Int(value),
                    _ => expr.clone(),
                })
                .collect(),
        }
    }
}

/// A symbolic program variable, identified by its index in the `vars` vec.
type VarId = usize;

#[derive(Clone, Debug, Eq, PartialEq)]
enum SymbolicExpr {
    Int(i64),
    Input(usize),
    Op(SymbolicBinaryOp),
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SymbolicBinaryOp {
    op: Op,
    a: VarId,
    b: VarId,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct SymbolicRegisterState([VarId; 4]);

impl SymbolicRegisterState {
    fn get(&self, reg: Register) -> VarId {
        self.0[reg as usize]
    }

    fn set(&mut self, reg: Register, var: VarId) {
        self.0[reg as usize] = var;
    }
}

/// Describes the range of values that a particular variable may take.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ValueRange {
    /// No information available.
    Unknown,
    /// A closed range, start..=end.
    RangeIncl(i64, i64),
}

/// Perform symbolic execution of a program, returning it in Static Single Assignment form, as well
/// as the state of the registers after each instructions.
fn execute_symbolic(
    program: &Program,
    result_reg: Register,
) -> (SymbolicProgram, Vec<SymbolicRegisterState>) {
    // Create 4 variables set to 0 for the initial register states.
    let mut sym_prog = SymbolicProgram {
        vars: vec![SymbolicExpr::Int(0); 4],
    };
    // For each register, which variable it currently points to.
    let mut states = vec![SymbolicRegisterState([0, 1, 2, 3])];
    // The next input digit to be read.
    let mut next_input_idx = 0;

    for (_idx, instr) in program.instructions.iter().enumerate() {
        let mut new_state = states.last().unwrap().clone();

        match instr {
            Instruction::Input(reg) => {
                let var_id = sym_prog.push_var(SymbolicExpr::Input(next_input_idx));
                new_state.set(*reg, var_id);
                next_input_idx += 1;
            }
            Instruction::Op(binop) => {
                let a = new_state.get(binop.a);
                let b = match binop.b {
                    Value::Int(x) => sym_prog.push_var(SymbolicExpr::Int(x)),
                    Value::Reg(reg) => new_state.get(reg),
                };
                let result =
                    sym_prog.push_var(SymbolicExpr::Op(SymbolicBinaryOp { op: binop.op, a, b }));
                new_state.set(binop.a, result);
            }
        }

        states.push(new_state);
    }

    // The last assigment in our symbolic program is the one that yields the final state of the
    // result register. Assert that this is already the case (ie there was no extra useless
    // instructions at the end of the program).
    let result_var = states.last().unwrap().get(result_reg);
    assert_eq!(result_var + 1, sym_prog.vars.len());

    (sym_prog, states)
}

/// Optimizes a symbolic program by:
/// - Propagating constants, eg c = op(a, b) when a and b are both known integer values.
/// - Simplifying operations like adding 0, multiplying by 0, modulo 1, etc.
fn optimize_constant_results(prog: &SymbolicProgram) -> SymbolicProgram {
    let mut res = SymbolicProgram { vars: Vec::new() };

    for expr in &prog.vars {
        let new_expr = if let SymbolicExpr::Op(binop) = expr {
            simplify_symbolic_op(prog, binop)
        } else {
            expr.clone()
        };
        res.push_var(new_expr);
    }

    res
}

fn simplify_symbolic_op(prog: &SymbolicProgram, binop: &SymbolicBinaryOp) -> SymbolicExpr {
    let is_scalar = |x: VarId, scalar: i64| -> bool {
        matches!(prog.vars[x], SymbolicExpr::Int(x) if x == scalar)
    };

    let unpack_scalars = |a: VarId, b: VarId| -> Option<(i64, i64)> {
        if let (SymbolicExpr::Int(a), SymbolicExpr::Int(b)) = (&prog.vars[a], &prog.vars[b]) {
            Some((*a, *b))
        } else {
            None
        }
    };

    let a = binop.a;
    let b = binop.b;

    let a_is_zero = is_scalar(a, 0);
    let b_is_zero = is_scalar(b, 0);
    let a_is_one = is_scalar(a, 1);
    let b_is_one = is_scalar(b, 1);

    match binop.op {
        Op::Add => {
            if a_is_zero {
                prog.vars[b].clone()
            } else if b_is_zero {
                prog.vars[a].clone()
            } else if let Some((a, b)) = unpack_scalars(a, b) {
                SymbolicExpr::Int(a + b)
            } else {
                SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Add, a, b })
            }
        }
        Op::Mul => {
            if a_is_zero || b_is_zero {
                SymbolicExpr::Int(0)
            } else if a_is_one {
                prog.vars[b].clone()
            } else if b_is_one {
                prog.vars[a].clone()
            } else if let Some((a, b)) = unpack_scalars(a, b) {
                SymbolicExpr::Int(a * b)
            } else {
                SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Mul, a, b })
            }
        }
        Op::Div => {
            if a_is_zero {
                prog.vars[a].clone()
            } else if b_is_one {
                prog.vars[a].clone()
            } else if let Some((a, b)) = unpack_scalars(a, b) {
                SymbolicExpr::Int(a / b)
            } else {
                SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Div, a, b })
            }
        }
        Op::Mod => {
            if a_is_zero {
                prog.vars[a].clone()
            } else if b_is_one {
                SymbolicExpr::Int(0)
            } else if let Some((a, b)) = unpack_scalars(a, b) {
                SymbolicExpr::Int(a % b)
            } else {
                SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Mod, a, b })
            }
        }
        Op::Eql => {
            if let Some((a, b)) = unpack_scalars(a, b) {
                SymbolicExpr::Int((a == b) as i64)
            } else if a == b {
                SymbolicExpr::Int(1)
            } else {
                SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Eql, a, b })
            }
        }
    }
}

/// Starting from a result variable `result_var`, backtracks to identify live and dead
/// code, and removes the dead parts.
/// Returns:
/// - A new var id corresponding to the original `result_var`.
/// - A list of intermediate variables that contribute to this final result.
fn optimize_prune_dead_code(prog: &SymbolicProgram) -> SymbolicProgram {
    assert!(!prog.vars.is_empty());

    let result_var = prog.vars.len() - 1;
    let mut used_vars = vec![false; prog.vars.len()];

    let mut to_visit = Vec::new();
    // First variable that is used is whichever lives in the result register.
    to_visit.push(result_var);

    while let Some(var_id) = to_visit.pop() {
        used_vars[var_id] = true;

        match &prog.vars[var_id] {
            SymbolicExpr::Op(binop) => {
                to_visit.push(binop.a);
                to_visit.push(binop.b);
            }
            SymbolicExpr::Int(_) | SymbolicExpr::Input(_) => {}
        }
    }

    // A vector mapping new var ids to old ones.
    let new_to_old: Vec<usize> = used_vars
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(var_id, is_used)| if is_used { Some(var_id) } else { None })
        .collect();

    // A vector mapping old var id to Some(new id).
    let mut old_to_new = vec![None; prog.vars.len()];
    for (new_id, old_id) in new_to_old.iter().copied().enumerate() {
        old_to_new[old_id] = Some(new_id);
    }

    let new_vars: Vec<SymbolicExpr> = prog
        .vars
        .iter()
        .zip(used_vars)
        .filter_map(|(expr, is_used)| {
            if !is_used {
                None
            } else {
                let new_expr = match expr {
                    SymbolicExpr::Int(_) => expr.clone(),
                    SymbolicExpr::Input(_) => expr.clone(),
                    SymbolicExpr::Op(binop) => SymbolicExpr::Op(SymbolicBinaryOp {
                        op: binop.op,
                        a: old_to_new[binop.a].unwrap(),
                        b: old_to_new[binop.b].unwrap(),
                    }),
                };
                Some(new_expr)
            }
        })
        .collect();

    assert_eq!(old_to_new[result_var].unwrap(), new_vars.len() - 1);

    SymbolicProgram { vars: new_vars }
}

fn optimize_based_on_input_value_range(prog: &SymbolicProgram) -> SymbolicProgram {
    // For each variable, the corresponding ValueRange.
    let mut ranges: Vec<ValueRange> = Vec::new();

    use ValueRange::*;
    for (_i, var) in prog.vars.iter().enumerate() {
        match var {
            SymbolicExpr::Int(x) => ranges.push(RangeIncl(*x, *x)),
            // Inputs are digits in 1-9.
            SymbolicExpr::Input(_) => ranges.push(RangeIncl(1, 9)),
            SymbolicExpr::Op(binop) => {
                let range_a = &ranges[binop.a];
                let range_b = &ranges[binop.b];

                let result_range = match (binop.op, range_a, range_b) {
                    // Add
                    (Op::Add, Unknown, _) | (Op::Add, _, Unknown) => Unknown,
                    (Op::Add, RangeIncl(sa, ea), RangeIncl(sb, eb)) => RangeIncl(sa + sb, ea + eb),
                    // Mul
                    (Op::Mul, Unknown, _) | (Op::Mul, _, Unknown) => Unknown,
                    (Op::Mul, RangeIncl(sa, ea), RangeIncl(sb, eb)) => RangeIncl(
                        i64::min(i64::min(sa * sb, sa * eb), i64::min(ea * sb, ea * eb)),
                        i64::max(i64::max(sa * sb, sa * eb), i64::max(ea * sb, ea * eb)),
                    ),
                    // Div
                    // LHS is positive, RHS is always larger than LHS => result is 0.
                    (Op::Div, RangeIncl(sa, ea), RangeIncl(sb, _)) if *sa >= 0 && ea < sb => {
                        RangeIncl(0, 0)
                    }
                    (Op::Div, _, _) => Unknown,
                    // Mod
                    (Op::Mod, _, RangeIncl(_, e)) if *e <= 0 => {
                        panic!("Modulo operation is guaranteed to panic. Bug?")
                    }
                    (Op::Mod, RangeIncl(_, ea), RangeIncl(_, eb)) => {
                        RangeIncl(0, i64::min((*ea).max(0), *eb - 1))
                    }
                    (Op::Mod, _, RangeIncl(_, e)) => RangeIncl(0, e - 1),
                    (Op::Mod, _, Unknown) => Unknown,
                    // Eql
                    // No overlap between the ranges => never equal.
                    (Op::Eql, RangeIncl(sa, ea), RangeIncl(sb, eb)) if (ea < sb) || (eb < sa) => {
                        RangeIncl(0, 0)
                    }
                    // 2 size-1 ranges that are equal => always equal.
                    (Op::Eql, RangeIncl(sa, ea), RangeIncl(sb, eb))
                        if (sa == ea) && (ea == sb) && (sb == eb) =>
                    {
                        RangeIncl(1, 1)
                    }
                    // General case: 0 or 1.
                    (Op::Eql, _, _) => RangeIncl(0, 1),
                };

                ranges.push(result_range);
            }
        }
    }
    assert_eq!(ranges.len(), prog.vars.len());

    // Iterate over the vars, replacing the ones that have exactly known values by constants.
    let new_vars = prog
        .vars
        .iter()
        .zip(ranges.iter())
        .map(|(var, range)| {
            // Integers and inputs don't need to be optimized.
            if !matches!(var, SymbolicExpr::Op(_)) {
                return var.clone();
            }
            let (s, e) = match range {
                Unknown => return var.clone(),
                RangeIncl(s, e) => (s, e),
            };
            if s == e {
                SymbolicExpr::Int(*s)
            } else {
                var.clone()
            }
        })
        .collect();

    SymbolicProgram { vars: new_vars }
}

fn optimize_common_subexpressions(prog: &SymbolicProgram) -> SymbolicProgram {
    let mut res = SymbolicProgram { vars: Vec::new() };

    let mut bin_op_to_old_id: std::collections::HashMap<&SymbolicBinaryOp, VarId> =
        Default::default();
    let mut old_id_to_new_id = vec![None; prog.num_vars()];

    for (old_id, expr) in prog.vars.iter().enumerate() {
        let new_id = if let SymbolicExpr::Op(binop) = expr {
            if let Some(other_old_id) = bin_op_to_old_id.get(binop) {
                // We have already seen an identical binary op. Just reuse the corresponding
                // new_id.
                old_id_to_new_id[*other_old_id].unwrap()
            } else {
                // First time seeing this binary op. Record the op, and create a new `new_id`.
                bin_op_to_old_id.insert(binop, old_id);
                res.push_var(SymbolicExpr::Op(SymbolicBinaryOp {
                    op: binop.op,
                    a: old_id_to_new_id[binop.a].unwrap(),
                    b: old_id_to_new_id[binop.b].unwrap(),
                }))
            }
        } else {
            res.push_var(expr.clone())
        };

        old_id_to_new_id[old_id] = Some(new_id);
    }

    res
}

/// Implements a very specific optimization for the puzzle input:
/// (a * 26 + b) % 26   =>   b % 26
fn optimize_modulos(prog: &SymbolicProgram) -> SymbolicProgram {
    let mut res = prog.clone();

    let is_mul_by_x = |var_id: VarId, x: i64| -> bool {
        match prog.vars[var_id] {
            SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Mul, a, b }) => {
                (prog.vars[a] == SymbolicExpr::Int(x)) || (prog.vars[b] == SymbolicExpr::Int(x))
            }
            _ => false,
        }
    };

    for (i, expr) in prog.vars.iter().enumerate() {
        // Check if expr is of the form:
        //   expr = (add_a + add_b) % Int(mod_by)
        //        = ((x * Int(mod_by)) + add_b) % Int(mod_by)

        // Only look at binary ops.
        let binop = if let SymbolicExpr::Op(binop) = expr {
            binop
        } else {
            continue;
        };

        // Where the op is modulo.
        if binop.op != Op::Mod {
            continue;
        }
        let lhs = &prog.vars[binop.a];
        let rhs = &prog.vars[binop.b];

        // Where the rhs is a constant.
        let mod_by = if let SymbolicExpr::Int(m) = rhs {
            *m
        } else {
            continue;
        };

        // And the lhs is an addition.
        let (add_a, add_b) = if let SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Add, a, b }) = lhs {
            (*a, *b)
        } else {
            continue;
        };

        // The addition lhs is a multiplication by `mod_by`.
        if is_mul_by_x(add_a, mod_by) {
            res.vars[i] = SymbolicExpr::Op(SymbolicBinaryOp {
                op: Op::Mod,
                a: add_b,
                b: binop.b,
            });
        }
    }

    res
}

/// Implements a very specific optimization for the puzzle input:
/// (a * 26 + b) / 26 => a + b / 26
fn optimize_divisions(prog: &SymbolicProgram) -> SymbolicProgram {
    let mut res = SymbolicProgram { vars: Vec::new() };
    let mut old_id_to_new_id = vec![0; prog.num_vars()];

    let push_new_op = |new_prog: &mut SymbolicProgram,
                       old_id_to_new_id: &mut [VarId],
                       old_id: VarId,
                       old_expr: &SymbolicExpr| {
        let new_expr = match old_expr {
            SymbolicExpr::Op(binop) => SymbolicExpr::Op(SymbolicBinaryOp {
                op: binop.op,
                a: old_id_to_new_id[binop.a],
                b: old_id_to_new_id[binop.b],
            }),
            _ => old_expr.clone(),
        };
        let new_id = new_prog.push_var(new_expr);
        old_id_to_new_id[old_id] = new_id;
    };

    let is_mul_by_x = |var_id: VarId, x: i64| -> Option<VarId> {
        match prog.vars[var_id] {
            SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Mul, a, b }) => {
                if prog.vars[a] == SymbolicExpr::Int(x) {
                    Some(b)
                } else if prog.vars[b] == SymbolicExpr::Int(x) {
                    Some(a)
                } else {
                    None
                }
            }
            _ => None,
        }
    };

    for (i, expr) in prog.vars.iter().enumerate() {
        // Only look at binary ops.
        let binop = if let SymbolicExpr::Op(binop) = expr {
            binop
        } else {
            push_new_op(&mut res, &mut old_id_to_new_id, i, expr);
            continue;
        };

        // Where the op is modulo.
        if binop.op != Op::Div {
            push_new_op(&mut res, &mut old_id_to_new_id, i, expr);
            continue;
        }
        let lhs = &prog.vars[binop.a];
        let rhs = &prog.vars[binop.b];

        // Where the rhs is a constant.
        let div_by = if let SymbolicExpr::Int(d) = rhs {
            *d
        } else {
            push_new_op(&mut res, &mut old_id_to_new_id, i, expr);
            continue;
        };

        // And the lhs is an addition.
        let (add_a, add_b) = if let SymbolicExpr::Op(SymbolicBinaryOp { op: Op::Add, a, b }) = lhs {
            (*a, *b)
        } else {
            push_new_op(&mut res, &mut old_id_to_new_id, i, expr);
            continue;
        };

        // The addition lhs is a multiplication by `div_by`.
        let stuff_multiplied_by_div_by = if let Some(stuff) = is_mul_by_x(add_a, div_by) {
            stuff
        } else {
            push_new_op(&mut res, &mut old_id_to_new_id, i, expr);
            continue;
        };
        // The new variable is:
        //      add_a + add_b / div_by
        // This must be done in 2 assignments.
        let add_b_div_by = res.push_var(SymbolicExpr::Op(SymbolicBinaryOp {
            op: Op::Div,
            a: old_id_to_new_id[add_b],
            b: old_id_to_new_id[binop.b],
        }));
        let new_i = res.push_var(SymbolicExpr::Op(SymbolicBinaryOp {
            op: Op::Add,
            a: old_id_to_new_id[stuff_multiplied_by_div_by],
            b: add_b_div_by,
        }));
        old_id_to_new_id[i] = new_i;
    }

    res
}

/// Displays the program, and at the same time, keeps track of which input values are affecting the
/// contents of each variable.
fn display_what_inputs_affect_what_vars(prog: &SymbolicProgram) {
    let num_inputs = 14;
    let mut affected_by = vec![u32::MAX; prog.num_vars()];
    // Represent each variable as a fully expanded version, depending only on constants, inputs and
    // binary ops on them.
    let mut fully_expanded_assignments = vec![String::new(); prog.num_vars()];

    for (i, expr) in prog.vars.iter().enumerate() {
        let bits = match expr {
            SymbolicExpr::Int(_) => 0,
            SymbolicExpr::Input(input_idx) => 1u32 << (num_inputs - 1 - input_idx),
            SymbolicExpr::Op(binop) => {
                let a_bits = affected_by[binop.a];
                let b_bits = affected_by[binop.b];
                a_bits | b_bits
            }
        };
        affected_by[i] = bits;
    }

    println!("Which vars depend on which input digits?");
    for (i, (expr, bits)) in prog.vars.iter().zip(affected_by.iter()).enumerate() {
        let lhs = format!("v{}", i);
        let rhs = match expr {
            SymbolicExpr::Int(x) => x.to_string(),
            SymbolicExpr::Input(input_idx) => format!("in[{}]", input_idx),
            SymbolicExpr::Op(binop) => format!("v{} {} v{}", binop.a, binop.op, binop.b),
        };
        let affected_by_str: String = (0..num_inputs)
            .map(|rev_idx| (bits >> (num_inputs - 1 - rev_idx)) & 1)
            .map(|is_on| if is_on == 1 { '#' } else { '.' })
            .collect();
        let fully_expanded = match expr {
            SymbolicExpr::Int(x) => x.to_string(),
            SymbolicExpr::Input(input_idx) => format!("in[{}]", input_idx),
            SymbolicExpr::Op(binop) => {
                let a_is_binop = matches!(prog.vars[binop.a], SymbolicExpr::Op(_));
                let b_is_binop = matches!(prog.vars[binop.b], SymbolicExpr::Op(_));
                let a = &fully_expanded_assignments[binop.a];
                let b = &fully_expanded_assignments[binop.b];

                let a_str = if a.len() > 150 {
                    "...".into()
                } else if a_is_binop {
                    format!("({})", a)
                } else {
                    a.into()
                };
                let b_str = if b.len() > 150 {
                    "...".into()
                } else if b_is_binop {
                    format!("({})", b)
                } else {
                    b.into()
                };
                format!("{} {} {}", a_str, binop.op, b_str)
            }
        };
        println!(
            "| {:>4} = {:<16} {:<14}  {}",
            lhs, rhs, affected_by_str, fully_expanded
        );
        fully_expanded_assignments[i] = fully_expanded;
    }
    println!("+---");
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let str = match self {
            W => "W",
            X => "X",
            Y => "Y",
            Z => "Z",
        };
        write!(f, "{}", str)
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let str = match self {
            Op::Add => "+",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Mod => "%",
            Op::Eql => "==",
        };
        write!(f, "{}", str)
    }
}

impl std::fmt::Display for ModelNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for digit in self.0.iter() {
            write!(f, "{}", *digit)?;
        }
        Ok(())
    }
}

fn format_vars(vars: &[SymbolicExpr], mut f: impl std::fmt::Write) -> Result<(), std::fmt::Error> {
    writeln!(f, "+--- Variables")?;
    for (i, v) in vars.iter().enumerate() {
        write!(f, "| v{} = ", i)?;
        match v {
            SymbolicExpr::Int(x) => writeln!(f, "{}", x)?,
            SymbolicExpr::Input(x) => writeln!(f, "input[{}]", x)?,
            SymbolicExpr::Op(binop) => writeln!(f, "v{} {} v{}", binop.a, binop.op, binop.b)?,
        }
    }
    writeln!(f, "+---")?;

    Ok(())
}

impl std::fmt::Display for SymbolicProgram {
    fn fmt(&self, mut f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "SymbolicProgram")?;
        format_vars(&self.vars, &mut f)?;
        Ok(())
    }
}

fn parse_program<Iter, Item>(lines: Iter) -> Program
where
    Iter: Iterator<Item = Item>,
    Item: AsRef<str>,
{
    let mut instructions = Vec::new();
    for line in lines {
        let line = line.as_ref().trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("inp ") {
            assert_eq!(rest.len(), 1);
            instructions.push(Instruction::Input(Register::from_char(
                rest.chars().next().unwrap(),
            )));
        } else {
            let mut parts = line.split_whitespace();
            let op = parts.next().unwrap();
            let a = parts.next().unwrap();
            let b = parts.next().unwrap();
            assert_eq!(parts.next(), None);

            let op = match op {
                "add" => Op::Add,
                "mul" => Op::Mul,
                "div" => Op::Div,
                "mod" => Op::Mod,
                "eql" => Op::Eql,
                _ => panic!("Invalid op: {}", op),
            };

            assert_eq!(a.len(), 1);
            let a = Register::from_char(a.chars().next().unwrap());

            let b_first = b.chars().next().unwrap();
            let b = if b_first.is_ascii_digit() || b_first == '-' {
                Value::Int(b.parse::<i64>().unwrap())
            } else {
                assert_eq!(b.len(), 1);
                Value::Reg(Register::from_char(b_first))
            };

            instructions.push(Instruction::Op(BinaryOp { op, a, b }));
        }
    }

    Program { instructions }
}

#[cfg(test)]
mod tests {
    use super::{
        BinaryOp, ExecutionError, Instruction, Op, Register::*, RegisterState, SymbolicExpr, Value,
    };

    #[test]
    fn parse_program() {
        let program = super::parse_program("inp x\nmul x -1".lines());
        assert_eq!(
            program.instructions,
            vec![
                Instruction::Input(X),
                Instruction::Op(BinaryOp {
                    op: Op::Mul,
                    a: X,
                    b: Value::Int(-1),
                }),
            ]
        );

        let program_str = "\
                           inp w
                           add x w
                           mod y 123
                               div w -123
                                   eql w z";
        let program = super::parse_program(program_str.lines());
        assert_eq!(
            program.instructions,
            vec![
                Instruction::Input(W),
                Instruction::Op(BinaryOp {
                    op: Op::Add,
                    a: X,
                    b: Value::Reg(W),
                }),
                Instruction::Op(BinaryOp {
                    op: Op::Mod,
                    a: Y,
                    b: Value::Int(123),
                }),
                Instruction::Op(BinaryOp {
                    op: Op::Div,
                    a: W,
                    b: Value::Int(-123),
                }),
                Instruction::Op(BinaryOp {
                    op: Op::Eql,
                    a: W,
                    b: Value::Reg(Z),
                }),
            ]
        );
    }

    #[test]
    fn execute_program() {
        let prog_mul_minus_1 = super::parse_program("inp x\nmul x -1".lines());
        assert_eq!(
            prog_mul_minus_1.execute_with_logging(&[]),
            Err(ExecutionError::InvalidInput)
        );
        assert_eq!(
            prog_mul_minus_1
                .execute_with_logging(&[123])
                .unwrap()
                .last()
                .unwrap(),
            &RegisterState([0, -123, 0, 0])
        );

        let prog_is_mul_by_3 = super::parse_program("inp z\ninp x\nmul z 3\neql z x".lines());
        assert_eq!(
            prog_is_mul_by_3
                .execute_with_logging(&[22, 66])
                .unwrap()
                .last()
                .unwrap(),
            &RegisterState([0, 66, 0, 1])
        );
        assert_eq!(
            prog_is_mul_by_3
                .execute_with_logging(&[22, 65])
                .unwrap()
                .last()
                .unwrap(),
            &RegisterState([0, 65, 0, 0])
        );

        let prog_decompose_into_bits = super::parse_program(
            "\
            inp w
            add z w
            mod z 2
            div w 2
            add y w
            mod y 2
            div w 2
            add x w
            mod x 2
            div w 2
            mod w 2"
                .lines(),
        );
        assert_eq!(
            prog_decompose_into_bits
                .execute_with_logging(&[0b1010])
                .unwrap()
                .last()
                .unwrap(),
            &RegisterState([1, 0, 1, 0])
        );
    }

    /// Check that the optimized symbolic program produces output identical to the regular program.
    #[test]
    fn no_mistake_during_symbolic_execution() {
        let input_program = &advent_of_code::env::get_puzzle_input_path("2021-12-24.txt");

        let prog = super::parse_program(std::fs::read_to_string(&input_program).unwrap().lines());
        let (sym_prog, sym_reg_states) = super::execute_symbolic(&prog, Z);

        assert_eq!(prog.num_inputs(), sym_prog.num_inputs());
        let inputs: Vec<i64> = (1..=prog.num_inputs()).map(|x| x as i64).collect();

        let reg_states = prog.execute_with_logging(&inputs).unwrap();
        assert_eq!(reg_states.len(), sym_reg_states.len());

        // Go state by state, making sure both programs have matching registers.
        // The symbolic program needs to evaluate variables as it goes to do so.
        let mut evaluated_vars = vec![0i64; 4];
        for (idx, (regs, sym_regs)) in reg_states.iter().zip(sym_reg_states.iter()).enumerate() {
            // How far down the list of variables do we need to go to evaluate the symbolic
            // registers?
            let max_required_var_id = *sym_regs.0.iter().max().unwrap();
            for var_id in evaluated_vars.len()..=max_required_var_id {
                let val: i64 = match &sym_prog.vars[var_id] {
                    SymbolicExpr::Int(x) => *x,
                    SymbolicExpr::Input(x) => inputs[*x],
                    SymbolicExpr::Op(binop) => {
                        let a = evaluated_vars[binop.a];
                        let b = evaluated_vars[binop.b];
                        binop.op.apply(a, b).unwrap()
                    }
                };

                assert_eq!(evaluated_vars.len(), var_id);
                evaluated_vars.push(val);
            }

            let evaled_sym_regs = RegisterState(sym_regs.0.map(|var_id| evaluated_vars[var_id]));
            assert_eq!(
                regs, &evaled_sym_regs,
                "Mismatch at instruction {}:\nProg: {:?}\nSym prog: {:?} = {:?}",
                idx, prog.instructions[idx], sym_regs, evaled_sym_regs
            );
        }
    }
}
