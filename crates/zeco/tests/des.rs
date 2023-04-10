use zeco::*;

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct A<'s> {
    #[zeco(arg = Len(4))]
    name: &'s [u8],
    #[zeco(arg = BE)]
    num: u32,
    #[zeco(arg = LE)]
    le_num: u32,
}

#[test]
fn de_a() {
    let buf = [
        b'H', b'u', b'g', b'o', 0x01, 0x31, 0xcb, 0xaf, 0xaf, 0xcb, 0x31, 0x01,
    ];
    let mut offset = 0;
    let out = A::deserialize(&buf, &mut offset, ()).unwrap();
    assert_eq!(
        out,
        A {
            name: "Hugo".as_bytes(),
            num: 20040623,
            le_num: 20040623
        }
    )
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct B<'s> {
    len: u8,
    #[zeco(arg = Len(len as usize))]
    name: &'s str,
}

#[test]
fn de_b() {
    let buf = [0x05, b'P', b'a', b'u', b'l', b'a'];
    let mut offset = 0;
    let out = B::deserialize(&buf, &mut offset, ()).unwrap();
    assert_eq!(
        out,
        B {
            len: 5,
            name: "Paula"
        }
    )
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum C {
    A,
    B,
    C,
}

#[test]
fn de_c() {
    let buf = [0x01];
    let mut offset = 0;
    let out = C::deserialize(&buf, &mut offset, ()).unwrap();
    assert_eq!(out, C::B)
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[zeco(tag_repr = u16,tag_arg = BE)]
enum D {
    A,
    B,
    C,
}

#[test]
fn de_d() {
    let buf = [0x00, 0x01];
    let mut offset = 0;
    let out = D::deserialize(&buf, &mut offset, ()).unwrap();
    assert_eq!(out, D::B)
}
