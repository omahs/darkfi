(function() {var implementors = {
"arrayvec":[["impl&lt;const CAP: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.usize.html\">usize</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"arrayvec/struct.ArrayString.html\" title=\"struct arrayvec::ArrayString\">ArrayString</a>&lt;CAP&gt;"],["impl&lt;T, const CAP: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.usize.html\">usize</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.slice.html\">[T]</a>&gt; for <a class=\"struct\" href=\"arrayvec/struct.ArrayVec.html\" title=\"struct arrayvec::ArrayVec\">ArrayVec</a>&lt;T, CAP&gt;"]],
"async_std":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"async_std/path/struct.Path.html\" title=\"struct async_std::path::Path\">Path</a>&gt; for <a class=\"struct\" href=\"async_std/path/struct.PathBuf.html\" title=\"struct async_std::path::PathBuf\">PathBuf</a>"]],
"bitvec":[["impl&lt;A, O&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"bitvec/slice/struct.BitSlice.html\" title=\"struct bitvec::slice::BitSlice\">BitSlice</a>&lt;&lt;A as <a class=\"trait\" href=\"bitvec/view/trait.BitView.html\" title=\"trait bitvec::view::BitView\">BitView</a>&gt;::<a class=\"associatedtype\" href=\"bitvec/view/trait.BitView.html#associatedtype.Store\" title=\"type bitvec::view::BitView::Store\">Store</a>, O&gt;&gt; for <a class=\"struct\" href=\"bitvec/array/struct.BitArray.html\" title=\"struct bitvec::array::BitArray\">BitArray</a>&lt;A, O&gt;<span class=\"where fmt-newline\">where\n    A: <a class=\"trait\" href=\"bitvec/view/trait.BitViewSized.html\" title=\"trait bitvec::view::BitViewSized\">BitViewSized</a>,\n    O: <a class=\"trait\" href=\"bitvec/order/trait.BitOrder.html\" title=\"trait bitvec::order::BitOrder\">BitOrder</a>,</span>"],["impl&lt;T, O&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"bitvec/slice/struct.BitSlice.html\" title=\"struct bitvec::slice::BitSlice\">BitSlice</a>&lt;T, O&gt;&gt; for <a class=\"struct\" href=\"bitvec/vec/struct.BitVec.html\" title=\"struct bitvec::vec::BitVec\">BitVec</a>&lt;T, O&gt;<span class=\"where fmt-newline\">where\n    T: <a class=\"trait\" href=\"bitvec/store/trait.BitStore.html\" title=\"trait bitvec::store::BitStore\">BitStore</a>,\n    O: <a class=\"trait\" href=\"bitvec/order/trait.BitOrder.html\" title=\"trait bitvec::order::BitOrder\">BitOrder</a>,</span>"],["impl&lt;T, O&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"bitvec/slice/struct.BitSlice.html\" title=\"struct bitvec::slice::BitSlice\">BitSlice</a>&lt;T, O&gt;&gt; for <a class=\"struct\" href=\"bitvec/boxed/struct.BitBox.html\" title=\"struct bitvec::boxed::BitBox\">BitBox</a>&lt;T, O&gt;<span class=\"where fmt-newline\">where\n    T: <a class=\"trait\" href=\"bitvec/store/trait.BitStore.html\" title=\"trait bitvec::store::BitStore\">BitStore</a>,\n    O: <a class=\"trait\" href=\"bitvec/order/trait.BitOrder.html\" title=\"trait bitvec::order::BitOrder\">BitOrder</a>,</span>"]],
"bytes":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>]&gt; for <a class=\"struct\" href=\"bytes/struct.BytesMut.html\" title=\"struct bytes::BytesMut\">BytesMut</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>]&gt; for <a class=\"struct\" href=\"bytes/struct.Bytes.html\" title=\"struct bytes::Bytes\">Bytes</a>"]],
"clap_builder":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"clap_builder/builder/struct.Str.html\" title=\"struct clap_builder::builder::Str\">Str</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.69.0/std/ffi/os_str/struct.OsStr.html\" title=\"struct std::ffi::os_str::OsStr\">OsStr</a>&gt; for <a class=\"struct\" href=\"clap_builder/builder/struct.OsStr.html\" title=\"struct clap_builder::builder::OsStr\">OsStr</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"clap_builder/struct.Id.html\" title=\"struct clap_builder::Id\">Id</a>"]],
"crossbeam_epoch":[["impl&lt;T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + <a class=\"trait\" href=\"crossbeam_epoch/trait.Pointable.html\" title=\"trait crossbeam_epoch::Pointable\">Pointable</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;T&gt; for <a class=\"struct\" href=\"crossbeam_epoch/struct.Owned.html\" title=\"struct crossbeam_epoch::Owned\">Owned</a>&lt;T&gt;"]],
"encode_unicode":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"encode_unicode/struct.Utf8Char.html\" title=\"struct encode_unicode::Utf8Char\">Utf8Char</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u16.html\">u16</a>]&gt; for <a class=\"struct\" href=\"encode_unicode/struct.Utf16Char.html\" title=\"struct encode_unicode::Utf16Char\">Utf16Char</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>]&gt; for <a class=\"struct\" href=\"encode_unicode/struct.Utf8Char.html\" title=\"struct encode_unicode::Utf8Char\">Utf8Char</a>"]],
"futures_lite":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;T&gt; for <a class=\"struct\" href=\"futures_lite/io/struct.AsyncAsSync.html\" title=\"struct futures_lite::io::AsyncAsSync\">AsyncAsSync</a>&lt;'_, '_, T&gt;"]],
"generic_array":[["impl&lt;T, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/core/primitive.slice.html\">[T]</a>&gt; for <a class=\"struct\" href=\"generic_array/struct.GenericArray.html\" title=\"struct generic_array::GenericArray\">GenericArray</a>&lt;T, N&gt;<span class=\"where fmt-newline\">where\n    N: <a class=\"trait\" href=\"generic_array/trait.ArrayLength.html\" title=\"trait generic_array::ArrayLength\">ArrayLength</a>&lt;T&gt;,</span>"]],
"http":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"http/header/struct.HeaderName.html\" title=\"struct http::header::HeaderName\">HeaderName</a>"]],
"log":[["impl&lt;'k&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"log/kv/struct.Key.html\" title=\"struct log::kv::Key\">Key</a>&lt;'k&gt;"]],
"rkyv":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.slice.html\">[T]</a>&gt; for <a class=\"struct\" href=\"rkyv/vec/struct.RawArchivedVec.html\" title=\"struct rkyv::vec::RawArchivedVec\">RawArchivedVec</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"rkyv/trait.ArchivePointee.html\" title=\"trait rkyv::ArchivePointee\">ArchivePointee</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;T&gt; for <a class=\"struct\" href=\"rkyv/rc/struct.ArchivedRc.html\" title=\"struct rkyv::rc::ArchivedRc\">ArchivedRc</a>&lt;T, F&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"rkyv/string/struct.ArchivedString.html\" title=\"struct rkyv::string::ArchivedString\">ArchivedString</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>]&gt; for <a class=\"struct\" href=\"rkyv/util/struct.AlignedVec.html\" title=\"struct rkyv::util::AlignedVec\">AlignedVec</a>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.slice.html\">[T]</a>&gt; for <a class=\"struct\" href=\"rkyv/util/struct.ScratchVec.html\" title=\"struct rkyv::util::ScratchVec\">ScratchVec</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.slice.html\">[T]</a>&gt; for <a class=\"struct\" href=\"rkyv/vec/struct.ArchivedVec.html\" title=\"struct rkyv::vec::ArchivedVec\">ArchivedVec</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"rkyv/trait.ArchivePointee.html\" title=\"trait rkyv::ArchivePointee\">ArchivePointee</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;T&gt; for <a class=\"struct\" href=\"rkyv/boxed/struct.ArchivedBox.html\" title=\"struct rkyv::boxed::ArchivedBox\">ArchivedBox</a>&lt;T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.69.0/core/ffi/c_str/struct.CStr.html\" title=\"struct core::ffi::c_str::CStr\">CStr</a>&gt; for <a class=\"struct\" href=\"rkyv/ffi/struct.ArchivedCString.html\" title=\"struct rkyv::ffi::ArchivedCString\">ArchivedCString</a>"]],
"serde_spanned":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"serde_spanned/struct.Spanned.html\" title=\"struct serde_spanned::Spanned\">Spanned</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.69.0/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;"]],
"sled":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>]&gt; for <a class=\"struct\" href=\"sled/struct.IVec.html\" title=\"struct sled::IVec\">IVec</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>]&gt; for &amp;<a class=\"struct\" href=\"sled/struct.IVec.html\" title=\"struct sled::IVec\">IVec</a>"]],
"smallvec":[["impl&lt;A: <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[&lt;A as <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt;::<a class=\"associatedtype\" href=\"smallvec/trait.Array.html#associatedtype.Item\" title=\"type smallvec::Array::Item\">Item</a>]&gt; for <a class=\"struct\" href=\"smallvec/struct.SmallVec.html\" title=\"struct smallvec::SmallVec\">SmallVec</a>&lt;A&gt;"]],
"time":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.69.0/std/time/struct.Instant.html\" title=\"struct std::time::Instant\">Instant</a>&gt; for <a class=\"struct\" href=\"time/struct.Instant.html\" title=\"struct time::Instant\">Instant</a>"]],
"tinyvec":[["impl&lt;A: <a class=\"trait\" href=\"tinyvec/trait.Array.html\" title=\"trait tinyvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[&lt;A as <a class=\"trait\" href=\"tinyvec/trait.Array.html\" title=\"trait tinyvec::Array\">Array</a>&gt;::<a class=\"associatedtype\" href=\"tinyvec/trait.Array.html#associatedtype.Item\" title=\"type tinyvec::Array::Item\">Item</a>]&gt; for <a class=\"enum\" href=\"tinyvec/enum.TinyVec.html\" title=\"enum tinyvec::TinyVec\">TinyVec</a>&lt;A&gt;"],["impl&lt;A: <a class=\"trait\" href=\"tinyvec/trait.Array.html\" title=\"trait tinyvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;[&lt;A as <a class=\"trait\" href=\"tinyvec/trait.Array.html\" title=\"trait tinyvec::Array\">Array</a>&gt;::<a class=\"associatedtype\" href=\"tinyvec/trait.Array.html#associatedtype.Item\" title=\"type tinyvec::Array::Item\">Item</a>]&gt; for <a class=\"struct\" href=\"tinyvec/struct.ArrayVec.html\" title=\"struct tinyvec::ArrayVec\">ArrayVec</a>&lt;A&gt;"],["impl&lt;'s, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/core/primitive.slice.html\">[T]</a>&gt; for <a class=\"struct\" href=\"tinyvec/struct.SliceVec.html\" title=\"struct tinyvec::SliceVec\">SliceVec</a>&lt;'s, T&gt;"]],
"toml_edit":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"toml_edit/struct.InternalString.html\" title=\"struct toml_edit::InternalString\">InternalString</a>"]],
"wasmer_types":[],
"wasmparser":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"wasmparser/types/struct.KebabStr.html\" title=\"struct wasmparser::types::KebabStr\">KebabStr</a>&gt; for <a class=\"struct\" href=\"wasmparser/types/struct.KebabString.html\" title=\"struct wasmparser::types::KebabString\">KebabString</a>"]],
"winnow":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"winnow/stream/struct.BStr.html\" title=\"struct winnow::stream::BStr\">BStr</a>&gt; for <a class=\"struct\" href=\"https://doc.rust-lang.org/1.69.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.69.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"winnow/stream/struct.Bytes.html\" title=\"struct winnow::stream::Bytes\">Bytes</a>&gt; for <a class=\"struct\" href=\"https://doc.rust-lang.org/1.69.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.69.0/std/primitive.u8.html\">u8</a>&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()