# memory

rustのメモリ表現がどうなっているか。

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
    let data: &[u8] = std::slice::from_raw_parts(p, a.len()); // size_of_val使わないと1バイト長じゃないTでうまく動かないかも
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

### str


### &str


### String


### Box<u8>


## References
- [Rustのvtableの内部構造 \- 簡潔なQ](https://qnighy.hatenablog.com/entry/2017/03/18/070000)
- [RustのSizedとfatポインタ \- 簡潔なQ](https://qnighy.hatenablog.com/entry/2017/03/04/131311)
- [Function std::mem::size_of \- rust-lang](https://doc.rust-lang.org/std/mem/fn.size_of.html)