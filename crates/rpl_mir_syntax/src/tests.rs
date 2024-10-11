use crate::*;

use crate::parse::ParseError;

use pretty_assertions::assert_eq;
use quote::{quote, ToTokens};
use syn::parse::Parse;

#[track_caller]
fn test_pass<T: Parse + ToTokens>(input: impl ToTokens) {
    let input = input.into_token_stream();
    let input_string = input.to_string();
    let mir_patterns: T = syn::parse2(input).unwrap();
    let output = mir_patterns.into_token_stream();
    assert_eq!(input_string, output.to_string());
}

#[track_caller]
fn test_fail<T: Parse + ToTokens>(input: impl ToTokens, msg: impl ToString) {
    match syn::parse2::<T>(input.into_token_stream()) {
        Ok(t) => panic!("unexpected successful parse: {}", t.into_token_stream()),
        Err(e) => assert_eq!(e.to_string(), msg.to_string()),
    }
}

macro_rules! pass {
    ($test_struct:ident!( $( $tt:tt )* ) $(,)?) => {
        test_pass::<$test_struct>(quote!($($tt)*));
    };
    ($test_struct:ident!{ $( $tt:tt )* } $(,)?) => {
        pass!($test_struct!( $($tt)* ));
    };
    ($test_struct:ident![ $( $tt:tt )* ] $(,)?) => {
        pass!($test_struct!( $($tt)* ));
    };
}

macro_rules! fail {
    ($test_struct:ident!( $( $tt:tt )* ), $msg:expr $(,)?) => {
        test_fail::<$test_struct>(quote!($($tt)*), $msg);
    };
    ($test_struct:ident!{ $( $tt:tt )* }, $msg:expr $(,)?) => {
        fail!($test_struct!( $($tt)* ), $msg);
    };
    ($test_struct:ident![ $( $tt:tt )* ], $msg:expr $(,)?) => {
        fail!($test_struct!( $($tt)* ), $msg);
    };
}

#[test]
#[rustfmt::skip]
fn test_type_decl() {
    pass!(TypeDecl!( type SliceT = [T]; ));
    fail!(
        TypeDecl!( type SliceT<T> = [T]; ),
        ParseError::TypeWithGenericsNotSupported
    );
}

#[test]
fn test_path() {
    pass!(PathSegment!(std));
    pass!(Path!(std::mem::take));
    pass!(Path!(Vec<T>));
    pass!(Path!(core::ffi::c_str::CStr));
    pass!(Path!($crate::ffi::sqlite3session_attach));
    pass!(TypePath!(<Vec<T> >));
    pass!(TypePath!(<Vec<T> as Clone>::clone));
    pass!(TypePath!(<$crate::alloc::Vec<T> as Clone>::clone));
    pass!(TypePath!(<Vec<T> as $crate::clone::Clone>::clone));
    pass!(TypePath!(<$crate::alloc::Vec<T> as $crate::clone::Clone>::clone));
    pass!(TypePath!(<core::ffi::c_str::CStr>));
    pass!(TypePath!(<core::ffi::c_str::CStr>::from_bytes_with_nul_unchecked));
    #[rustfmt::skip]
    pass!(TypePath!(< <core::ffi::c_str::CStr>::from_bytes_with_nul_unchecked>::___rt_impl));

    fail!(Path!(crate::crate), "expected identifier, found keyword `crate`");
    fail!(Path!($crate::crate), "expected identifier, found keyword `crate`");
    fail!(Path!(std::crate), "expected identifier, found keyword `crate`");
    fail!(Path!($crate), format!("expected `::`"));
    fail!(Path!($crate::), "unexpected end of input, expected identifier");
    fail!(Path!(from_ptr as), "unexpected token");
    fail!(TypePath!(from_ptr as), "unexpected token");
}

#[test]
fn test_type() {
    pass!(Type!(*const u8));
    pass!(Type!([T]));
    #[rustfmt::skip]
    pass!(Type!(< <core::ffi::c_str::CStr>::from_bytes_with_nul_unchecked>::___rt_impl));

    fail!(Type!(*const u8(PtrToPtr)), "unexpected token");
}

#[test]
fn test_place() {
    pass!(Place!(x));
    pass!(Place!(x.0));
    pass!(Place!((*x.0)));
    pass!(Place!((*x.0)[2 of 3]));
    pass!(Place!((*x.0)[y]));
    pass!(Place!((*x.0)[-3 of 4]));
    pass!(Place!((*x.0)[1:3]));
    pass!(Place!((*x.0)[1:-3]));
    pass!(Place!((*self).mem));

    fail!(Place!(from_ptr as), "unexpected token");
}

#[test]
fn test_operand() {
    pass!(Operand!(std::mem::take));
    pass!(Operand!(move y));
    fail!(Operand!(copy from_ptr as), "unexpected token");
}

#[test]
fn test_rvalue() {
    pass!(CastKind!(PtrToPtr));
    pass!(RvalueCast!(from_ptr as *const u8(PtrToPtr)));

    pass!(RvalueOrCall!(&x));
    pass!(RvalueOrCall!(&mut y));
    pass!(RvalueOrCall!(&raw const *x));
    pass!(RvalueOrCall!(&raw mut *y));
    pass!(RvalueOrCall!([const 0; 5]));
    pass!(RvalueOrCall!([const 0, const 1, const 2, const 3, const 4]));
    pass!(RvalueOrCall!((const 0, const 1, const 2, const 3, const 4)));
    pass!(RvalueOrCall!(Test { x: const 0 }));
    pass!(RvalueOrCall!(*const [i32] from (ptr, meta)));

    fail!(
        RvalueCast!(from_ptr as *const u8),
        "unexpected end of input, expected parentheses"
    );
}

#[test]
fn test_call() {
    pass!(Call!( std::mem::take(move y) ));
    pass!(RvalueOrCall!( std::mem::take(move y) ));
    #[rustfmt::skip]
    pass!(Call!( < <core::ffi::c_str::CStr>::from_bytes_with_nul_unchecked>::___rt_impl(move uslice) ));
    #[rustfmt::skip]
    pass!(RvalueOrCall!( < <core::ffi::c_str::CStr>::from_bytes_with_nul_unchecked>::___rt_impl(move uslice) ));

    pass!(Call!( $crate::ffi::sqlite3session_attach(move s, move iptr) ));
    pass!(RvalueOrCall!( $crate::ffi::sqlite3session_attach(move s, move iptr) ));
}

#[test]
fn test_assign() {
    pass!(Assign!( *x = std::mem::take(move y) ));
}

#[test]
fn test_meta() {
    pass!(Meta!(meta!($T:ty);));
    pass!(Meta!(meta![$T:ty, $U:ty]; ));
    pass!(Meta!(meta! { $T:ty, $U:ty, }));
}

#[test]
fn test_declaration() {
    #[rustfmt::skip]
    pass!(Declaration!( type SliceT = [$T]; ));
    pass!(Declaration!( let x: u32 = const 0_usize; ));
    pass!(Declaration!( let to_ptr: *const u8 = from_ptr as *const u8 (PtrToPtr); ));
}

#[test]
fn test_block() {
    pass!(Block!({
        x1 = copy x0;
        x2 = <usize as Step>::forward_unchecked(copy x1, const 1_usize);
        x0 = move x2;
        x3 = Some(copy x1);
        x = copy (x3 as Some).0;
        base = copy (*self).mem;
        offset = copy x as isize (IntToInt);
        elem_ptr = Offset(copy base, copy offset);
        _ = drop_in_place(copy elem_ptr);
    }));
}

#[test]
fn test_switch_int() {
    pass!(SwitchInt!(
        switchInt(move cmp) {
            false => break,
            _ => {
                x1 = copy x0;
                x2 = <usize as Step>::forward_unchecked(copy x1, const 1_usize);
                x0 = move x2;
                x3 = Some(copy x1);
                x = copy (x3 as Some).0;
                base = copy (*self).mem;
                offset = copy x as isize (IntToInt);
                elem_ptr = Offset(copy base, copy offset);
                _ = drop_in_place(copy elem_ptr);
            }
        }
    ));
}

#[test]
fn test_loop() {
    pass!(Loop!(loop {}));
    pass!(Loop!(
        loop {
            x_cmp = copy x0;
            cmp = Lt(move x_cmp, copy len);
            switchInt(move cmp) {
                false => break,
                _ => {
                    x1 = copy x0;
                    x2 = <usize as Step>::forward_unchecked(copy x1, const 1_usize);
                    x0 = move x2;
                    x3 = Some(copy x1);
                    x = copy (x3 as Some).0;
                    base = copy (*self).mem;
                    offset = copy x as isize (IntToInt);
                    elem_ptr = Offset(copy base, copy offset);
                    _ = drop_in_place(copy elem_ptr);
                }
            }
        }
    ));
}

#[test]
fn test_statement() {
    #[rustfmt::skip]
    pass!(Statement!( *x = copy y.0; ));
    pass!(Statement!( cmp = Lt(move x_cmp, copy len); ));
    pass!(Statement!( x1 = copy x0; ));
    pass!(Statement!( x2 = <usize as Step>::forward_unchecked(copy x1, const 1_usize); ));
    pass!(Statement!( x0 = move x2; ));
    pass!(Statement!( x3 = Some(copy x1); ));
    pass!(Statement!( x = copy (x3 as Some).0; ));
    pass!(Statement!( base = copy (*self).mem; ));
    pass!(Statement!( offset = copy x as isize (IntToInt); ));
    pass!(Statement!( elem_ptr = Offset(copy base, copy offset); ));
    pass!(Statement!( _ = drop_in_place(copy elem_ptr); ));
    pass!(Statement!( *x = std::mem::take(move y); ));
    pass!(Statement!(drop(y[x]);));
}

#[test]
fn test_mir_pattern() {
    pass!(Mir!());
    pass!(Mir! {
        meta!($T:ty);
        type SliceT = [$T];
        type RefSliceT = &SliceT;
        type PtrSliceT = *const SliceT;
        type PtrU8 = *const u8;
        type SliceU8 = [u8];
        type PtrSliceU8 = *const SliceU8;
        type RefSliceU8 = &SliceU8;

        let from_slice: SliceT = _;
        let from_raw_slice: PtrSliceT = &raw const *from_slice;
        let from_len: usize = Len(from_slice);
        let ty_size: usize = SizeOf($T);
        let to_ptr: PtrU8 = from_ptr as PtrU8 (PtrToPtr);
        let to_len: usize = Mul(from_len, ty_size);
        let to_raw_slice: PtrSliceU8 = *const SliceU8 from (to_ptr, t_len);
        let to_slice: RefSliceU8 = &*to_raw_slice;
    });
    pass!(Mir! {
        use core::ffi::c_str::CString;
        use core::ffi::c_str::Cstr;
        use core::ptr::non_null::NonNull;
        use $crate::ffi::sqlite3session_attach;

        type NonNullSliceU8 = NonNull<[u8]>;
        type PtrSliceU8 = *const [u8];
        type RefSliceU8 = &[u8];
        type PtrCStr = *const CStr;
        type RefCStr = &CStr;
        type PtrSliceI8 = *const [i8];
        type PtrI8 = *const i8;

        let cstring: CString = _;
        let non_null: NonNullSliceU8 = copy (((cstring.inner).0).pointer);
        let uslice_ptr: PtrSliceU8 = copy (non_null.pointer);
        let cstr: PtrCStr = copy uslice_ptr as PtrCStr (PtrToPtr);
        // /*
        let uslice: RefSliceU8 = &(*uslice_ptr);
        let cstr: RefCStr = < <CStr>::from_bytes_with_nul_unchecked>::___rt_impl(move uslice);
        // */
        let islice: PtrSliceI8 = &raw const ((*cstr).inner);
        let iptr: PtrI8 = move islice as PtrI8 (PtrToPtr);
        let s: i32;
        let ret: i32;
        drop(cstring);
        s = _;
        ret = sqlite3session_attach(move s, move iptr);
    });
    pass!(Mir! {
        let _0: [u32; 3];
        let _1: isize;

        switchInt(copy _1) {
            0_isize => _0 = [const 3_u32, const 4_u32, const 5_u32],
            1_isize => _0 = [const 6_u32, const 7_u32, const 8_u32],
            _ => _0 = [const 9_u32, const 10_u32, const 11_u32],
        }
    });
}

#[test]
fn test_parse_cve_2018_21000() {
    pass!(Mir! {
        meta!{
            $T1:ty,
            $T2:ty,
            $T3:ty,
        }

        type VecT1 = std::vec::Vec<$T1>;
        type VecT2 = std::vec::Vec<$T2>;
        type VecT3 = std::vec::Vec<$T3>;
        type PtrT1 = *mut $T1;
        type PtrT3 = *mut $T3;

        let from_vec: VecT1 = _;
        let size: usize = SizeOf($T2);
        let from_cap: usize = Vec::capacity(move from_vec);
        let to_cap: usize = Mul(copy from_cap, copy size);
        let from_len: usize = Len(from_vec);
        let to_len: usize = Mul(copy from_len, copy size);
        let from_vec_ptr: PtrT1 = Vec::as_mut_ptr(move from_vec);
        let to_vec_ptr: PtrT3 = copy from_vec_ptr as PtrT3 (PtrToPtr);
        // tuple: not implemented yet
        // let tmp: () = std::mem::forget(move from_vec); 
        let res: VecT3 = Vec::from_raw_parts(copy to_vec_ptr, copy to_cap, copy to_len);
    });
}
