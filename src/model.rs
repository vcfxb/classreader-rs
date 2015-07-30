macro_rules! modifier_raw {
    ($name:ident, $field:ident, $mask:expr) => {
        pub fn $name(&self) -> bool {
            self.$field & $mask == $mask
        }
    }
}

macro_rules! modifier {
    ($name:ident, $mask:expr) => { modifier_raw!($name, access_flags, $mask); }
}

macro_rules! modifier_inner {
    ($name:ident, $mask:expr) => { modifier_raw!($name, inner_class_access_flags, $mask); }
}

#[derive(Debug)]
pub struct Class {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolInfo>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>
}

impl Class {
    modifier!(is_public, 0x0001);
    modifier!(is_final, 0x0010);
    modifier!(is_super, 0x0020);
    modifier!(is_interface, 0x0200);
    modifier!(is_abstract, 0x0400);
    modifier!(is_synthetic, 0x1000);
    modifier!(is_annotation, 0x2000);
    modifier!(is_enum, 0x4000);
}

#[derive(Debug)]
pub enum ConstantPoolInfo {
    Utf8(String),                       // 1
    Integer(i32),                       // 3
    Float(f32),                         // 4
    Long(i64),                          // 5
    Double(f64),                        // 6
    Class(u16),                         // 7
    String(u16),                        // 8
    Fieldref(u16, u16),                 // 9
    Methodref(u16, u16),                // 10
    InterfaceMethodref(u16, u16),       // 11
    NameAndType(u16, u16),              // 12
    MethodHandle(u8, u16),              // 15
    MethodType(u16),                    // 16
    InvokeDynamic(u16, u16),            // 18
    Invalid
}

#[derive(Debug)]
pub struct Field {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>
}

impl Field {
    modifier!(is_public, 0x0001);
    modifier!(is_private, 0x0002);
    modifier!(is_protected, 0x0004);
    modifier!(is_static, 0x0008);
    modifier!(is_final, 0x0010);
    modifier!(is_volatile, 0x0040);
    modifier!(is_transient, 0x0080);
    modifier!(is_synthetic, 0x1000);
    modifier!(is_enum, 0x4000);
}

#[derive(Debug)]
pub struct Method {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>
}

impl Method {
    modifier!(is_public, 0x0001);
    modifier!(is_private, 0x0002);
    modifier!(is_protected, 0x0004);
    modifier!(is_static, 0x0008);
    modifier!(is_final, 0x0010);
    modifier!(is_synchronized, 0x0020);
    modifier!(is_bridge, 0x0040);
    modifier!(is_varargs, 0x0080);
    modifier!(is_native, 0x01000);
    modifier!(is_abstract, 0x0400);
    modifier!(is_strict, 0x0800);
    modifier!(is_synthetic, 0x1000);
}

#[derive(Debug)]
pub enum Attribute {
    ConstantValue{ constvalue_index: u16 },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<(u32, Instruction)>,
        exception_table: Vec<Exception>,
        attributes: Vec<Attribute>
    },
    StackMapTable(Vec<StackMapFrame>),
    Exceptions { exception_index_table: Vec<u16> },
    InnerClasses { classes: Vec<InnerClass> },
    EnclosingMethod { class_index: u16, method_index: u16 },
    Synthetic,
    Signature { signature_index: u16 },
    SourceFile { sourcefile_index: u16 },
    SourceDebugExtension(Vec<u8>),
    LineNumberTable(Vec<LineNumber>),
    LocalVariableTable(Vec<LocalVariable>),
    LocalVariableTypeTable(Vec<LocalVariable>),
    Deprecated,
    RuntimeVisibleAnnotations(Vec<Annotation>),
    RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeInvisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeVisibleTypeAnnotations(Vec<TypeAnnotation>),
    RuntimeInvisibleTypeAnnotations(Vec<TypeAnnotation>),
    AnnotationDefault { element_value: ElementValue },
    BootstrapMethods(Vec<BootstrapMethod>),
    MethodParameters(Vec<MethodParameter>),
    Unknown(Vec<u8>)
}

#[derive(Debug)]
pub struct Exception {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16
}

#[derive(Debug)]
pub enum StackMapFrame {
    SameFrame,
    SameLocals1StackItemFrame(VerificationType),
    SameLocals1StackItemFrameExtended { offset_delta: u16, stack: VerificationType },
    ChopFrame { offset_delta: u16 },
    SameFrameExtended { offset_delta: u16 },
    AppendFrame { offset_delta: u16, locals: Vec<VerificationType> },
    FullFrame {
        offset_delta: u16,
        locals: Vec<VerificationType>,
        stack: Vec<VerificationType>
    }
}

#[derive(Debug)]
pub enum VerificationType {
    Top,                                        // 0
    Integer,                                    // 1
    Float,                                      // 2
    Double,                                     // 3
    Long,                                       // 4
    Null,                                       // 5
    UninitializedThis,                          // 6
    Object { index: u16 },                      // 7
    UninitializedVariable { offset: u16 },      // 8
}

#[derive(Debug)]
pub struct InnerClass {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: u16
}

impl InnerClass {
    modifier_inner!(is_public, 0x0001);
    modifier_inner!(is_private, 0x0002);
    modifier_inner!(is_protected, 0x0004);
    modifier_inner!(is_static, 0x0008);
    modifier_inner!(is_final, 0x0010);
    modifier_inner!(is_interface, 0x0200);
    modifier_inner!(is_abstract, 0x0400);
    modifier_inner!(is_synthetic, 0x1000);
    modifier_inner!(is_annotation, 0x2000);
    modifier_inner!(is_enum, 0x4000);
}

#[derive(Debug)]
pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16
}

#[derive(Debug)]
pub struct LocalVariable {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_or_signature_index: u16,
    pub index: u16
}

#[derive(Debug)]
pub struct Annotation {
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>
}

#[derive(Debug)]
pub struct TypeAnnotation {
    pub target_type: TargetType,
    pub target_info: TargetInfo,
    pub type_path: TypePath,
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>
}

#[derive(Debug)]
pub struct ElementValuePair {
    pub element_name_index: u16,
    pub value: ElementValue
}

#[derive(Debug)]
pub enum ElementValue {
    Constant { const_value_index: u16 },
    EnumConstant { type_name_index: u16, const_name_index: u16 },
    Class { class_info_index: u16 },
    Annotation(Annotation),
    Array(Vec<ElementValue>)
}

#[derive(Debug)]
pub enum TargetType {
    Type,                               // 0x00
    Method,                             // 0x01
    Supertype,                          // 0x10
    TypeBound,                          // 0x11
    MethodBound,                        // 0x12
    Field,                              // 0x13
    MethodReturnType,                   // 0x14
    ReceiverType,                       // 0x15
    Parameter,                          // 0x16
    Throws,                             // 0x17
    LocalVariableDeclaration,           // 0x40
    ResourceVariableDeclaration,        // 0x41
    ExceptionParameterDeclaration,      // 0x42
    Instanceof,                         // 0x43
    New,                                // 0x44
    MethodReferenceNew,                 // 0x45
    MethodReference,                    // 0x46
    Cast,                               // 0x47
    ConstructorArgument,                // 0x48
    MethodArgument,                     // 0x49
    MethodReferenceNewArgument,         // 0x4A
    MethodReferenceArgument             // 0x4B
}

#[derive(Debug)]
pub enum TargetInfo {
    TypeParameter { index: u8 },
    Supertype { index: u16 },
    TypeParameterBound { index: u8, bound_index: u8 },
    Empty,
    MethodFormalParameter { index: u8 },
    Throws { type_index: u16 },
    Localvar(Vec<LocalVariableTarget>),
    Catch { exception_table_index: u16 },
    Offset(u16),
    TypeArgument { offset: u16, index: u8 }
}

#[derive(Debug)]
pub struct TypePath {
    pub path: Vec<PathElement>
}

#[derive(Debug)]
pub struct PathElement {
    pub kind: TypePathKind,
    pub argument_index: u8
}

#[derive(Debug)]
pub enum TypePathKind {
    Array,              // 0
    Nested,             // 1
    WildcardBound,      // 2
    TypeArgument        // 3
}

#[derive(Debug)]
pub struct LocalVariableTarget {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16
}

#[derive(Debug)]
pub struct BootstrapMethod {
    pub method_ref: u16,
    pub arguments: Vec<u16>
}

#[derive(Debug)]
pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: u16
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    aaload,                                     //  50 (0x32)
    aastore,                                    //  83 (0x53)
    aconst_null,                                //   1 (0x01)
    aload(u8),                                  //  25 (0x19)
    aload_0,                                    //  42 (0x2a)
    aload_1,                                    //  43 (0x2b)
    aload_2,                                    //  44 (0x2c)
    aload_3,                                    //  45 (0x2d)
    anewarray(u16),                             // 189 (0xbd)
    areturn,                                    // 176 (0xb0)
    arraylength,                                // 190 (0xbe)
    astore(u8),                                 //  58 (0x3a)
    astore_0,                                   //  75 (0x4b)
    astore_1,                                   //  76 (0x4c)
    astore_2,                                   //  77 (0x4d)
    astore_3,                                   //  78 (0x4e)
    athrow,                                     // 191 (0xbf)
    baload,                                     //  51 (0x33)
    bastore,                                    //  84 (0x54)
    bipush(i8),                                 //  16 (0x10)
    caload,                                     //  52 (0x34)
    castore,                                    //  85 (0x55)
    checkcast(u16),                             // 192 (0xc0)
    d2f,                                        // 144 (0x90)
    d2i,                                        // 142 (0x8e)
    d2l,                                        // 143 (0x8f)
    dadd,                                       //  99 (0x63)
    daload,                                     //  49 (0x31)
    dastore,                                    //  82 (0x52)
    dcmpg,                                      // 152 (0x98)
    dcmpl,                                      // 151 (0x97)
    dconst_0,                                   //  14 (0x0e)
    dconst_1,                                   //  15 (0x0f)
    ddiv,                                       // 111 (0x6f)
    dload(u8),                                  //  24 (0x18)
    dload_0,                                    //  38 (0x26)
    dload_1,                                    //  39 (0x27)
    dload_2,                                    //  40 (0x28)
    dload_3,                                    //  41 (0x29)
    dmul,                                       // 107 (0x6b)
    dneg,                                       // 119 (0x77)
    drem,                                       // 115 (0x73)
    dreturn,                                    // 175 (0xaf)
    dstore(u8),                                 //  57 (0x39)
    dstore_0,                                   //  71 (0x47)
    dstore_1,                                   //  72 (0x48)
    dstore_2,                                   //  73 (0x49)
    dstore_3,                                   //  74 (0x4a)
    dsub,                                       // 103 (0x67)
    dup,                                        //  89 (0x59)
    dup_x1,                                     //  90 (0x5a)
    dup_x2,                                     //  91 (0x5b)
    dup2,                                       //  92 (0x5c)
    dup2_x1,                                    //  93 (0x5d)
    dup2_x2,                                    //  94 (0x5e)
    f2d,                                        // 141 (0x8d)
    f2i,                                        // 193 (0x8b)
    f2l,                                        // 140 (0x8c)
    fadd,                                       //  98 (0x62)
    faload,                                     //  48 (0x30)
    fastore,                                    //  81 (0x51)
    fcmpg,                                      // 150 (0x96)
    fcmpl,                                      // 149 (0x95)
    fconst_0,                                   //  11 (0x0b)
    fconst_1,                                   //  12 (0x0c)
    fconst_2,                                   //  13 (0x0d)
    fdiv,                                       // 110 (0x6e)
    fload(u8),                                  //  23 (0x17)
    fload_0,                                    //  34 (0x22)
    fload_1,                                    //  35 (0x23)
    fload_2,                                    //  36 (0x24)
    fload_3,                                    //  37 (0x25)
    fmul,                                       // 106 (0x6a)
    fneg,                                       // 118 (0x76)
    frem,                                       // 114 (0x72)
    freturn,                                    // 174 (0xae)
    fstore(u8),                                 //  56 (0x38)
    fstore_0,                                   //  67 (0x43)
    fstore_1,                                   //  68 (0x44)
    fstore_2,                                   //  69 (0x45)
    fstore_3,                                   //  70 (0x46)
    fsub,                                       // 102 (0x66)
    getfield(u16),                              // 180 (0xb4)
    getstatic(u16),                             // 178 (0xb2)
    goto(i16),                                  // 167 (0xa7)
    goto_w(i32),                                // 200 (0xc8)
    i2b,                                        // 145 (0x91)
    i2c,                                        // 146 (0x92)
    i2d,                                        // 135 (0x87)
    i2f,                                        // 134 (0x86)
    i2l,                                        // 133 (0x85)
    i2s,                                        // 147 (0x93)
    iadd,                                       //  96 (0x60)
    iaload,                                     //  46 (0x2e)
    iand,                                       // 126 (0x7e)
    iastore,                                    //  79 (0x4f)
    iconst_m1,                                  //   2 (0x02)
    iconst_0,                                   //   3 (0x03)
    iconst_1,                                   //   4 (0x04)
    iconst_2,                                   //   5 (0x05)
    iconst_3,                                   //   6 (0x06)
    iconst_4,                                   //   7 (0x07)
    iconst_5,                                   //   8 (0x08)
    idiv,                                       // 108 (0x6c)
    if_acmpeq(i16),                             // 165 (0xa5)
    if_acmpne(i16),                             // 166 (0xa6)
    if_icmpeq(i16),                             // 159 (0x9f)
    if_icmpne(i16),                             // 160 (0xa0)
    if_icmplt(i16),                             // 161 (0xa1)
    if_icmpge(i16),                             // 162 (0xa2)
    if_icmpgt(i16),                             // 163 (0xa3)
    if_icmple(i16),                             // 164 (0xa4)
    ifeq(i16),                                  // 153 (0x99)
    ifne(i16),                                  // 154 (0x9a)
    iflt(i16),                                  // 155 (0x9b)
    ifge(i16),                                  // 156 (0x9c)
    ifgt(i16),                                  // 157 (0x9d)
    ifle(i16),                                  // 158 (0x9e)
    ifnonnull(i16),                             // 199 (0xc7)
    ifnull(i16),                                // 198 (0xc6)
    iinc(u8, i8),                               // 132 (0x84)
    iload(u8),                                  //  21 (0x15)
    iload_0,                                    //  26 (0x1a)
    iload_1,                                    //  27 (0x1b)
    iload_2,                                    //  28 (0x1c)
    iload_3,                                    //  29 (0x1d)
    imul,                                       // 104 (0x68)
    ineg,                                       // 116 (0x74)
    instanceof(u16),                            // 193 (0xc1)
    invokedynamic(u16),                         // 186 (0xba)
    invokeinterface(u16, u8),                   // 185 (0xb9)
    invokespecial(u16),                         // 183 (0xb7)
    invokestatic(u16),                          // 184 (0xb8)
    invokevirtual(u16),                         // 182 (0xb6)
    ior,                                        // 128 (0x80)
    irem,                                       // 112 (0x70)
    ireturn,                                    // 172 (0xac)
    ishl,                                       // 120 (0x78)
    ishr,                                       // 122 (0x7a)
    istore(u8),                                 //  54 (0x36)
    istore_0,                                   //  59 (0x3b)
    istore_1,                                   //  60 (0x3c)
    istore_2,                                   //  61 (0x3d)
    istore_3,                                   //  62 (0x3e)
    isub,                                       // 100 (0x64)
    iushr,                                      // 124 (0x7c)
    ixor,                                       // 130 (0x82)
    jsr(i16),                                   // 168 (0xa8)
    jsr_w(i32),                                 // 201 (0xc9)
    l2d,                                        // 138 (0x8a)
    l2f,                                        // 137 (0x89)
    l2i,                                        // 136 (0x88)
    ladd,                                       //  97 (0x61)
    laload,                                     //  47 (0x2f)
    land,                                       // 127 (0x7f)
    lastore,                                    //  80 (0x50)
    lcmp,                                       // 148 (0x94)
    lconst_0,                                   //   9 (0x09)
    lconst_1,                                   //  10 (0x0a)
    ldc(u8),                                    //  18 (0x12)
    ldc_w(u16),                                 //  19 (0x13)
    ldc2_w(u16),                                //  20 (0x14)
    ldiv,                                       // 109 (0x6d)
    lload(u8),                                  //  22 (0x16)
    lload_0,                                    //  30 (0x1e)
    lload_1,                                    //  31 (0x1f)
    lload_2,                                    //  32 (0x20)
    lload_3,                                    //  33 (0x21)
    lmul,                                       // 105 (0x69)
    lneg,                                       // 117 (0x75)
    lookupswitch(i32, Box<[(i32, i32)]>),       // 171 (0xab)
    lor,                                        // 129 (0x81)
    lrem,                                       // 113 (0x71)
    lreturn,                                    // 173 (0xad)
    lshl,                                       // 121 (0x79)
    lshr,                                       // 123 (0x7b)
    lstore(u8),                                 //  55 (0x37)
    lstore_0,                                   //  63 (0x3f)
    lstore_1,                                   //  64 (0x40)
    lstore_2,                                   //  65 (0x41)
    lstore_3,                                   //  66 (0x42)
    lsub,                                       // 101 (0x65)
    lushr,                                      // 125 (0x7d)
    lxor,                                       // 131 (0x83)
    monitorenter,                               // 194 (0xc2)
    monitorexit,                                // 195 (0xc3)
    multianewarray(u16, u8),                    // 197 (0xc5)
    new(u16),                                   // 187 (0xbb)
    newarray(ArrayType),                        // 188 (0xbc)
    nop,                                        //   0 (0x00)
    pop,                                        //  87 (0x57)
    pop2,                                       //  88 (0x58)
    putfield(u16),                              // 181 (0xb5)
    putstatic(u16),                             // 179 (0xb3)
    ret(u8),                                    // 169 (0xa9)
    return_,                                    // 177 (0xb1)
    saload,                                     //  53 (0x35)
    sastore,                                    //  86 (0x56)
    sipush(i16),                                //  17 (0x11)
    swap,                                       //  95 (0x5f)
    tableswitch(i32, i32, Box<[i32]>),          // 170 (0xaa)
    iload_w(u16),                               // 196 (0xc4)
    fload_w(u16),                               // 196 (0xc4)
    aload_w(u16),                               // 196 (0xc4)
    lload_w(u16),                               // 196 (0xc4)
    dload_w(u16),                               // 196 (0xc4)
    istore_w(u16),                              // 196 (0xc4)
    fstore_w(u16),                              // 196 (0xc4)
    astore_w(u16),                              // 196 (0xc4)
    lstore_w(u16),                              // 196 (0xc4)
    dstore_w(u16),                              // 196 (0xc4)
    ret_w(u16),                                 // 196 (0xc4)
    iinc_w(u16, i16)                            // 196 (0xc4)
}

#[derive(Debug)]
pub enum ArrayType {
    Boolean,    //  4
    Char,       //  5
    Float,      //  6
    Double,     //  7
    Byte,       //  8
    Short,      //  9
    Int,        // 10
    Long        // 11
}

impl ConstantPoolInfo {
    pub fn is_double_length(self: &ConstantPoolInfo) -> bool {
        match *self {
            ConstantPoolInfo::Long(_) | ConstantPoolInfo::Double(_) => { true },
            _ => { false }
        }
    }
}
