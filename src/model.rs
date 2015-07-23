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

#[derive(Debug)]
pub struct Method {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>
}

#[derive(Debug)]
pub enum Attribute {
    ConstantValue{ constvalue_index: u16 },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
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

impl ConstantPoolInfo {
    pub fn is_double_length(self: &ConstantPoolInfo) -> bool {
        match *self {
            ConstantPoolInfo::Long(_) | ConstantPoolInfo::Double(_) => { true },
            _ => { false }
        }
    }
}
