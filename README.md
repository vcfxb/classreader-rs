classreader
===========

Parses the [class file format](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html) that is used by the Java Virtual Machine version 8.

Synopsis
--------

```rust
extern crate classreader;

use classreader::ClassReader;
use std::fs::File;

pub fn main() {
    let mut file = File::open("pkg/Test.class").unwrap();
    let class = ClassReader::new_from_reader(file).unwrap();

    assert_eq!(0xCAFEBABE, class.magic);
}
```

classreader uses the log crate to emit some log messages. They are mainly useful for low level debugging.

Completeness
------------

Parses the whole *rt.jar* of OpenJDK 8 without issue. Code is not yet decoded. Apart from that, everything is parsed into suitable data structures.

Issues
------

Strings from a class file's constant pool are currently parsed into a Rust `String`. It seems that class files may contain code points for surrogate pairs. These are invalid for UTF-8 encoded strings which is what Rust uses. So whenever such a code point is decoded, it is replaced by the Unicode Replacement Character U+FFFD. This happens rarely but it does happen. A log message with info level is emitted.

License
-------

Apache License Version 2. See LICENSE for the text.
