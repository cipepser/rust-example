# memory

## version

```sh
❯ cargo version
cargo 1.32.0

❯ rustc --version
rustc 1.33.0
```

## rustのメモリ表現がどうなっているか

[RustのSizedとfatポインタ \- 簡潔なQ](https://qnighy.hatenablog.com/entry/2017/03/04/131311)から`as_raw_bytes`を借りる。

```rust
fn as_raw_bytes<'a, T: ?Sized>(x: &'a T) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(
            x as *const T as *const u8,
            std::mem::size_of_val(x))
    }
}
```

### [u8]

まずは`[u8]`（`u8`のスライス）でメモリ表現を見てみる。

これ(`[T]`)はスタック領域に確保されるはず。

`[i32]`を表示してみる。

```rust
let a = [1, 2, 3, 4]; // a: [i32; 4]
println!("{:?}", a);
println!("{:x?}", as_raw_bytes(&a));
```

結果。`0`が多い？

```
[1, 2, 3, 4, 5]
[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0]
```

数字を変えてみる。

```rust
let a = [255, 256, 1023, 1024, 1025];
println!("{:?}", a);
println!("{:x?}", as_raw_bytes(&a));
```

4バイトごと？

```
// 改行は手動
[255, 256, 1023, 1024, 1025]
[ff, 0, 0, 0,
0, 1, 0, 0,
ff, 3, 0, 0,
 0, 4, 0, 0,
 1, 4, 0, 0]
```

`0d255` = `0x00ff`
`0d256` = `0x0100`
`0d1023` = `0x03ff`
`0d1024` = `0x0400`
`0d1025` = `0x0401`

```
// 改行は手動
[4096, 4097, 8192, 8193, 16384, 16385, 32768, 32769]
[0, 10, 0, 0,
 1, 10, 0, 0,
 0, 20, 0, 0,
 1, 20, 0, 0,
 0, 40, 0, 0,
 1, 40, 0, 0,
 0, 80, 0, 0,
 1, 80, 0, 0]
```

```
// 改行、スペースは手動
[32768, 65536, 131072, 262144]
[0, 80, 0, 0,
 0,  0, 1, 0,
 0,  0, 2, 0,
 0,  0, 4, 0]
```

これまでの結果を見てわかるようにリトルエンディアンで表現されている。
`as_raw_bytes`の返り値の型は`[u8]`なので、

```
[0, 4, 0, 0,
 1, 4, 0, 0]
```

は

```
[00, 04, 00, 00,
 01, 04, 00, 00]
```

であることに注意。

これをリトルエンディアンで読めば、

`0x00000400` = `0d1024`
`0x00000401` = `0d1025`

となる。

またちゃんとスタック領域に確保もされていた。

### Vec<u8>

次に`Vec<u8>`を見ていく。
これはfat pointerになっているはず。

```rust
let a = vec![1, 2, 3, 4];
println!("{:?}", a);
println!("{:x?}", as_raw_bytes(&a));
```

結果

```
// 改行とスペースを入れる整形は手動
[1, 2, 3, 4]
[d0, 2c, c0, d5, a8, 7f, 0, 0,
  4, 0, 0, 0, 0, 0, 0, 0,
  4, 0, 0, 0, 0, 0, 0, 0]
```

`vec.rs`を見てみると以下のようになっている。

```rust
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}
```

`RawVec`は`raw_vec.rs`で定義されている。

```
#[allow(missing_debug_implementations)]
pub struct RawVec<T, A: Alloc = Global> {
    ptr: Unique<T>,
    cap: usize,
    a: A,
}
```


また`usize`について、自分の環境では8バイト。
```rust
println!("size of usize: {}", std::mem::size_of::<usize>());
// size of usize: 8
```

なので、改めて結果を見てみると後半の`4`から始まる箇所は`len`と`cap`で、最初の8バイトが実データへのポインタになっている。

```
[d0, 2c, c0, d5, a8, 7f, 0, 0,
  4, 0, 0, 0, 0, 0, 0, 0,
  4, 0, 0, 0, 0, 0, 0, 0]
```

いよいよやりたかったヒープ上の実データをみてみる。

```rust
let a = vec![4, 1, 2, 3];
println!("{:?}", a);
println!("{:x?}", as_raw_bytes(&a));

unsafe {
    let p = a.as_ptr();
    println!("{:?}", p); // 0x7feba05027c0
    println!("{:?}", *p); // 4 <- 先頭のデータ
    let data: &[u8] = std::slice::from_raw_parts(p, a.len());
    println!("{:?}", data); // [4, 1, 2, 3]
}
```

実装は`vec.rs`の`deref`（以下）を参考にした。

```rust
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> ops::Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            let p = self.buf.ptr();
            assume(!p.is_null());
            slice::from_raw_parts(p, self.len)
        }
    }
}
```

### [char]

`[char]`でもどうなるか見てみる。

```rust
let a = ['a', 'b', 'c'];
println!("{:?}", a);
// ['a', 'b', 'c']

println!("{:x?}", as_raw_bytes(&a));
// [61, 0, 0, 0, 62, 0, 0, 0, 63, 0, 0, 0]
```

`[u8]`と同じでスタック上に載っている。

[char \- Rust](https://doc.rust-lang.org/std/primitive.char.html)にあるように`char`は4バイト長。

> `char` is always four bytes in size.

`Vec<char>`はどうか。

```rust
let a = vec!['a', 'b', 'c'];
println!("{:?}", a);
// ['a', 'b', 'c']

println!("{:x?}", as_raw_bytes(&a));
// [d0, 2c, 40, d7, ad, 7f, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]

unsafe {
    let p = a.as_ptr();
    println!("{:?}", p);
    // 0x7fadd7402cd0

    println!("{:?}", *p);
    // 'a'

    let data = std::slice::from_raw_parts(p, a.len());
    println!("{:?}", data);
    // ['a', 'b', 'c']
}
```

```
[d0, 2c, 40, d7, ad, 7f, 0, 0,
  3, 0, 0, 0, 0, 0, 0, 0,
  3, 0, 0, 0, 0, 0, 0, 0]
```

`len`と`cap`が`3`（うしろ2行）で、1行目はヒープにある実データへのポインタになっている。


### [&str]

```rust
let a = ["a", "b", "c"]; // a: [&str; 3]

println!("{:?}", a);
// ["a", "b", "c"]

println!("{:x?}", as_raw_bytes(&a));
// [82, 59, 85, 4, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 80, 59, 85, 4, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 81, 59, 85, 4, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]
```

別の例ももうひとつ。


```rust
let a = ["a", "ab", "abc"];
println!("{:?}", a);
// ["a", "ab", "abc"]

println!("{:x?}", as_raw_bytes(&a));
// [35, 77, 46, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 30, 77, 46, 1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 32, 77, 46, 1, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]
```

バイト列を見やすいように整形する。

```
[82, 59, 85, 4, 1, 0, 0, 0,
 1, 0, 0, 0, 0, 0, 0, 0,
 80, 59, 85, 4, 1, 0, 0, 0,
 1, 0, 0, 0, 0, 0, 0, 0,
 81, 59, 85, 4, 1, 0, 0, 0,
 1, 0, 0, 0, 0, 0, 0, 0]
```

```
[35, 77, 46, 1, 1, 0, 0, 0,
 1, 0, 0, 0, 0, 0, 0, 0,
 30, 77, 46, 1, 1, 0, 0, 0,
 2, 0, 0, 0, 0, 0, 0, 0,
 32, 77, 46, 1, 1, 0, 0, 0,
 3, 0, 0, 0, 0, 0, 0, 0]
```


[str \- Rust](https://doc.rust-lang.org/std/primitive.str.html)にあるように`&str`は実データへのポインタと`len`でできている。

> A &str is made up of two components: a pointer to some bytes, and a length.

今回の表示した対象は`[&str]`である。つまり、`&str`のスライス。
`[u8]`でみたようにスライスはスタック上にデータをそのまま載せる。

そして`&str`は上述の通り、実データへのポインタと`len`で表現される。
よって上記のような結果（ポインタ＋`len`が3つ並ぶ）となった。

最初がポインタであることも確かめておく。

```rust
let a = ["a", "ab", "abc"];
println!("{:?}", a);
// ["a", "ab", "abc"]

println!("{:x?}", as_raw_bytes(&a));
// [55, 8e, f9, 7, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 50, 8e, f9, 7, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 52, 8e, f9, 7, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]

unsafe {
    for i in 0..a.len() {
        println!("-------");
        let p = a[i].as_ptr();
        println!("{:?}", p);
        println!("{:?}", *p);
        let data = std::slice::from_raw_parts(p, a[i].len());
        println!("{:?}", data);
    }
    // -------
    // 0x107f98e55
    // 97
    // [97]
    // -------
    // 0x107f98e50
    // 97
    // [97, 98]
    // -------
    // 0x107f98e52
    // 97
    // [97, 98, 99]
}
```

`println!("{:x?}", as_raw_bytes(&a));`で表示した結果を整形する。

```
[55, 8e, f9, 7, 1, 0, 0, 0,
 1, 0, 0, 0, 0, 0, 0, 0,
 50, 8e, f9, 7, 1, 0, 0, 0,
 2, 0, 0, 0, 0, 0, 0, 0,
 52, 8e, f9, 7, 1, 0, 0, 0,
 3, 0, 0, 0, 0, 0, 0, 0]
```

3つに分かれた各要素の先頭8バイトがデータへのポインタになっていることが確認できた。

### String

```rust
let a = [
    "a".to_string(),
    "ab".to_string(),
    "abc".to_string()]; // a: [String; 3]

println!("{:?}", a);
// ["a", "ab", "abc"]

println!("{:x?}", as_raw_bytes(&a));
// [90, 2c, c0, 4a, 9b, 7f, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, a0, 2c, c0, 4a, 9b, 7f, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, b0, 2c, c0, 4a, 9b, 7f, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]
```

例によって整形する。

```
[90, 2c, c0, 4a, 9b, 7f, 0, 0,
  1, 0, 0, 0, 0, 0, 0, 0,
  1, 0, 0, 0, 0, 0, 0, 0,
 a0, 2c, c0, 4a, 9b, 7f, 0, 0,
  2, 0, 0, 0, 0, 0, 0, 0,
  2, 0, 0, 0, 0, 0, 0, 0,
 b0, 2c, c0, 4a, 9b, 7f, 0, 0,
  3, 0, 0, 0, 0, 0, 0, 0,
  3, 0, 0, 0, 0, 0, 0, 0]
```

`[String]`も`[&str]`と同じように3つの`String`が並んでいる。
上述の通り`&str`は実データへのポインタと`len`を持っている。
`String`はこれに加えて`cap`を持つ。ちょうど`Vec`に対応している。

そうとなれば実データをちゃんと指しているか確認したくなる。


```rust
let a = [
    "a".to_string(),
    "ab".to_string(),
    "abc".to_string()]; // a: Vec<String>

println!("{:?}", a);
// ["a", "ab", "abc"]

println!("{:x?}", as_raw_bytes(&a));
// [90, 2e, 40, 6f, 8c, 7f, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, a0, 2e, 40, 6f, 8c, 7f, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, b0, 2e, 40, 6f, 8c, 7f, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]

unsafe {
    for i in 0..a.len() {
        println!("-------");
        let p = a[i].as_ptr();
        println!("{:?}", p);
        println!("{:?}", *p);
        let data = std::slice::from_raw_parts(p, a[i].len());
        println!("{:?}", data);
    }
    // -------
    // 0x7f8c6f402e90
    // 97
    // [97]
    // -------
    // 0x7f8c6f402ea0
    // 97
    // [97, 98]
    // -------
    // 0x7f8c6f402eb0
    // 97
    // [97, 98, 99]
}
```

`Vec<String>`がどうなるのか確認しましょう。

```rust
let a = vec![
    "a".to_string(),
    "ab".to_string(),
    "abc".to_string()];
println!("{:?}", a);
// ["a", "ab", "abc"]

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
```

これを見るとわかるように`a.as_ptr()`と`a[0].as_ptr()`は違うアドレスだが、同じ`"a"`を参照する。
プログラミングRustP.78にこの参照の図がある。


### Box<u8>


## References
- [Rustのvtableの内部構造 \- 簡潔なQ](https://qnighy.hatenablog.com/entry/2017/03/18/070000)
- [RustのSizedとfatポインタ \- 簡潔なQ](https://qnighy.hatenablog.com/entry/2017/03/04/131311)
- [Function std::mem::size_of \- rust-lang](https://doc.rust-lang.org/std/mem/fn.size_of.html)
- [char \- Rust](https://doc.rust-lang.org/std/primitive.char.html)
- [str \- Rust](https://doc.rust-lang.org/std/primitive.str.html)