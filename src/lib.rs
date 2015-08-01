#![feature(convert)]

#[macro_use]
extern crate log;

mod model;
mod result;
mod decode;

use std::char;
use std::io::Read;
use std::fs::File;

pub use ::result::*;
pub use ::model::*;
pub use ::decode::*;

pub struct ClassReader<'a> {
    reader: Box<Read + 'a>,
    position: usize
}

impl<'a> ClassReader<'a> {

    pub fn new_from_path(path: &str) -> ParseResult<Class> {
        let mut file = match File::open(path) {
            Result::Ok(f) => f,
            Result::Err(e) => { return Result::Err(ParseError::Io(e)); }
        };
        ClassReader::new_from_reader(&mut file)
    }

    pub fn new_from_reader<T: Read + 'a>(reader: &mut T) -> ParseResult<Class> {
        let mut cr = ClassReader { reader: Box::new(reader), position: 0 };

        let magic = try!(cr.read_u32());
        let minor_version = try!(cr.read_u16());
        let major_version = try!(cr.read_u16());
        let constant_pool = try!(cr.read_constant_pool());
        let access_flags = try!(cr.read_u16());
        let this_class = try!(cr.read_u16());
        let super_class = try!(cr.read_u16());
        let interfaces = try!(cr.read_interfaces());
        let fields = try!(cr.read_fields(&constant_pool));
        let methods = try!(cr.read_methods(&constant_pool));
        let attributes = try!(cr.read_attributes(&constant_pool));

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

    fn read_attribute(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Attribute> {
        let name_index = try!(self.read_u16());
        let length = try!(self.read_u32());

        let name = match &constant_pool[name_index as usize - 1] {
            &ConstantPoolInfo::Utf8(ref name) => { name },
            i @ _ => {
                let error_message = format!("expected utf8 at index {} but got {:?}", name_index, i);
                return Result::Err(ParseError::Format(error_message));
            }
        };

        let info = match name.as_str() {
            "Code" => {
                let max_stack = try!(self.read_u16());
                let max_locals = try!(self.read_u16());
                let code_length = try!(self.read_u32());
                let code = try!(self.read_bytes(code_length));
                let exception_table_length = try!(self.read_u16());
                let mut exceptions = Vec::with_capacity(exception_table_length as usize);
                for _ in 0..exception_table_length {
                    let start_pc = try!(self.read_u16());
                    let end_pc = try!(self.read_u16());
                    let handler_pc = try!(self.read_u16());
                    let catch_type = try!(self.read_u16());

                    let exception = Exception {
                        start_pc: start_pc,
                        end_pc: end_pc,
                        handler_pc: handler_pc,
                        catch_type: catch_type
                    };
                    exceptions.push(exception);
                }
                let attributes = try!(self.read_attributes(constant_pool));

                let instructions = try!(decode_code(&code));

                Attribute::Code {
                    max_stack: max_stack,
                    max_locals: max_locals,
                    code: instructions,
                    exception_table: exceptions,
                    attributes: attributes
                }
            },
            "ConstantValue" => {
                let constvalue_index = try!(self.read_u16());
                Attribute::ConstantValue { constvalue_index: constvalue_index }
            },
            "StackMapTable" => {
                let number_of_entries = try!(self.read_u16());
                let mut entries = Vec::with_capacity(number_of_entries as usize);
                for _ in 0..number_of_entries {
                    let frame_type = try!(self.read_u8());
                    let frame = match frame_type {
                        0...63 => StackMapFrame::SameFrame,
                        64...127 => {
                            let info = try!(self.read_verification_type_info());
                            StackMapFrame::SameLocals1StackItemFrame(info)
                        },
                        128...246 => {
                            let message = format!("reserved frame type {} used", frame_type);
                            return Result::Err(ParseError::Format(message));
                        }
                        247 => {
                            let offset_delta = try!(self.read_u16());
                            let info = try!(self.read_verification_type_info());
                            StackMapFrame::SameLocals1StackItemFrameExtended {
                                offset_delta: offset_delta,
                                stack: info
                            }
                        },
                        248...250 => {
                            let offset_delta = try!(self.read_u16());
                            StackMapFrame::ChopFrame { offset_delta: offset_delta }
                        },
                        251 => {
                            let offset_delta = try!(self.read_u16());
                            StackMapFrame::SameFrameExtended { offset_delta: offset_delta }
                        },
                        252...254 => {
                            let offset_delta = try!(self.read_u16());
                            let infos = try!(self.read_verification_type_infos(frame_type as u16 - 251));
                            StackMapFrame::AppendFrame {
                                offset_delta: offset_delta,
                                locals: infos
                            }
                        },
                        255 => {
                            let offset_delta = try!(self.read_u16());
                            let number_of_locals = try!(self.read_u16());
                            let locals = try!(self.read_verification_type_infos(number_of_locals));
                            let number_of_stack_items = try!(self.read_u16());
                            let stack = try!(self.read_verification_type_infos(number_of_stack_items));
                            StackMapFrame::FullFrame {
                                offset_delta: offset_delta,
                                locals: locals,
                                stack: stack
                            }
                        },
                        _ => panic!("unknown frame type {}", frame_type) // impossible
                    };
                    entries.push(frame);
                }
                Attribute::StackMapTable(entries)
            },
            "Exceptions" => {
                let number_of_exceptions = try!(self.read_u16());
                let mut exception_index_table = Vec::with_capacity(number_of_exceptions as usize);
                for _ in 0..number_of_exceptions {
                    let exception_index = try!(self.read_u16());
                    exception_index_table.push(exception_index);
                }
                Attribute::Exceptions { exception_index_table: exception_index_table }
            },
            "InnerClasses" => {
                let number_of_classes = try!(self.read_u16());
                let mut classes = Vec::with_capacity(number_of_classes as usize);
                for _ in 0..number_of_classes {
                    let inner_class_info_index = try!(self.read_u16());
                    let outer_class_info_index = try!(self.read_u16());
                    let inner_name_index = try!(self.read_u16());
                    let inner_class_access_flags = try!(self.read_u16());

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
                let class_index = try!(self.read_u16());
                let method_index = try!(self.read_u16());
                Attribute::EnclosingMethod {
                    class_index: class_index,
                    method_index: method_index
                }
            },
            "Synthetic" => {
                Attribute::Synthetic
            },
            "Signature" => {
                let signature_index = try!(self.read_u16());
                Attribute::Signature { signature_index: signature_index }
            },
            "SourceFile" => {
                let sourcefile_index = try!(self.read_u16());
                Attribute::SourceFile { sourcefile_index: sourcefile_index }
            },
            "SourceDebugExtension" => {
                let data = try!(self.read_bytes(length));
                Attribute::SourceDebugExtension(data)
            },
            "LineNumberTable" => {
                let table_length = try!(self.read_u16());
                let mut entries = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = try!(self.read_u16());
                    let line_number = try!(self.read_u16());
                    let entry = LineNumber {
                        start_pc: start_pc,
                        line_number: line_number
                    };
                    entries.push(entry);
                }
                Attribute::LineNumberTable(entries)
            },
            "LocalVariableTable" | "LocalVariableTypeTable" => {
                let table_length = try!(self.read_u16());
                let mut entries = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = try!(self.read_u16());
                    let length = try!(self.read_u16());
                    let name_index = try!(self.read_u16());
                    let descriptor_or_signature_index = try!(self.read_u16());
                    let index = try!(self.read_u16());
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
                let annotations = try!(self.read_annotations(constant_pool));
                Attribute::RuntimeVisibleAnnotations(annotations)
            },
            "RuntimeInvisibleAnnotations" => {
                let annotations = try!(self.read_annotations(constant_pool));
                Attribute::RuntimeInvisibleAnnotations(annotations)
            },
            "RuntimeVisibleParameterAnnotations" => {
                let parameter_annotations = try!(self.read_parameter_annotations(constant_pool));
                Attribute::RuntimeVisibleParameterAnnotations(parameter_annotations)
            },
            "RuntimeInvisibleParameterAnnotations" => {
                let parameter_annotations = try!(self.read_parameter_annotations(constant_pool));
                Attribute::RuntimeInvisibleParameterAnnotations(parameter_annotations)
            },
            "RuntimeVisibleTypeAnnotations" => {
                let type_annotations = try!(self.read_type_annotations(constant_pool));
                Attribute::RuntimeVisibleTypeAnnotations(type_annotations)
            },
            "RuntimeInvisibleTypeAnnotations" => {
                let type_annotations = try!(self.read_type_annotations(constant_pool));
                Attribute::RuntimeInvisibleTypeAnnotations(type_annotations)
            },
            "AnnotationDefault" => {
                let element_value = try!(self.read_element_value(constant_pool));
                Attribute::AnnotationDefault { element_value: element_value }
            },
            "BootstrapMethods" => {
                let num_bootstrap_methods = try!(self.read_u16());
                let mut bootstrap_methods = Vec::with_capacity(num_bootstrap_methods as usize);
                for _ in 0..num_bootstrap_methods {
                    let bootstrap_method_ref = try!(self.read_u16());
                    let num_bootstrap_arguments = try!(self.read_u16());
                    let mut bootstrap_arguments = Vec::with_capacity(num_bootstrap_arguments as usize);
                    for _ in 0..num_bootstrap_arguments {
                        let bootstrap_argument = try!(self.read_u16());
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
                let num_parameters = try!(self.read_u8());
                let mut parameters = Vec::with_capacity(num_parameters as usize);
                for _ in 0..num_parameters {
                    let name_index = try!(self.read_u16());
                    let access_flags = try!(self.read_u16());
                    let parameter = MethodParameter {
                        name_index: name_index,
                        access_flags: access_flags
                    };
                    parameters.push(parameter);
                }
                Attribute::MethodParameters(parameters)
            },
            _ => {
                let info = try!(self.read_bytes(length));
                Attribute::Unknown(info)
            }
        };
        Result::Ok(info)
    }

    fn read_verification_type_info(self: &mut ClassReader<'a>) -> ParseResult<VerificationType> {
        let tag = try!(self.read_u8());
        let verification_type_info = match tag {
            0 => VerificationType::Top,
            1 => VerificationType::Integer,
            2 => VerificationType::Float,
            3 => VerificationType::Double,
            4 => VerificationType::Long,
            5 => VerificationType::Null,
            6 => VerificationType::UninitializedThis,
            7 => {
                let index = try!(self.read_u16());
                VerificationType::Object { index: index }
            }
            8 => {
                let offset = try!(self.read_u16());
                VerificationType::UninitializedVariable { offset: offset }
            }
            _ => {
                let error_message = format!("unknown verification type info tag {}", tag);
                return Result::Err(ParseError::Format(error_message));
            }
        };
        Result::Ok(verification_type_info)
    }

    fn read_verification_type_infos(self: &mut ClassReader<'a>, num: u16) -> ParseResult<Vec<VerificationType>> {
        let mut infos = Vec::with_capacity(num as usize);
        for _ in 0..num {
            let info = try!(self.read_verification_type_info());
            infos.push(info);
        }
        Result::Ok(infos)
    }

    fn read_type_annotations(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<TypeAnnotation>> {
        let num_annotations = try!(self.read_u16());
        let mut annotations = Vec::with_capacity(num_annotations as usize);
        for _ in 0..num_annotations {
            let annotation = try!(self.read_type_annotation(constant_pool));
            annotations.push(annotation);
        }
        Result::Ok(annotations)
    }

    fn read_type_annotation(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<TypeAnnotation> {
        let target_type_tag = try!(self.read_u8());
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
            _ => {
                let message = format!("unknown target type {}", target_type_tag);
                return Result::Err(ParseError::Format(message));
            }
        };
        let target_info = match target_type_tag {
            0x00 | 0x01 => {
                let type_parameter_index = try!(self.read_u8());
                TargetInfo::TypeParameter { index: type_parameter_index }
            },
            0x10 => {
                let supertype_index = try!(self.read_u16());
                TargetInfo::Supertype { index: supertype_index }
            },
            0x11 | 0x12 => {
                let type_parameter_index = try!(self.read_u8());
                let bound_index = try!(self.read_u8());
                TargetInfo::TypeParameterBound { index: type_parameter_index, bound_index: bound_index }
            },
            0x13...0x15 => {
                TargetInfo::Empty
            },
            0x16 => {
                let formal_parameter_index = try!(self.read_u8());
                TargetInfo::MethodFormalParameter { index: formal_parameter_index }
            },
            0x17 => {
                let throws_type_index = try!(self.read_u16());
                TargetInfo::Throws { type_index: throws_type_index }
            },
            0x40 | 0x41 => {
                let table_length = try!(self.read_u16());
                let mut table = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = try!(self.read_u16());
                    let length = try!(self.read_u16());
                    let index = try!(self.read_u16());
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
                let exception_table_index = try!(self.read_u16());
                TargetInfo::Catch { exception_table_index : exception_table_index }
            },
            0x43...0x46 => {
                let offset = try!(self.read_u16());
                TargetInfo::Offset(offset)
            },
            0x47...0x4B => {
                let offset = try!(self.read_u16());
                let type_argument_index = try!(self.read_u8());
                TargetInfo::TypeArgument { offset: offset, index: type_argument_index }
            },
            _ => {
                let error_message = format!("unknown target type {}", target_type_tag);
                return Result::Err(ParseError::Format(error_message));
            }
        };
        let path_length = try!(self.read_u8());
        let mut path = Vec::with_capacity(path_length as usize);
        for _ in 0..path_length {
            let type_path_kind_tag = try!(self.read_u8());
            let type_path_kind = match type_path_kind_tag {
                0 => TypePathKind::Array,
                1 => TypePathKind::Nested,
                2 => TypePathKind::WildcardBound,
                3 => TypePathKind::TypeArgument,
                _ => {
                    let error_message = format!("unknown type path kind {}", type_path_kind_tag);
                    return Result::Err(ParseError::Format(error_message));
                }
            };
            let type_argument_index = try!(self.read_u8());
            let path_element = PathElement { kind: type_path_kind, argument_index: type_argument_index };
            path.push(path_element);
        }
        let type_index = try!(self.read_u16());
        let element_value_pairs = try!(self.read_element_value_pairs(constant_pool));
        Result::Ok(TypeAnnotation {
            target_type: target_type,
            target_info: target_info,
            type_path: TypePath { path: path },
            type_index: type_index,
            element_value_pairs: element_value_pairs
        })
    }

    fn read_parameter_annotations(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<Vec<Annotation>>> {
        let num_parameters = try!(self.read_u8());
        let mut parameter_annotations = Vec::with_capacity(num_parameters as usize);
        for _ in 0..num_parameters {
            let annotations = try!(self.read_annotations(constant_pool));
            parameter_annotations.push(annotations);
        }
        Result::Ok(parameter_annotations)
    }

    fn read_annotations(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<Annotation>> {
        let num_annotations = try!(self.read_u16());
        let mut annotations = Vec::with_capacity(num_annotations as usize);
        for _ in 0..num_annotations {
            let annotation = try!(self.read_annotation(constant_pool));
            annotations.push(annotation);
        }
        Result::Ok(annotations)
    }

    fn read_annotation(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Annotation> {
        let type_index = try!(self.read_u16());
        let element_value_pairs = try!(self.read_element_value_pairs(constant_pool));
        Result::Ok(Annotation {
            type_index: type_index,
            element_value_pairs: element_value_pairs
        })
    }

    fn read_element_value_pairs(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<ElementValuePair>> {
        let num_evps = try!(self.read_u16());
        let mut element_value_pairs = Vec::with_capacity(num_evps as usize);
        for _ in 0..num_evps {
            let element_name_index = try!(self.read_u16());
            let element_value = try!(self.read_element_value(constant_pool));

            let element_value_pair = ElementValuePair {
                element_name_index: element_name_index,
                value: element_value
            };
            element_value_pairs.push(element_value_pair);
        }
        Result::Ok(element_value_pairs)
    }

    fn read_element_value(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<ElementValue> {
        let tag = try!(self.read_u8()) as char;
        let value = match tag {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                let const_value_index = try!(self.read_u16());
                ElementValue::Constant { const_value_index: const_value_index }
            },
            'e' => {
                let type_name_index = try!(self.read_u16());
                let const_name_index = try!(self.read_u16());
                ElementValue::EnumConstant {
                    type_name_index: type_name_index,
                    const_name_index: const_name_index
                }
            },
            'c' => {
                let class_info_index = try!(self.read_u16());
                ElementValue::Class { class_info_index: class_info_index }
            },
            '@' => {
                let annotation = try!(self.read_annotation(constant_pool));
                ElementValue::Annotation(annotation)
            },
            '[' => {
                let num_values = try!(self.read_u16());
                let mut element_values = Vec::with_capacity(num_values as usize);
                for _ in 0..num_values {
                    let array_value = try!(self.read_element_value(constant_pool));
                    element_values.push(array_value);
                }
                ElementValue::Array(element_values)
            },
            _ => {
                let message = format!("unknown element value tag {}", tag);
                return Result::Err(ParseError::Format(message));
            }
        };
        Result::Ok(value)
    }

    fn read_attributes(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<Attribute>> {
        let attribute_count = try!(self.read_u16());
        let mut attributes = Vec::with_capacity(attribute_count as usize);
        for _ in 0..attribute_count {
            let attribute = try!(self.read_attribute(constant_pool));
            attributes.push(attribute);
        }
        Result::Ok(attributes)
    }

    fn read_methods(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<Method>> {
        let method_count = try!(self.read_u16());
        let mut methods = Vec::with_capacity(method_count as usize);
        for _ in 0..method_count {
            let access_flags = try!(self.read_u16());
            let name_index = try!(self.read_u16());
            let descriptor_index = try!(self.read_u16());
            let attributes = try!(self.read_attributes(&constant_pool));

            let method = Method {
                access_flags: access_flags,
                name_index: name_index,
                descriptor_index: descriptor_index,
                attributes: attributes
            };
            methods.push(method);
        }
        Result::Ok(methods)
    }

    fn read_fields(self: &mut ClassReader<'a>, constant_pool: &Vec<ConstantPoolInfo>) -> ParseResult<Vec<Field>> {
        let field_count = try!(self.read_u16());
        let mut fields = Vec::with_capacity(field_count as usize);
        for _ in 0..field_count {
            let access_flags = try!(self.read_u16());
            let name_index = try!(self.read_u16());
            let descriptor_index = try!(self.read_u16());
            let attributes = try!(self.read_attributes(&constant_pool));

            let field = Field {
                access_flags: access_flags,
                name_index: name_index,
                descriptor_index:
                descriptor_index,
                attributes: attributes };
            fields.push(field);
        }
        Result::Ok(fields)
    }

    fn read_interfaces(self: &mut ClassReader<'a>) -> ParseResult<Vec<u16>> {
        let interfaces_count = try!(self.read_u16());
        let mut interfaces = Vec::with_capacity(interfaces_count as usize);
        for _ in 0..interfaces_count {
            let interface = try!(self.read_u16());
            interfaces.push(interface);
        }
        Result::Ok(interfaces)
    }

    fn read_constant_pool(self: &mut ClassReader<'a>) -> ParseResult<Vec<ConstantPoolInfo>> {
        let cp_count = try!(self.read_u16()) - 1;
        let mut cp: Vec<ConstantPoolInfo> = Vec::with_capacity(cp_count as usize);

        let mut i = 0;
        while i < cp_count {
            let cp_info = try!(self.read_constant_pool_info());
            let is_double_length = cp_info.is_double_length();

            cp.push(cp_info);
            if is_double_length {
                cp.push(ConstantPoolInfo::Invalid);
                i += 1;
            }

            i += 1;
        }

        Result::Ok(cp)
    }

    fn read_constant_pool_info(self: &mut ClassReader<'a>) -> ParseResult<ConstantPoolInfo> {
        let tag = try!(self.read_u8());
        debug!("read constant pool info tag {}", tag);
        let info = match tag {
            1 => {
                let length = try!(self.read_u16());
                let data = try!(self.read_bytes(length as u32));
                let string = read_modified_utf8(&data.clone());
                trace!("read {} utf8 bytes {:?} -> {}", length, data, string);
                ConstantPoolInfo::Utf8(string)
            },
            3 => {
                let value = try!(self.read_u32()) as i32;
                ConstantPoolInfo::Integer(value)
            },
            4 => {
                let value = try!(self.read_u32()) as f32;
                ConstantPoolInfo::Float(value)
            },
            5 => {
                let value = try!(self.read_u64()) as i64;
                ConstantPoolInfo::Long(value)
            },
            6 => {
                let value = try!(self.read_u64()) as f64;
                ConstantPoolInfo::Double(value)
            },
            7 => {
                let name_index = try!(self.read_u16());
                ConstantPoolInfo::Class(name_index)
            },
            8 => {
                let string_index = try!(self.read_u16());
                ConstantPoolInfo::String(string_index)
            },
            9 => {
                let class_index = try!(self.read_u16());
                let name_and_type_index = try!(self.read_u16());
                ConstantPoolInfo::Fieldref(class_index, name_and_type_index)
            },
            10 => {
                let class_index = try!(self.read_u16());
                let name_and_type_index = try!(self.read_u16());
                ConstantPoolInfo::Methodref(class_index, name_and_type_index)
            },
            11 => {
                let class_index = try!(self.read_u16());
                let name_and_type_index = try!(self.read_u16());
                ConstantPoolInfo::InterfaceMethodref(class_index, name_and_type_index)
            },
            12 => {
                let name_index = try!(self.read_u16());
                let descriptor_index = try!(self.read_u16());
                ConstantPoolInfo::NameAndType(name_index, descriptor_index)
            },
            15 => {
                let reference_kind = try!(self.read_u8());
                let reference_index = try!(self.read_u16());
                ConstantPoolInfo::MethodHandle(reference_kind, reference_index)
            },
            16 => {
                let descriptor_index = try!(self.read_u16());
                ConstantPoolInfo::MethodType(descriptor_index)
            },
            18 => {
                let bootstrap_method_attr_index = try!(self.read_u16());
                let name_and_type_index = try!(self.read_u16());
                ConstantPoolInfo::InvokeDynamic(bootstrap_method_attr_index, name_and_type_index)
            },
            _ => {
                let message = format!("unknown constant pool item with tag {}", tag);
                return Result::Err(ParseError::Format(message));
            }
        };
        Result::Ok(info)
    }

    fn read_bytes(self: &mut ClassReader<'a>, length: u32) -> ParseResult<Vec<u8>> {
        let mut vec: Vec<u8> = Vec::with_capacity(length as usize);
        try!(self.reader.by_ref().take(length as u64).read_to_end(&mut vec));

        self.position += length as usize;
        Result::Ok(vec)
    }

    fn read_u64(self: &mut ClassReader<'a>) -> ParseResult<u64> {
        let mut buf: Vec<u8> = Vec::with_capacity(8);
        try!(self.reader.by_ref().take(8).read_to_end(&mut buf));

        self.position += 8;
        Result::Ok((buf[0] as u64) << 56 | (buf[1] as u64) << 48
                | (buf[2] as u64) << 40 | (buf[3] as u64) << 32
                | (buf[4] as u64) << 24 | (buf[5] as u64) << 16
                | (buf[6] as u64) << 8 | (buf[7] as u64))
    }

    fn read_u32(self: &mut ClassReader<'a>) -> ParseResult<u32> {
        let mut buf: Vec<u8> = Vec::with_capacity(4);
        try!(self.reader.by_ref().take(4).read_to_end(&mut buf));

        self.position += 4;
        Result::Ok((buf[0] as u32) << 24 | (buf[1] as u32) << 16
                | (buf[2] as u32) << 8 | (buf[3] as u32))
    }

    fn read_u16(self: &mut ClassReader<'a>) -> ParseResult<u16> {
        let mut buf: Vec<u8> = Vec::with_capacity(2);
        try!(self.reader.by_ref().take(2).read_to_end(&mut buf));

        self.position += 2;
        Result::Ok((buf[0] as u16) << 8 | (buf[1] as u16))
    }

    fn read_u8(self: &mut ClassReader<'a>) -> ParseResult<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(1);
        try!(self.reader.by_ref().take(1).read_to_end(&mut buf));

        self.position += 1;
        Result::Ok(buf[0] as u8)
    }

}

fn read_modified_utf8(buf: &Vec<u8>) -> String {
    let mut string = String::with_capacity(buf.len());

    let mut i = 0;
    while i < buf.len() {
        let b0 = buf[i] as u32;
        let decoded_char = if (b0 >> 7) == 0 {
            trace!("read byte {:08.b}", b0);
            char::from_u32(b0)
        } else if (b0 >> 5) == 0b110 {
            i += 1;
            let b1 = buf[i] as u32; // assert that (b1 >> 6) == 0b10
            let code_point = ((b0 & 0b0001_1111) << 6)
                    + (b1 & 0b0011_1111);
            trace!("read bytes {:08.b} {:08.b} -> {:032.b}", b0, b1, code_point);
            char::from_u32(code_point)
        } else if (b0 >> 4) == 0b1110 {
            i += 1;
            let b1 = buf[i] as u32; // assert that (b1 >> 6) == 0b10
            i += 1;
            let b2 = buf[i] as u32; // assert that (b1 >> 6) == 0b10
            let check_for_surrogate = i < (buf.len() - 2);
            if (b0 == 0b11101101) && ((b1 >> 4) == 0b1010) && check_for_surrogate && (buf[i+1] == 0b1110_1101) { // surrogate pair
                i += 1;
                let b3 = buf[i] as u32; // assert that b3 == 0b1110_1101
                i += 1;
                let b4 = buf[i] as u32; // assert that (b4 >> 4) == b1011
                i += 1;
                let b5 = buf[i] as u32; // assert that (b5 >> 6) == b10
                let code_point = 0b1_0000_0000_0000_0000
                        + ((b1 & 0b0000_1111) << 16)
                        + ((b2 & 0b0011_1111) << 10)
                        + ((b4 & 0b0000_1111) << 6)
                        + (b5 & 0x0011_1111);
                trace!("read bytes {:08.b} {:08.b} {:08.b} {:08.b} {:08.b} {:08.b} -> {:032.b}", b0, b1, b2, b3, b4, b5, code_point);
                trace!(" -> {} {:x}", code_point, code_point);
                char::from_u32(code_point)
            } else {
                let code_point = ((b0 & 0b0000_1111) << 12)
                        + ((b1 & 0b0011_1111) << 6)
                        + (b2 & 0b0011_1111);
                if ((code_point >= 0xD800) && (code_point <= 0xDBFF))
                        || ((code_point >= 0xDC00) && (code_point <= 0xDFFF)) {
                    info!("encountered surrogate code point {:x} which is invalid for UTF-8", code_point);
                    Option::None
                } else {
                    trace!("read bytes {:08.b} {:08.b} {:08.b} -> {:032.b}", b0, b1, b2, code_point);
                    trace!(" -> {} {:x}", code_point, code_point);
                    char::from_u32(code_point)
                }
            }
        } else {
            Option::None
        };
        match decoded_char {
            Option::None => string.push('\u{FFFD}'),
            Option::Some(c) => string.push(c)
        }
        i += 1;
    }
    string
}
