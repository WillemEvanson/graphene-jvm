pub mod call_frame;
pub mod class;
pub mod value;

use call_frame::CallFrame;
use class::{parse, Class, Entry, Instruction, ParseError};

use crate::java_str;
use crate::string::JavaStr;

#[derive(Debug, Default)]
pub struct ClassManager {
    classes: Vec<Class>,
}

impl ClassManager {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }

    pub fn load(&mut self, slice: &[u8]) -> Result<(), ParseError> {
        self.classes.push(parse(slice)?);
        Ok(())
    }

    pub fn get<'a>(&'a self, name: &JavaStr) -> Option<&'a Class> {
        self.classes.iter().find(|class| class.name() == name)
    }
}

pub fn execute(classes: &ClassManager, main_class: &JavaStr) {
    #[track_caller]
    fn bin_op_int<F: FnOnce(i32, i32) -> i32>(frame: &mut CallFrame, f: F) {
        let rhs = frame.stack.pop_int();
        let lhs = frame.stack.pop_int();
        frame.stack.push_int(f(lhs, rhs));
    }
    #[track_caller]
    fn bin_op_long<F: FnOnce(i64, i64) -> i64>(frame: &mut CallFrame, f: F) {
        let rhs = frame.stack.pop_long();
        let lhs = frame.stack.pop_long();
        frame.stack.push_long(f(lhs, rhs));
    }
    #[track_caller]
    fn bin_op_float<F: FnOnce(f32, f32) -> f32>(frame: &mut CallFrame, f: F) {
        let rhs = frame.stack.pop_float();
        let lhs = frame.stack.pop_float();
        frame.stack.push_float(f(lhs, rhs));
    }
    #[track_caller]
    fn bin_op_double<F: FnOnce(f64, f64) -> f64>(frame: &mut CallFrame, f: F) {
        let rhs = frame.stack.pop_double();
        let lhs = frame.stack.pop_double();
        frame.stack.push_double(f(lhs, rhs));
    }

    #[track_caller]
    fn un_op_int<F: FnOnce(i32) -> i32>(frame: &mut CallFrame, f: F) {
        let val = frame.stack.pop_int();
        frame.stack.push_int(f(val));
    }
    #[track_caller]
    fn un_op_float<F: FnOnce(f32) -> f32>(frame: &mut CallFrame, f: F) {
        let val = frame.stack.pop_float();
        frame.stack.push_float(f(val));
    }
    #[track_caller]
    fn un_op_long<F: FnOnce(i64) -> i64>(frame: &mut CallFrame, f: F) {
        let val = frame.stack.pop_long();
        frame.stack.push_long(f(val));
    }
    #[track_caller]
    fn un_op_double<F: FnOnce(f64) -> f64>(frame: &mut CallFrame, f: F) {
        let val = frame.stack.pop_double();
        frame.stack.push_double(f(val));
    }

    const MAIN_METHOD_NAME: &JavaStr = java_str!("main");
    const METHOD_METHOD_DESCRIPTOR: &JavaStr = java_str!("([Ljava/lang/String;)V");

    let Some(main_class) = classes.get(main_class) else {
        panic!("expected class \"{main_class}\" to be loaded");
    };
    let main_method = main_class.get_method(MAIN_METHOD_NAME, METHOD_METHOD_DESCRIPTOR);

    let mut call_stack = vec![CallFrame::new(
        main_class.constants(),
        main_method.bytecode().unwrap(),
    )];

    while let Some(frame) = call_stack.last_mut() {
        'method: while let Some((pc, inst)) = frame.bytecode.next() {
            match inst {
                // Constant
                Instruction::nop => (),
                Instruction::aconst_null => todo!("no support for objects"),
                Instruction::iconst(val) => frame.stack.push_int(val),
                Instruction::lconst(val) => frame.stack.push_long(val),
                Instruction::fconst(val) => frame.stack.push_float(val),
                Instruction::dconst(val) => frame.stack.push_double(val),
                Instruction::bipush(val) => frame.stack.push_int(val as i32),
                Instruction::sipush(val) => frame.stack.push_int(val as i32),
                Instruction::ldc(idx) => match frame.constants.get(idx) {
                    Entry::Integer(val) => frame.stack.push_int(*val),
                    Entry::Long(val) => frame.stack.push_long(*val),
                    Entry::Float(val) => frame.stack.push_float(*val),
                    Entry::Double(val) => frame.stack.push_double(*val),
                    entry => panic!("unexpected constant pool entry type: {entry:?}"),
                },

                // Load
                Instruction::iload(idx) => {
                    let val = frame.locals.get_int(idx as usize);
                    frame.stack.push_int(val);
                }
                Instruction::lload(idx) => {
                    let val = frame.locals.get_long(idx as usize);
                    frame.stack.push_long(val);
                }
                Instruction::fload(idx) => {
                    let val = frame.locals.get_float(idx as usize);
                    frame.stack.push_float(val);
                }
                Instruction::dload(idx) => {
                    let val = frame.locals.get_double(idx as usize);
                    frame.stack.push_double(val);
                }

                // Store
                Instruction::istore(idx) => {
                    let val = frame.stack.pop_int();
                    frame.locals.set_int(idx as usize, val);
                }
                Instruction::lstore(idx) => {
                    let val = frame.stack.pop_long();
                    frame.locals.set_long(idx as usize, val);
                }
                Instruction::fstore(idx) => {
                    let val = frame.stack.pop_float();
                    frame.locals.set_float(idx as usize, val);
                }
                Instruction::dstore(idx) => {
                    let val = frame.stack.pop_double();
                    frame.locals.set_double(idx as usize, val);
                }

                // Stack
                Instruction::pop => frame.stack.inst_pop(),
                Instruction::pop2 => frame.stack.inst_pop2(),
                Instruction::dup => frame.stack.inst_dup(),
                Instruction::dup_x1 => frame.stack.inst_dup_x1(),
                Instruction::dup_x2 => frame.stack.inst_dup_x2(),
                Instruction::dup2 => frame.stack.inst_dup2(),
                Instruction::dup2_x1 => frame.stack.inst_dup2_x1(),
                Instruction::dup2_x2 => frame.stack.inst_dup2_x2(),
                Instruction::swap => frame.stack.inst_swap(),

                // Math
                Instruction::iadd => bin_op_int(frame, std::ops::Add::add),
                Instruction::fadd => bin_op_float(frame, std::ops::Add::add),
                Instruction::ladd => bin_op_long(frame, std::ops::Add::add),
                Instruction::dadd => bin_op_double(frame, std::ops::Add::add),
                Instruction::isub => bin_op_int(frame, std::ops::Sub::sub),
                Instruction::fsub => bin_op_float(frame, std::ops::Sub::sub),
                Instruction::lsub => bin_op_long(frame, std::ops::Sub::sub),
                Instruction::dsub => bin_op_double(frame, std::ops::Sub::sub),
                Instruction::imul => bin_op_int(frame, std::ops::Mul::mul),
                Instruction::fmul => bin_op_float(frame, std::ops::Mul::mul),
                Instruction::lmul => bin_op_long(frame, std::ops::Mul::mul),
                Instruction::dmul => bin_op_double(frame, std::ops::Mul::mul),
                Instruction::idiv => bin_op_int(frame, std::ops::Div::div),
                Instruction::fdiv => bin_op_float(frame, std::ops::Div::div),
                Instruction::ldiv => bin_op_long(frame, std::ops::Div::div),
                Instruction::ddiv => bin_op_double(frame, std::ops::Div::div),
                Instruction::irem => bin_op_int(frame, std::ops::Rem::rem),
                Instruction::frem => bin_op_float(frame, std::ops::Rem::rem),
                Instruction::lrem => bin_op_long(frame, std::ops::Rem::rem),
                Instruction::drem => bin_op_double(frame, std::ops::Rem::rem),
                Instruction::ineg => un_op_int(frame, std::ops::Neg::neg),
                Instruction::fneg => un_op_float(frame, std::ops::Neg::neg),
                Instruction::lneg => un_op_long(frame, std::ops::Neg::neg),
                Instruction::dneg => un_op_double(frame, std::ops::Neg::neg),
                Instruction::ishl => bin_op_int(frame, |val1, val2| val1.wrapping_shl(val2 as u32)),
                Instruction::lshl => {
                    bin_op_long(frame, |val1, val2| val1.wrapping_shl(val2 as u32));
                }
                Instruction::ishr => bin_op_int(frame, |val1, val2| val1.wrapping_shr(val2 as u32)),
                Instruction::lshr => {
                    bin_op_long(frame, |val1, val2| val1.wrapping_shr(val2 as u32));
                }
                Instruction::iushr => bin_op_int(frame, |val1, val2| {
                    (val1 as u32).wrapping_shr(val2 as u32) as i32
                }),
                Instruction::lushr => bin_op_int(frame, |val1, val2| {
                    (val1 as u32).wrapping_shr(val2 as u32) as i32
                }),
                Instruction::iand => bin_op_int(frame, std::ops::BitAnd::bitand),
                Instruction::land => bin_op_long(frame, std::ops::BitAnd::bitand),
                Instruction::ior => bin_op_int(frame, std::ops::BitOr::bitor),
                Instruction::lor => bin_op_long(frame, std::ops::BitOr::bitor),
                Instruction::ixor => bin_op_int(frame, std::ops::BitXor::bitxor),
                Instruction::lxor => bin_op_long(frame, std::ops::BitXor::bitxor),

                // Conversion
                Instruction::i2l => {
                    let val = frame.stack.pop_int();
                    frame.stack.push_long(val as i64);
                }
                Instruction::i2f => {
                    let val = frame.stack.pop_int();
                    frame.stack.push_float(val as f32);
                }
                Instruction::i2d => {
                    let val = frame.stack.pop_int();
                    frame.stack.push_double(val as f64);
                }
                Instruction::l2i => {
                    let val = frame.stack.pop_long();
                    frame.stack.push_int(val as i32);
                }
                Instruction::l2f => {
                    let val = frame.stack.pop_long();
                    frame.stack.push_float(val as f32);
                }
                Instruction::l2d => {
                    let val = frame.stack.pop_long();
                    frame.stack.push_double(val as f64);
                }
                Instruction::f2i => {
                    let val = frame.stack.pop_float();
                    frame.stack.push_int(val as i32);
                }
                Instruction::f2l => {
                    let val = frame.stack.pop_float();
                    frame.stack.push_long(val as i64);
                }
                Instruction::f2d => {
                    let val = frame.stack.pop_float();
                    frame.stack.push_double(val as f64);
                }
                Instruction::d2i => {
                    let val = frame.stack.pop_double();
                    frame.stack.push_int(val as i32);
                }
                Instruction::d2l => {
                    let val = frame.stack.pop_double();
                    frame.stack.push_long(val as i64);
                }
                Instruction::d2f => {
                    let val = frame.stack.pop_double();
                    frame.stack.push_float(val as f32);
                }
                Instruction::i2b => {
                    let val = frame.stack.pop_int();
                    frame.stack.push_int((val as i8) as i32);
                }
                Instruction::i2c => {
                    let val = frame.stack.pop_int();
                    frame.stack.push_int((val as u16) as i32);
                }
                Instruction::i2s => {
                    let val = frame.stack.pop_int();
                    frame.stack.push_int((val as i16) as i32);
                }

                // Comparison
                Instruction::lcmp => {
                    let val1 = frame.stack.pop_long();
                    let val2 = frame.stack.pop_long();
                    let result = match val1.cmp(&val2) {
                        std::cmp::Ordering::Greater => 1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Less => -1,
                    };
                    frame.stack.push_int(result);
                }
                Instruction::fcmp(greater_if_nan) => {
                    let val1 = frame.stack.pop_float();
                    let val2 = frame.stack.pop_float();
                    let result = match val1.partial_cmp(&val2) {
                        Some(std::cmp::Ordering::Greater) => 1,
                        Some(std::cmp::Ordering::Equal) => 0,
                        Some(std::cmp::Ordering::Less) => -1,
                        None => {
                            if greater_if_nan {
                                1
                            } else {
                                -1
                            }
                        }
                    };
                    frame.stack.push_int(result);
                }
                Instruction::dcmp(less_if_nan) => {
                    let val1 = frame.stack.pop_double();
                    let val2 = frame.stack.pop_double();
                    let result = match val1.partial_cmp(&val2) {
                        Some(std::cmp::Ordering::Greater) => 1,
                        Some(std::cmp::Ordering::Equal) => 0,
                        Some(std::cmp::Ordering::Less) => -1,
                        None => {
                            if less_if_nan {
                                1
                            } else {
                                -1
                            }
                        }
                    };
                    frame.stack.push_int(result);
                }
                Instruction::if_eq(offset) => {
                    let val = frame.stack.pop_int();
                    if val == 0 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_ne(offset) => {
                    let val = frame.stack.pop_int();
                    if val != 0 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_lt(offset) => {
                    let val = frame.stack.pop_int();
                    if val < 0 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_le(offset) => {
                    let val = frame.stack.pop_int();
                    if val <= 0 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_gt(offset) => {
                    let val = frame.stack.pop_int();
                    if val > 0 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_ge(offset) => {
                    let val = frame.stack.pop_int();
                    if val >= 0 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_icmp_eq(offset) => {
                    let val2 = frame.stack.pop_int();
                    let val1 = frame.stack.pop_int();
                    if val1 == val2 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_icmp_ne(offset) => {
                    let val2 = frame.stack.pop_int();
                    let val1 = frame.stack.pop_int();
                    if val1 != val2 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_icmp_lt(offset) => {
                    let val2 = frame.stack.pop_int();
                    let val1 = frame.stack.pop_int();
                    if val1 < val2 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_icmp_le(offset) => {
                    let val2 = frame.stack.pop_int();
                    let val1 = frame.stack.pop_int();
                    if val1 <= val2 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_icmp_gt(offset) => {
                    let val2 = frame.stack.pop_int();
                    let val1 = frame.stack.pop_int();
                    if val1 > val2 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }
                Instruction::if_icmp_ge(offset) => {
                    let val2 = frame.stack.pop_int();
                    let val1 = frame.stack.pop_int();
                    if val1 >= val2 {
                        frame.bytecode.set_pc(pc.wrapping_add_signed(offset as i32));
                    }
                }

                // Control
                Instruction::goto(offset) => frame.bytecode.set_pc(pc.wrapping_add_signed(offset)),
                Instruction::jsr(offset) => {
                    frame.stack.push_ret_addr(frame.bytecode.pc());
                    frame.bytecode.set_pc(pc.wrapping_add_signed(offset));
                }
                Instruction::ret(idx) => {
                    let ret_addr = frame.locals.get_ret_addr(idx as usize);
                    frame.bytecode.set_pc(ret_addr);
                }
                Instruction::tableswitch(switch) => {
                    let idx = frame.stack.pop_int();
                    let offset = switch.lookup(idx);
                    frame.bytecode.set_pc(pc.wrapping_add_signed(offset));
                }
                Instruction::lookupswitch(switch) => {
                    let key = frame.stack.pop_int();
                    let offset = switch.lookup(key);
                    frame.bytecode.set_pc(pc.wrapping_add_signed(offset));
                }
                Instruction::ireturn => {
                    let ret_val = frame.stack.pop_int();
                    call_stack.pop();

                    let invoker_frame = call_stack.last_mut().unwrap();
                    invoker_frame.stack.push_int(ret_val);
                    break 'method;
                }
                Instruction::lreturn => {
                    let ret_val = frame.stack.pop_long();
                    call_stack.pop();

                    let invoker_frame = call_stack.last_mut().unwrap();
                    invoker_frame.stack.push_long(ret_val);
                    break 'method;
                }
                Instruction::freturn => {
                    let ret_val = frame.stack.pop_float();
                    call_stack.pop();

                    let invoker_frame = call_stack.last_mut().unwrap();
                    invoker_frame.stack.push_float(ret_val);
                    break 'method;
                }
                Instruction::dreturn => {
                    let ret_val = frame.stack.pop_double();
                    call_stack.pop();

                    let invoker_frame = call_stack.last_mut().unwrap();
                    invoker_frame.stack.push_double(ret_val);
                    break 'method;
                }
                Instruction::ret_void => {
                    call_stack.pop();
                    break 'method;
                }

                // Reference
                Instruction::invokestatic(idx) => {
                    let (class, name_type) = frame.constants.get(idx).into_ref();
                    let class = frame.constants.get(class).into_class();

                    // We should be checking that the class is initialized, and if not, then
                    // initializing it. Right now we assume it to be already initialized. This does
                    // mean that static fields don't work.
                    let class_name = frame.constants.get(class).into_utf8();

                    let (method_name, method_descriptor) =
                        frame.constants.get(name_type).into_name_type();
                    let method_name = frame.constants.get(method_name).into_utf8();
                    let method_descriptor = frame.constants.get(method_descriptor).into_utf8();

                    let class = classes
                        .get(class_name)
                        .expect("All classes which are to be used should be already loaded");
                    let method = class.get_method(method_name, method_descriptor);

                    let mut invoked_frame =
                        CallFrame::new(class.constants(), method.bytecode().expect("This instruction calls static methods and static methods must have a Code attribute"));

                    // Load method arguments into invoked method's frame
                    let mut i = 0;
                    for _ in method.parsed_descriptor().args() {
                        let value = frame.stack.pop();
                        invoked_frame.locals.set(i, value);
                        i += value.size();
                    }

                    call_stack.push(invoked_frame);
                    break 'method;
                }

                // Extended
                Instruction::iinc(idx, constant) => {
                    let value = frame.locals.get_int(idx as usize);
                    frame
                        .locals
                        .set_int(idx as usize, value.wrapping_add(constant as i32));
                }

                inst => todo!("{inst:?} not implemented: no support for objects"),
            }
        }
    }
}
