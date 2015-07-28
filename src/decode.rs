use ::model::Instruction;
use ::model::Instruction::*;
use ::model::ArrayType;
use ::result::{ParseError, ParseResult};

pub fn decode_code(bytes: &Vec<u8>) -> ParseResult<Vec<(u32, Instruction)>> {
    let mut decoded_instructions = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let opcode = bytes[i];
        let instruction_index = i as u32;
        let instruction = match opcode {
            0x32 => aaload,
            0x53 => aastore,
            0x01 => aconst_null,
            0x19 => { let index = read_u8(bytes, &mut i); aload(index) }
            0x2a => aload_0,
            0x2b => aload_1,
            0x2c => aload_2,
            0x2d => aload_3,
            0xbd => { let index = read_u16(bytes, &mut i); anewarray(index) }
            0xb0 => areturn,
            0xbe => arraylength,
            0x3a => { let index = read_u8(bytes, &mut i); astore(index) }
            0x4b => astore_0,
            0x4c => astore_1,
            0x4d => astore_2,
            0x4e => astore_3,
            0xbf => athrow,
            0x33 => baload,
            0x54 => bastore,
            0x10 => { let val = read_u8(bytes, &mut i) as i8; bipush(val) }
            0x34 => caload,
            0x55 => castore,
            0xc0 => { let index = read_u16(bytes, &mut i); checkcast(index) }
            0x90 => d2f,
            0x8e => d2i,
            0x8f => d2l,
            0x63 => dadd,
            0x31 => daload,
            0x52 => dastore,
            0x98 => dcmpg,
            0x97 => dcmpl,
            0x0e => dconst_0,
            0x0f => dconst_1,
            0x6f => ddiv,
            0x18 => { let index = read_u8(bytes, &mut i); dload(index) }
            0x26 => dload_0,
            0x27 => dload_1,
            0x28 => dload_2,
            0x29 => dload_3,
            0x6b => dmul,
            0x77 => dneg,
            0x73 => drem,
            0xaf => dreturn,
            0x39 => { let index = read_u8(bytes, &mut i); dstore(index) }
            0x47 => dstore_0,
            0x48 => dstore_1,
            0x49 => dstore_2,
            0x4a => dstore_3,
            0x67 => dsub,
            0x59 => dup,
            0x5a => dup_x1,
            0x5b => dup_x2,
            0x5c => dup2,
            0x5d => dup2_x1,
            0x5e => dup2_x2,
            0x8d => f2d,
            0x8b => f2i,
            0x8c => f2l,
            0x62 => fadd,
            0x30 => faload,
            0x51 => fastore,
            0x96 => fcmpg,
            0x95 => fcmpl,
            0x0b => fconst_0,
            0x0c => fconst_1,
            0x0d => fconst_2,
            0x6e => fdiv,
            0x17 => { let index = read_u8(bytes, &mut i); fload(index) }
            0x22 => fload_0,
            0x23 => fload_1,
            0x24 => fload_2,
            0x25 => fload_3,
            0x6a => fmul,
            0x76 => fneg,
            0x72 => frem,
            0xae => freturn,
            0x38 => { let index = read_u8(bytes, &mut i); fstore(index) }
            0x43 => fstore_0,
            0x44 => fstore_1,
            0x45 => fstore_2,
            0x46 => fstore_3,
            0x66 => fsub,
            0xb4 => { let index = read_u16(bytes, &mut i); getfield(index) }
            0xb2 => { let index = read_u16(bytes, &mut i); getstatic(index) }
            0xa7 => { let offset = read_u16(bytes, &mut i) as i16; goto(offset) }
            0xc8 => { let offset = read_u32(bytes, &mut i) as i32; goto_w(offset) }
            0x91 => i2b,
            0x92 => i2c,
            0x87 => i2d,
            0x86 => i2f,
            0x85 => i2l,
            0x93 => i2s,
            0x60 => iadd,
            0x2e => iaload,
            0x7e => iand,
            0x4f => iastore,
            0x02 => iconst_m1,
            0x03 => iconst_0,
            0x04 => iconst_1,
            0x05 => iconst_2,
            0x06 => iconst_3,
            0x07 => iconst_4,
            0x08 => iconst_5,
            0x6c => idiv,
            0xa5 => { let index = read_u16(bytes, &mut i) as i16; if_acmpeq(index) }
            0xa6 => { let index = read_u16(bytes, &mut i) as i16; if_acmpne(index) }
            0x9f => { let index = read_u16(bytes, &mut i) as i16; if_icmpeq(index) }
            0xa0 => { let index = read_u16(bytes, &mut i) as i16; if_icmpne(index) }
            0xa1 => { let index = read_u16(bytes, &mut i) as i16; if_icmplt(index) }
            0xa2 => { let index = read_u16(bytes, &mut i) as i16; if_icmpge(index) }
            0xa3 => { let index = read_u16(bytes, &mut i) as i16; if_icmpgt(index) }
            0xa4 => { let index = read_u16(bytes, &mut i) as i16; if_icmple(index) }
            0x99 => { let index = read_u16(bytes, &mut i) as i16; ifeq(index) }
            0x9a => { let index = read_u16(bytes, &mut i) as i16; ifne(index) }
            0x9b => { let index = read_u16(bytes, &mut i) as i16; iflt(index) }
            0x9c => { let index = read_u16(bytes, &mut i) as i16; ifge(index) }
            0x9d => { let index = read_u16(bytes, &mut i) as i16; ifgt(index) }
            0x9e => { let index = read_u16(bytes, &mut i) as i16; ifle(index) }
            0xc7 => { let index = read_u16(bytes, &mut i) as i16; ifnonnull(index) }
            0xc6 => { let index = read_u16(bytes, &mut i) as i16; ifnull(index) }
            0x84 => {
                let index = read_u8(bytes, &mut i);
                let constant = read_u8(bytes, &mut i) as i8;
                iinc(index, constant)
            }
            0x15 => { let index = read_u8(bytes, &mut i); iload(index) }
            0x1a => iload_0,
            0x1b => iload_1,
            0x1c => iload_2,
            0x1d => iload_3,
            0x68 => imul,
            0x74 => ineg,
            0xc1 => { let index = read_u16(bytes, &mut i); instanceof(index) }
            0xba => { let index = read_u16(bytes, &mut i); invokedynamic(index) }
            0xb9 => {
                let index = read_u16(bytes, &mut i);
                let count = read_u8(bytes, &mut i);
                invokeinterface(index, count)
            }
            0xb7 => { let index = read_u16(bytes, &mut i); invokespecial(index) }
            0xb8 => { let index = read_u16(bytes, &mut i); invokestatic(index) }
            0xb6 => { let index = read_u16(bytes, &mut i); invokevirtual(index) }
            0x80 => ior,
            0x70 => irem,
            0xac => ireturn,
            0x78 => ishl,
            0x7a => ishr,
            0x36 => { let index = read_u8(bytes, &mut i); istore(index) }
            0x3b => istore_0,
            0x3c => istore_1,
            0x3d => istore_2,
            0x3e => istore_3,
            0x64 => isub,
            0x7c => iushr,
            0x82 => ixor,
            0xa8 => { let offset = read_u16(bytes, &mut i) as i16; jsr(offset) }
            0xc9 => { let offset = read_u32(bytes, &mut i) as i32; jsr_w(offset) }
            0x8a => l2d,
            0x89 => l2f,
            0x88 => l2i,
            0x61 => ladd,
            0x2f => laload,
            0x7f => land,
            0x50 => lastore,
            0x94 => lcmp,
            0x09 => lconst_0,
            0x0a => lconst_1,
            0x12 => { let index = read_u8(bytes, &mut i); ldc(index) }
            0x13 => { let index = read_u16(bytes, &mut i); ldc_w(index) }
            0x14 => { let index = read_u16(bytes, &mut i); ldc2_w(index) }
            0x6d => ldiv,
            0x16 => { let index = read_u8(bytes, &mut i); lload(index) }
            0x1e => lload_0,
            0x1f => lload_1,
            0x20 => lload_2,
            0x21 => lload_3,
            0x69 => lmul,
            0x75 => lneg,
            0xab => {
                let padding = (4 - ((i + 1) % 4)) % 4;
                i += padding;
                let default = read_u32(bytes, &mut i) as i32;
                let npairs = read_u32(bytes, &mut i) as i32;
                let mut pairs = Vec::with_capacity(npairs as usize);
                for _ in 0..npairs {
                    let match_ = read_u32(bytes, &mut i) as i32;
                    let offset = read_u32(bytes, &mut i) as i32;
                    pairs.push((match_, offset));
                }
                lookupswitch(default, pairs.into_boxed_slice())
            }
            0x81 => lor,
            0x71 => lrem,
            0xad => lreturn,
            0x79 => lshl,
            0x7b => lshr,
            0x37 => { let index = read_u8(bytes, &mut i); lstore(index) }
            0x3f => lstore_0,
            0x40 => lstore_1,
            0x41 => lstore_2,
            0x42 => lstore_3,
            0x65 => lsub,
            0x7d => lushr,
            0x83 => lxor,
            0xc2 => monitorenter,
            0xc3 => monitorexit,
            0xc5 => {
                let index = read_u16(bytes, &mut i);
                let dimensions = read_u8(bytes, &mut i);
                multianewarray(index, dimensions)
            }
            0xbb => { let index = read_u16(bytes, &mut i); new(index) }
            0xbc => {
                let atype = read_u8(bytes, &mut i);
                let atype = match atype {
                    4 => ArrayType::Boolean,
                    5 => ArrayType::Char,
                    6 => ArrayType::Float,
                    7 => ArrayType::Double,
                    8 => ArrayType::Byte,
                    9 => ArrayType::Short,
                    10 => ArrayType::Int,
                    11 => ArrayType::Long,
                    _ => { return Result::Err(ParseError::Decode(format!("unknown array type {}", atype))); }
                };
                newarray(atype)
            }
            0x00 => nop,
            0x57 => pop,
            0x58 => pop2,
            0xb5 => { let index = read_u16(bytes, &mut i); putfield(index) }
            0xb3 => { let index = read_u16(bytes, &mut i); putstatic(index) }
            0xa9 => { let index = read_u8(bytes, &mut i); ret(index) }
            0xb1 => return_,
            0x35 => saload,
            0x56 => sastore,
            0x11 => { let value = read_u16(bytes, &mut i) as i16; sipush(value) }
            0x5f => swap,
            0xaa => {
                let padding = (4 - ((i + 1) % 4)) % 4;
                i += padding;
                let default = read_u32(bytes, &mut i) as i32;
                let low = read_u32(bytes, &mut i) as i32;
                let high = read_u32(bytes, &mut i) as i32;
                let mut offsets = Vec::with_capacity((high - low + 1) as usize);
                for _ in low..(high + 1) {
                    let offset = read_u32(bytes, &mut i) as i32;
                    offsets.push(offset);
                }
                tableswitch(default, low, offsets.into_boxed_slice())
            }
            0xc4 => {
                let opcode = read_u8(bytes, &mut i);
                let index = read_u16(bytes, &mut i);
                match opcode {
                    0x15 => iload_w(index),
                    0x17 => fload_w(index),
                    0x19 => aload_w(index),
                    0x16 => lload_w(index),
                    0x18 => dload_w(index),
                    0x36 => istore_w(index),
                    0x38 => fstore_w(index),
                    0x3a => astore_w(index),
                    0x39 => dstore_w(index),
                    0xa9 => ret_w(index),
                    0x84 => { let constant = read_u16(bytes, &mut i) as i16; iinc_w(index, constant) }
                    _ => { return Result::Err(ParseError::Decode(format!("unknown opcode {} in wide instruction", opcode))); }
                }
            }
            _ => { return Result::Err(ParseError::Decode(format!("unknown opcode {}", opcode))); }
        };
        decoded_instructions.push((instruction_index, instruction));
        i += 1;
    }
    Result::Ok(decoded_instructions)
}

fn read_u32(bytes: &Vec<u8>, i: &mut usize) -> u32 {
    let val = ((bytes[*i + 1] as u32) << 24)
            + ((bytes[*i + 2] as u32) << 16)
            + ((bytes[*i + 3] as u32) << 8)
            + (bytes[*i + 4] as u32);
    *i += 4;
    val
}

fn read_u16(bytes: &Vec<u8>, i: &mut usize) -> u16 {
    let val = ((bytes[*i + 1] as u16) << 8) + (bytes[*i + 2] as u16);
    *i += 2;
    val
}

fn read_u8(bytes: &Vec<u8>, i: &mut usize) -> u8 {
    *i += 1;
    bytes[*i]
}
