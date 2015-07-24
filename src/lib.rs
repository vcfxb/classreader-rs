#![feature(convert)]

pub use self::model::*;

mod model;

use std::io::Read;
use std::fs::File;

pub struct ClassReader<'a> {
    reader: Box<Read + 'a>
}

impl<'a> ClassReader<'a> {

    pub fn new_from_path(path: &str) -> Result<Class, ()> {
        let mut file = match File::open(path) {
            Result::Ok(f) => { f },
            Result::Err(_) => { panic!("blah") }
        };
        ClassReader::new_from_reader(&mut file)
    }

    pub fn new_from_reader<T: Read + 'a>(reader: &mut T) -> Result<Class, ()> {
        let mut cr = ClassReader { reader: Box::new(reader) };

        let magic = cr.read_u32();
        let minor_version = cr.read_u16();
        let major_version = cr.read_u16();
        let constant_pool = cr.read_constant_pool();
        let access_flags = cr.read_u16();
        let this_class = cr.read_u16();
        let super_class = cr.read_u16();
        let interfaces = cr.read_interfaces();
        let fields = cr.read_fields(&constant_pool);
        let methods = cr.read_methods(&constant_pool);
        let attributes = cr.read_attributes(&constant_pool);

        Result::Ok(Class {
            magic: magic,
            major_version: major_version,
            minor_version: minor_version,
            constant_pool: constant_pool,
            access_flags: access_flags,
            this_class: this_class,
            super_class: super_class,
            interfaces: interfaces,
            fields: fields,
            methods: methods,
            attributes: attributes
        })
    }

    fn read_attribute(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Attribute {
        let name_index = self.read_u16();
        let length = self.read_u32();

        let name = match &constant_pool[name_index as usize - 1] {
            &ConstantPoolInfo::Utf8(ref name) => { name },
            i @ _ => { panic!("expected utf8 at index {} but got {:?}", name_index, i) }
        };

        match name.as_str() {
            "Code" => {
                let max_stack = self.read_u16();
                let max_locals = self.read_u16();
                let code_length = self.read_u32();
                let code = self.read_bytes(code_length);
                let exception_table_length = self.read_u16();
                let mut exceptions = Vec::with_capacity(exception_table_length as usize);
                for _ in 0..exception_table_length {
                    let start_pc = self.read_u16();
                    let end_pc = self.read_u16();
                    let handler_pc = self.read_u16();
                    let catch_type = self.read_u16();

                    let exception = Exception {
                        start_pc: start_pc,
                        end_pc: end_pc,
                        handler_pc: handler_pc,
                        catch_type: catch_type
                    };
                    exceptions.push(exception);
                }
                let attributes = self.read_attributes(constant_pool);

                Attribute::Code {
                    max_stack: max_stack,
                    max_locals: max_locals,
                    code: code,
                    exception_table: exceptions,
                    attributes: attributes
                }
            },
            "ConstantValue" => {
                let constvalue_index = self.read_u16();
                Attribute::ConstantValue { constvalue_index: constvalue_index }
            },
            "StackMapTable" => {
                let number_of_entries = self.read_u16();
                let mut entries = Vec::with_capacity(number_of_entries as usize);
                for _ in 0..number_of_entries {
                    let frame_type = self.read_u8();
                    let frame = match frame_type {
                        0...63 => StackMapFrame::SameFrame,
                        64...127 => {
                            let info = self.read_verification_type_info();
                            StackMapFrame::SameLocals1StackItemFrame(info)
                        },
                        128...246 => panic!(format!("reserved frame type {} used", frame_type)),
                        247 => {
                            let offset_delta = self.read_u16();
                            let info = self.read_verification_type_info();
                            StackMapFrame::SameLocals1StackItemFrameExtended {
                                offset_delta: offset_delta,
                                stack: info
                            }
                        },
                        248...250 => {
                            let offset_delta = self.read_u16();
                            StackMapFrame::ChopFrame { offset_delta: offset_delta }
                        },
                        251 => {
                            let offset_delta = self.read_u16();
                            StackMapFrame::SameFrameExtended { offset_delta: offset_delta }
                        },
                        252...254 => {
                            let offset_delta = self.read_u16();
                            let infos = self.read_verification_type_infos(frame_type as u16 - 251);
                            StackMapFrame::AppendFrame {
                                offset_delta: offset_delta,
                                locals: infos
                            }
                        },
                        255 => {
                            let offset_delta = self.read_u16();
                            let number_of_locals = self.read_u16();
                            let locals = self.read_verification_type_infos(number_of_locals);
                            let number_of_stack_items = self.read_u16();
                            let stack = self.read_verification_type_infos(number_of_stack_items);
                            StackMapFrame::FullFrame {
                                offset_delta: offset_delta,
                                locals: locals,
                                stack: stack
                            }
                        },
                        _ => panic!(format!("unknown frame type {}", frame_type)) // impossible
                    };
                    entries.push(frame);
                }
                Attribute::StackMapTable(entries)
            },
            "Exceptions" => {
                let number_of_exceptions = self.read_u16();
                let mut exception_index_table = Vec::with_capacity(number_of_exceptions as usize);
                for _ in 0..number_of_exceptions {
                    let exception_index = self.read_u16();
                    exception_index_table.push(exception_index);
                }
                Attribute::Exceptions { exception_index_table: exception_index_table }
            },
            "InnerClasses" => {
                let number_of_classes = self.read_u16();
                let mut classes = Vec::with_capacity(number_of_classes as usize);
                for _ in 0..number_of_classes {
                    let inner_class_info_index = self.read_u16();
                    let outer_class_info_index = self.read_u16();
                    let inner_name_index = self.read_u16();
                    let inner_class_access_flags = self.read_u16();

                    let class = InnerClass {
                        inner_class_info_index: inner_class_info_index,
                        outer_class_info_index: outer_class_info_index,
                        inner_name_index: inner_name_index,
                        inner_class_access_flags: inner_class_access_flags
                    };
                    classes.push(class);
                }
                Attribute::InnerClasses {
                    classes: classes
                }
            },
            "EnclosingMethod" => {
                let class_index = self.read_u16();
                let method_index = self.read_u16();
                Attribute::EnclosingMethod {
                    class_index: class_index,
                    method_index: method_index
                }
            },
            "Synthetic" => {
                Attribute::Synthetic
            },
            "Signature" => {
                let signature_index = self.read_u16();
                Attribute::Signature { signature_index: signature_index }
            },
            "SourceFile" => {
                let sourcefile_index = self.read_u16();
                Attribute::SourceFile { sourcefile_index: sourcefile_index }
            },
            "SourceDebugExtension" => {
                let data = self.read_bytes(length);
                Attribute::SourceDebugExtension(data)
            },
            "LineNumberTable" => {
                let table_length = self.read_u16();
                let mut entries = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = self.read_u16();
                    let line_number = self.read_u16();
                    let entry = LineNumber {
                        start_pc: start_pc,
                        line_number: line_number
                    };
                    entries.push(entry);
                }
                Attribute::LineNumberTable(entries)
            },
            "LocalVariableTable" | "LocalVariableTypeTable" => {
                let table_length = self.read_u16();
                let mut entries = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = self.read_u16();
                    let length = self.read_u16();
                    let name_index = self.read_u16();
                    let descriptor_or_signature_index = self.read_u16();
                    let index = self.read_u16();
                    let entry = LocalVariable {
                        start_pc: start_pc,
                        length: length,
                        name_index: name_index,
                        descriptor_or_signature_index: descriptor_or_signature_index,
                        index: index
                    };
                    entries.push(entry);
                }
                if name == "LocalVariableTable" {
                    Attribute::LocalVariableTable(entries)
                } else {
                    Attribute::LocalVariableTypeTable(entries)
                }
            },
            "Deprecated" => {
                Attribute::Deprecated
            },
            "RuntimeVisibleAnnotations" => {
                let annotations = self.read_annotations(constant_pool);
                Attribute::RuntimeVisibleAnnotations(annotations)
            },
            "RuntimeInvisibleAnnotations" => {
                let annotations = self.read_annotations(constant_pool);
                Attribute::RuntimeInvisibleAnnotations(annotations)
            },
            "RuntimeVisibleParameterAnnotations" => {
                let parameter_annotations = self.read_parameter_annotations(constant_pool);
                Attribute::RuntimeVisibleParameterAnnotations(parameter_annotations)
            },
            "RuntimeInvisibleParameterAnnotations" => {
                let parameter_annotations = self.read_parameter_annotations(constant_pool);
                Attribute::RuntimeInvisibleParameterAnnotations(parameter_annotations)
            },
            "RuntimeVisibleTypeAnnotations" => {
                let type_annotations = self.read_type_annotations(constant_pool);
                Attribute::RuntimeVisibleTypeAnnotations(type_annotations)
            },
            "RuntimeInvisibleTypeAnnotations" => {
                let type_annotations = self.read_type_annotations(constant_pool);
                Attribute::RuntimeInvisibleTypeAnnotations(type_annotations)
            },
            "AnnotationDefault" => {
                let element_value = self.read_element_value(constant_pool);
                Attribute::AnnotationDefault { element_value: element_value }
            },
            "BootstrapMethods" => {
                let num_bootstrap_methods = self.read_u16();
                let mut bootstrap_methods = Vec::with_capacity(num_bootstrap_methods as usize);
                for _ in 0..num_bootstrap_methods {
                    let bootstrap_method_ref = self.read_u16();
                    let num_bootstrap_arguments = self.read_u16();
                    let mut bootstrap_arguments = Vec::with_capacity(num_bootstrap_arguments as usize);
                    for _ in 0..num_bootstrap_arguments {
                        let bootstrap_argument = self.read_u16();
                        bootstrap_arguments.push(bootstrap_argument);
                    }
                    let bootstrap_method = BootstrapMethod {
                        method_ref: bootstrap_method_ref,
                        arguments: bootstrap_arguments
                    };
                    bootstrap_methods.push(bootstrap_method);
                }
                Attribute::BootstrapMethods(bootstrap_methods)
            },
            "MethodParameters" => {
                let num_parameters = self.read_u8();
                let mut parameters = Vec::with_capacity(num_parameters as usize);
                for _ in 0..num_parameters {
                    let name_index = self.read_u16();
                    let access_flags = self.read_u16();
                    let parameter = MethodParameter {
                        name_index: name_index,
                        access_flags: access_flags
                    };
                    parameters.push(parameter);
                }
                Attribute::MethodParameters(parameters)
            },
            _ => {
                let info = self.read_bytes(length);
                Attribute::Unknown(info)
            }
        }
    }

    fn read_verification_type_info(self: &mut ClassReader<'a>) -> VerificationType {
        let tag = self.read_u8();
        match tag {
            0 => VerificationType::Top,
            1 => VerificationType::Integer,
            2 => VerificationType::Float,
            3 => VerificationType::Double,
            4 => VerificationType::Long,
            5 => VerificationType::Null,
            6 => VerificationType::UninitializedThis,
            7 => {
                let index = self.read_u16();
                VerificationType::Object { index: index }
            }
            8 => {
                let offset = self.read_u16();
                VerificationType::UninitializedVariable { offset: offset }
            }
            _ => panic!(format!("unknown verification type info tag {}", tag))
        }
    }

    fn read_verification_type_infos(self: &mut ClassReader<'a>, num: u16) -> Vec<VerificationType> {
        let mut infos = Vec::with_capacity(num as usize);
        for _ in 0..num {
            let info = self.read_verification_type_info();
            infos.push(info);
        }
        infos
    }

    fn read_type_annotations(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<TypeAnnotation> {
        let num_annotations = self.read_u16();
        let mut annotations = Vec::with_capacity(num_annotations as usize);
        for _ in 0..num_annotations {
            let annotation = self.read_type_annotation(constant_pool);
            annotations.push(annotation);
        }
        annotations
    }

    fn read_type_annotation(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> TypeAnnotation {
        let target_type_tag = self.read_u8();
        let target_type = match target_type_tag {
            0x00 => TargetType::Type,
            0x01 => TargetType::Method,
            0x10 => TargetType::Supertype,
            0x11 => TargetType::TypeBound,
            0x12 => TargetType::MethodBound,
            0x13 => TargetType::Field,
            0x14 => TargetType::MethodReturnType,
            0x15 => TargetType::ReceiverType,
            0x16 => TargetType::Parameter,
            0x17 => TargetType::Throws,
            0x40 => TargetType::LocalVariableDeclaration,
            0x41 => TargetType::ResourceVariableDeclaration,
            0x42 => TargetType::ExceptionParameterDeclaration,
            0x43 => TargetType::Instanceof,
            0x44 => TargetType::New,
            0x45 => TargetType::MethodReferenceNew,
            0x46 => TargetType::MethodReference,
            0x47 => TargetType::Cast,
            0x48 => TargetType::ConstructorArgument,
            0x49 => TargetType::MethodArgument,
            0x4A => TargetType::MethodReferenceNewArgument,
            0x4B => TargetType::MethodReferenceArgument,
            _ => panic!("unknown target type {}", target_type_tag)
        };
        let target_info = match target_type_tag {
            0x00 | 0x01 => {
                let type_parameter_index = self.read_u8();
                TargetInfo::TypeParameter { index: type_parameter_index }
            },
            0x10 => {
                let supertype_index = self.read_u16();
                TargetInfo::Supertype { index: supertype_index }
            },
            0x11 | 0x12 => {
                let type_parameter_index = self.read_u8();
                let bound_index = self.read_u8();
                TargetInfo::TypeParameterBound { index: type_parameter_index, bound_index: bound_index }
            },
            0x13...0x15 => {
                TargetInfo::Empty
            },
            0x16 => {
                let formal_parameter_index = self.read_u8();
                TargetInfo::MethodFormalParameter { index: formal_parameter_index }
            },
            0x17 => {
                let throws_type_index = self.read_u16();
                TargetInfo::Throws { type_index: throws_type_index }
            },
            0x40 | 0x41 => {
                let table_length = self.read_u16();
                let mut table = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = self.read_u16();
                    let length = self.read_u16();
                    let index = self.read_u16();
                    let entry = LocalVariableTarget {
                        start_pc: start_pc,
                        length: length,
                        index: index
                    };
                    table.push(entry);
                }
                TargetInfo::Localvar(table)
            },
            0x42 => {
                let exception_table_index = self.read_u16();
                TargetInfo::Catch { exception_table_index : exception_table_index }
            },
            0x43...0x46 => {
                let offset = self.read_u16();
                TargetInfo::Offset(offset)
            },
            0x47...0x4B => {
                let offset = self.read_u16();
                let type_argument_index = self.read_u8();
                TargetInfo::TypeArgument { offset: offset, index: type_argument_index }
            },
            _ => panic!("unknown target type {}", target_type_tag)
        };
        let path_length = self.read_u8();
        let mut path = Vec::with_capacity(path_length as usize);
        for _ in 0..path_length {
            let type_path_kind_tag = self.read_u8();
            let type_path_kind = match type_path_kind_tag {
                0 => TypePathKind::Array,
                1 => TypePathKind::Nested,
                2 => TypePathKind::WildcardBound,
                3 => TypePathKind::TypeArgument,
                _ => panic!(format!("unknown type path kind {}", type_path_kind_tag))
            };
            let type_argument_index = self.read_u8();
            let path_element = PathElement { kind: type_path_kind, argument_index: type_argument_index };
            path.push(path_element);
        }
        let type_index = self.read_u16();
        let element_value_pairs = self.read_element_value_pairs(constant_pool);
        TypeAnnotation {
            target_type: target_type,
            target_info: target_info,
            type_path: TypePath { path: path },
            type_index: type_index,
            element_value_pairs: element_value_pairs
        }
    }

    fn read_parameter_annotations(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<Vec<Annotation>> {
        let num_parameters = self.read_u8();
        let mut parameter_annotations = Vec::with_capacity(num_parameters as usize);
        for _ in 0..num_parameters {
            let annotations = self.read_annotations(constant_pool);
            parameter_annotations.push(annotations);
        }
        parameter_annotations
    }

    fn read_annotations(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<Annotation> {
        let num_annotations = self.read_u16();
        let mut annotations = Vec::with_capacity(num_annotations as usize);
        for _ in 0..num_annotations {
            let annotation = self.read_annotation(constant_pool);
            annotations.push(annotation);
        }
        annotations
    }

    fn read_annotation(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Annotation {
        let type_index = self.read_u16();
        let element_value_pairs = self.read_element_value_pairs(constant_pool);
        Annotation {
            type_index: type_index,
            element_value_pairs: element_value_pairs
        }
    }

    fn read_element_value_pairs(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<ElementValuePair> {
        let num_evps = self.read_u16();
        let mut element_value_pairs = Vec::with_capacity(num_evps as usize);
        for _ in 0..num_evps {
            let element_name_index = self.read_u16();
            let element_value = self.read_element_value(constant_pool);

            let element_value_pair = ElementValuePair {
                element_name_index: element_name_index,
                value: element_value
            };
            element_value_pairs.push(element_value_pair);
        }
        element_value_pairs
    }

    fn read_element_value(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ElementValue {
        let tag = self.read_u8() as char;
        match tag {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                let const_value_index = self.read_u16();
                ElementValue::Constant { const_value_index: const_value_index }
            },
            'e' => {
                let type_name_index = self.read_u16();
                let const_name_index = self.read_u16();
                ElementValue::EnumConstant {
                    type_name_index: type_name_index,
                    const_name_index: const_name_index
                }
            },
            'c' => {
                let class_info_index = self.read_u16();
                ElementValue::Class { class_info_index: class_info_index }
            },
            '@' => {
                let annotation = self.read_annotation(constant_pool);
                ElementValue::Annotation(annotation)
            },
            '[' => {
                let num_values = self.read_u16();
                let mut element_values = Vec::with_capacity(num_values as usize);
                for _ in 0..num_values {
                    element_values.push(self.read_element_value(constant_pool));
                }
                ElementValue::Array(element_values)
            },
            _ => {
                panic!("unknown element value tag {}", tag);
            }
        }
    }

    fn read_attributes(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<Attribute> {
        let attribute_count = self.read_u16();
        let mut attributes = Vec::with_capacity(attribute_count as usize);
        for _ in 0..attribute_count {
            let attribute = self.read_attribute(constant_pool);
            attributes.push(attribute);
        }
        attributes
    }

    fn read_methods(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<Method> {
        let method_count = self.read_u16();
        let mut methods = Vec::with_capacity(method_count as usize);
        for _ in 0..method_count {
            let access_flags = self.read_u16();
            let name_index = self.read_u16();
            let descriptor_index = self.read_u16();
            let attributes = self.read_attributes(&constant_pool);

            let method = Method {
                access_flags: access_flags,
                name_index: name_index,
                descriptor_index: descriptor_index,
                attributes: attributes
            };
            methods.push(method);
        }
        methods
    }

    fn read_fields(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> Vec<Field> {
        let field_count = self.read_u16();
        let mut fields = Vec::with_capacity(field_count as usize);
        for _ in 0..field_count {
            let access_flags = self.read_u16();
            let name_index = self.read_u16();
            let descriptor_index = self.read_u16();
            let attributes = self.read_attributes(&constant_pool);

            let field = Field {
                access_flags: access_flags,
                name_index: name_index,
                descriptor_index:
                descriptor_index,
                attributes: attributes };
            fields.push(field);
        }
        fields
    }

    fn read_interfaces(self: &mut ClassReader<'a>) -> Vec<u16> {
        let interfaces_count = self.read_u16();
        let mut interfaces = Vec::with_capacity(interfaces_count as usize);
        for _ in 0..interfaces_count {
            let interface = self.read_u16();
            interfaces.push(interface);
        }
        interfaces
    }

    fn read_constant_pool(self: &mut ClassReader<'a>) -> Vec<ConstantPoolInfo> {
        let cp_count = self.read_u16() - 1;
        let mut cp: Vec<ConstantPoolInfo> = Vec::with_capacity(cp_count as usize);

        let mut i = 0;
        while i < cp_count {
            let cp_info = self.read_constant_pool_info();
            let is_double_length = cp_info.is_double_length();

            cp.push(cp_info);
            if is_double_length {
                cp.push(ConstantPoolInfo::Invalid);
                i += 1;
            }

            i += 1;
        }

        cp
    }

    fn read_constant_pool_info(self: &mut ClassReader<'a>) -> ConstantPoolInfo {
        let tag = self.read_u8();
        match tag {
            1 => {
                let length = self.read_u16();
                let data = self.read_bytes(length as u32);
                let string = String::from_utf8(data).unwrap();
                ConstantPoolInfo::Utf8(string)
            },
            3 => {
                let value = self.read_u32() as i32;
                ConstantPoolInfo::Integer(value)
            },
            4 => {
                let value = self.read_u32() as f32;
                ConstantPoolInfo::Float(value)
            },
            5 => {
                let value = self.read_u64() as i64;
                ConstantPoolInfo::Long(value)
            },
            6 => {
                let value = self.read_u64() as f64;
                ConstantPoolInfo::Double(value)
            },
            7 => {
                let name_index = self.read_u16();
                ConstantPoolInfo::Class(name_index)
            },
            8 => {
                let string_index = self.read_u16();
                ConstantPoolInfo::String(string_index)
            },
            9 => {
                let class_index = self.read_u16();
                let name_and_type_index = self.read_u16();
                ConstantPoolInfo::Fieldref(class_index, name_and_type_index)
            },
            10 => {
                let class_index = self.read_u16();
                let name_and_type_index = self.read_u16();
                ConstantPoolInfo::Methodref(class_index, name_and_type_index)
            },
            11 => {
                let class_index = self.read_u16();
                let name_and_type_index = self.read_u16();
                ConstantPoolInfo::InterfaceMethodref(class_index, name_and_type_index)
            },
            12 => {
                let name_index = self.read_u16();
                let descriptor_index = self.read_u16();
                ConstantPoolInfo::NameAndType(name_index, descriptor_index)
            },
            15 => {
                let reference_kind = self.read_u8();
                let reference_index = self.read_u16();
                ConstantPoolInfo::MethodHandle(reference_kind, reference_index)
            },
            16 => {
                let descriptor_index = self.read_u16();
                ConstantPoolInfo::MethodType(descriptor_index)
            },
            18 => {
                let bootstrap_method_attr_index = self.read_u16();
                let name_and_type_index = self.read_u16();
                ConstantPoolInfo::InvokeDynamic(bootstrap_method_attr_index, name_and_type_index)
            },
            _ => {
                panic!(format!("unknown constant pool item with tag {}", tag))
            }
        }
    }

    fn read_bytes(self: &mut ClassReader<'a>, length: u32) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity(length as usize);
        match self.reader.by_ref().take(length as u64).read_to_end(&mut vec) {
            Result::Ok(_) => { },
            Result::Err(_) => { panic!("blah") }
        };
        vec
    }

    fn read_u64(self: &mut ClassReader<'a>) -> u64 {
        let mut buf = [0u8; 8];
        match self.reader.by_ref().read(&mut buf) {
            Result::Ok(_) => {},
            Result::Err(_) => { panic!("blah") }
        };
        (buf[0] as u64) << 56 | (buf[1] as u64) << 48
                | (buf[2] as u64) << 40 | (buf[3] as u64) << 32
                | (buf[4] as u64) << 24 | (buf[5] as u64) << 16
                | (buf[6] as u64) << 8 | (buf[7] as u64)
    }

    fn read_u32(self: &mut ClassReader<'a>) -> u32 {
        let mut buf = [0u8; 4];
        match self.reader.by_ref().read(&mut buf) {
            Result::Ok(_) => {},
            Result::Err(_) => { panic!("blah") }
        };
        (buf[0] as u32) << 24 | (buf[1] as u32) << 16
                | (buf[2] as u32) << 8 | (buf[3] as u32)
    }

    fn read_u16(self: &mut ClassReader<'a>) -> u16 {
        let mut buf = [0u8; 2];
        match self.reader.by_ref().read(&mut buf) {
            Result::Ok(_) => {},
            Result::Err(_) => { panic!("blah") }
        };
        (buf[0] as u16) << 8 | (buf[1] as u16)
    }

    fn read_u8(self: &mut ClassReader<'a>) -> u8 {
        let mut buf = [0u8; 1];
        match self.reader.by_ref().read(&mut buf) {
            Result::Ok(_) => {},
            Result::Err(_) => { panic!("blah") }
        };
        buf[0] as u8
    }

}
