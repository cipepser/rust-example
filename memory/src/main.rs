fn main() {
//    let s = "abcde".to_string();
////    println!("{}", s);
//
//
//    let size = std::mem::size_of_val(&s);
//    println!("{}", size);
//
//    println!("{:x?}", as_raw_bytes(&s));
//    println!("{}", s.len());
//    println!("{}", s.capacity());


//    let a = [
//        "a".to_string(),
//        "ab".to_string(),
//        "abc".to_string()];
//
//    println!("{:?}", a);
//    println!("{:x?}", as_raw_bytes(&a));
//
//
//    let a = [
//        "a".to_string(),
//        "ab".to_string(),
//        "abc".to_string()];
//
//    println!("{:?}", a);
//    println!("{:x?}", as_raw_bytes(&a));
//
//    unsafe {
//        for i in 0..a.len() {
//            println!("-------");
//            let p = a[i].as_ptr();
//            println!("{:?}", p);
//            println!("{:?}", *p);
//            let data = std::slice::from_raw_parts(p, a[i].len());
//            println!("{:?}", data);
//        }
//    }

//    let a = vec![
//        "a".to_string(),
//        "bc".to_string(),
//        "def".to_string()];
//    println!("{:?}", a);
//
//    println!("{:x?}", as_raw_bytes(&a));
//
//    println!("****");
//    unsafe {
//        let p = a.as_ptr();
//        println!("{:?}", p);
//
//        println!("{:?}", *p);
//
//        let data = std::slice::from_raw_parts(p, a.len());
//        println!("{:?}", data);
//
//        for i in 0..a.len() {
//            println!("-------");
//            let p = a[i].as_ptr();
//            println!("{:?}", p);
//            println!("{:?}", *p);
//            let data = std::slice::from_raw_parts(p, a[i].len());
//            println!("{:?}", data);
//        }
//    }

//    let b: Box<u8> = Box::new(1);
//    println!("{}", b);
//    println!("{:?}", b);
//    println!("{}", std::mem::size_of_val(&b));

//    println!("{:x?}", as_raw_bytes(&b));
//    let p = as_raw_bytes(&b);
//    let a: Vec<&str> = vec!["a"];
//    let ptr: *const &str = a.as_ptr();

//    let b = Box::new("a");
//    println!("{:x?}", as_raw_bytes(&b));
////    b.deref()
////    let p = &b as *const &str;
//    println!("{:x?}", *b);
//    let a: [i32; 5] = [255, 256, 1023, 1024, 1025];
//    println!("{:?}", a);


    let a: Vec<String> = vec![
        "a".to_string(),
        "ab".to_string(),
        "abc".to_string()];

    println!("{:x?}", as_raw_bytes(&a));
// [a0, 2d, c0, d0, d5, 7f, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]

    unsafe {
        let p = a.as_ptr(); // o *const String
        println!("{:?}", p);
        // 0x7fd5d0c02da0

        println!("{:?}", *p);
        // "a"

        let data = std::slice::from_raw_parts(p, a.len()); // data: &[String]
        println!("{:?}", data);
        // ["a", "ab", "abc"]

        for i in 0..a.len() {
            println!("-------");
            let p = a[i].as_ptr(); // p: *const u8
            println!("{:?}", p);
            println!("{:?}", *p);
            println!("len: {:?}", a[i].len());
            println!("cap: {:?}", a[i].capacity());
            let data = std::slice::from_raw_parts(p, a[i].len()); // data: &[u8]
            println!("{:?}", data);
        }
        // -------
        // 0x7fd5d0c02cd0
        // 97
        // [97]
        // -------
        // 0x7fd5d0c02ce0
        // 97
        // [97, 98]
        // -------
        // 0x7fd5d0c02cf0
        // 97
        // [97, 98, 99]
    }
}

// slice/mod.rs:
//    pub const fn as_ptr(&self) -> *const T {
//        self as *const [T] as *const T
//    }


fn as_raw_bytes<'a, T: ?Sized>(x: &'a T) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(
            x as *const T as *const u8,
            std::mem::size_of_val(x))
    }
}


//#[stable(feature = "rust1", since = "1.0.0")]
//impl<T> ops::Deref for Vec<T> {
//    type Target = [T];
//
//    fn deref(&self) -> &[T] {
//        unsafe {
//            let p = self.buf.ptr();
//            assume(!p.is_null());
//            slice::from_raw_parts(p, self.len)
//        }
//    }
//}
