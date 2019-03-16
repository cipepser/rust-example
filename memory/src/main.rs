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
//
//
//    let a = [32768, 32768 * 2, 32768 * 4, 32768 * 8];
//    println!("{:?}", a);
//    println!("{:x?}", as_raw_bytes(&a));

    let a = vec![4, 1, 2, 3];
    println!("{:?}", a);
    println!("{:x?}", as_raw_bytes(&a));

    unsafe {
        let p = a.as_ptr();
        println!("{:?}", p);
        println!("{:?}", *p);
        let data: &[u8] = std::slice::from_raw_parts(p, a.len());
        println!("{:?}", data);
    }
}

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
