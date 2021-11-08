// Copyright 2020 Tyler Neely
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{ffi, AsColumnFamilyRef};
use libc::{c_char, c_void, size_t};
use std::slice;

/// An atomic batch of write operations.
///
/// Making an atomic commit of several writes:
///
/// ```
/// use rocksdb::{DB, Options, WriteBatch};
///
/// let path = "_path_for_rocksdb_storage1";
/// {
///     let db = DB::open_default(path).unwrap();
///     let mut batch = WriteBatch::default();
///     batch.put(b"my key", b"my value");
///     batch.put(b"key2", b"value2");
///     batch.put(b"key3", b"value3");
///     db.write(batch); // Atomically commits the batch
/// }
/// let _ = DB::destroy(&Options::default(), path);
/// ```
pub struct WriteBatch {
    pub(crate) inner: *mut ffi::rocksdb_writebatch_t,
}

/// Receives the puts and deletes of a write batch.
///
/// The application must provide an implementation of this trait when
/// iterating the operations within a `WriteBatch`
pub trait WriteBatchIterator {
    /// Called with a key and value that were `put` into the batch.
    fn put(&mut self, key: Box<[u8]>, value: Box<[u8]>);
    /// Called with a key that was `delete`d from the batch.
    fn delete(&mut self, key: Box<[u8]>);
}

pub trait WriteBatchIteratorComplete : WriteBatchIterator {
    /// Called with a column_family_id, key and value that were `put` into the batch.
    fn merge_cf(&mut self, column_family_id: u32, key: Box<[u8]>, value: Box<[u8]>);

    /// Called with a key and value that were `put` into the batch.
    fn merge(&mut self, key: Box<[u8]>, value: Box<[u8]>);

    /// Called with a column_family_id, key and value that were `put` into the batch.
    fn put_cf(&mut self, column_family_id: u32, key: Box<[u8]>, value: Box<[u8]>);

    /// Called with a column_family_id, key that was `delete`d from the batch.
    fn single_delete_cf(&mut self, column_family_id: u32, key: Box<[u8]>);

    /// Called with a key that was `delete`d from the batch.
    fn single_delete(&mut self, key: Box<[u8]>);

    /// Called with a column_family_id, key that was `delete`d from the batch.
    fn delete_cf(&mut self, column_family_id: u32, key: Box<[u8]>);

    /// Called with a column_family_id, begin_key and end_key for range that was
    /// `delete`d from the batch.
    fn delete_range_cf(
        &mut self,
        column_family_id: u32,
        begin_key: Box<[u8]>,
        end_key: Box<[u8]>);

    fn log_data(&mut self, data: Box<[u8]>);

    /// Called with a column_family_id, key and value that were `put` into the batch.
    fn put_blob_index(&mut self, column_family_id: u32, key: Box<[u8]>, value: Box<[u8]>);

    /// Called on before preparation
    fn mark_begin_prepare(&mut self);

    /// Called at the end of ...
    fn mark_end_prepare(&mut self, xid: Box<[u8]>);

    /// Called when noop was processed
    fn mark_noop(&mut self, empty_batch: bool);

    /// Called for rollback on DB
    fn mark_rollback(&mut self, xid: Box<[u8]>);

    /// Called for commit of DB
    fn mark_commit(&mut self, xid: Box<[u8]>);
}

unsafe extern "C" fn writebatch_merge_cf_callback(
    state: *mut c_void,
    cf: u32,
    k: *const c_char,
    klen: usize,
    v: *const c_char,
    vlen: usize,
) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    let value = slice::from_raw_parts(v as *const u8, vlen as usize);
    leaked_cb.merge_cf(
        cf,
        key.to_vec().into_boxed_slice(),
        value.to_vec().into_boxed_slice(),
    );
}

unsafe extern "C" fn writebatch_merge_callback(
    state: *mut c_void,
    k: *const c_char,
    klen: usize,
    v: *const c_char,
    vlen: usize,
) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    let value = slice::from_raw_parts(v as *const u8, vlen as usize);
    leaked_cb.merge(
        key.to_vec().into_boxed_slice(),
        value.to_vec().into_boxed_slice(),
    );
}

unsafe extern "C" fn writebatch_put_cf_callback(
    state: *mut c_void,
    cf: u32,
    k: *const c_char,
    klen: usize,
    v: *const c_char,
    vlen: usize,
) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    let value = slice::from_raw_parts(v as *const u8, vlen as usize);
    leaked_cb.put_cf(
        cf,
        key.to_vec().into_boxed_slice(),
        value.to_vec().into_boxed_slice(),
    );
}

unsafe extern "C" fn writebatch_put_callback(
    state: *mut c_void,
    k: *const c_char,
    klen: usize,
    v: *const c_char,
    vlen: usize,
) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIterator);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    let value = slice::from_raw_parts(v as *const u8, vlen as usize);
    leaked_cb.put(
        key.to_vec().into_boxed_slice(),
        value.to_vec().into_boxed_slice(),
    );
}

unsafe extern "C" fn writebatch_put_blob_index_callback(
    state: *mut c_void,
    cf: u32,
    k: *const c_char,
    klen: usize,
    v: *const c_char,
    vlen: usize,
) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    let value = slice::from_raw_parts(v as *const u8, vlen as usize);
    leaked_cb.put_blob_index(
        cf,
        key.to_vec().into_boxed_slice(),
        value.to_vec().into_boxed_slice(),
    );
}

unsafe extern "C" fn writebatch_delete_cf_callback(state: *mut c_void, cf_id: u32, k: *const c_char, klen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    leaked_cb.delete_cf(cf_id, key.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_delete_callback(state: *mut c_void, k: *const c_char, klen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIterator);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    leaked_cb.delete(key.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_delete_range_cf_callback(
    state: *mut c_void,
    cf_id: u32,
    begin_k: *const c_char,
    begin_klen: usize,
    end_k: *const c_char,
    end_klen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let begin_key = slice::from_raw_parts(begin_k as *const u8, begin_klen as usize);
    let end_key = slice::from_raw_parts(end_k as *const u8, end_klen as usize);
    leaked_cb.delete_range_cf(
        cf_id,
        begin_key.to_vec().into_boxed_slice(),
        end_key.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_single_delete_cf_callback(state: *mut c_void, cf_id: u32, k: *const c_char, klen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    leaked_cb.single_delete_cf(cf_id, key.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_single_delete_callback(state: *mut c_void, k: *const c_char, klen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let key = slice::from_raw_parts(k as *const u8, klen as usize);
    leaked_cb.single_delete(key.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_log_data_callback(state: *mut c_void, data: *const c_char, datalen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let data = slice::from_raw_parts(data as *const u8, datalen as usize);
    leaked_cb.log_data(data.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_mark_begin_prepare_callback(state: *mut c_void) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    leaked_cb.mark_begin_prepare();
}

unsafe extern "C" fn writebatch_mark_end_prepare_callback(state: *mut c_void, xid: *const c_char, xidlen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let xid = slice::from_raw_parts(xid as *const u8, xidlen as usize);
    leaked_cb.mark_end_prepare(xid.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_mark_noop_callback(state: *mut c_void, empty_batch: i8) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    leaked_cb.mark_noop(empty_batch != 0);
}

unsafe extern "C" fn writebatch_mark_rollback_callback(state: *mut c_void, xid: *const c_char, xidlen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let xid = slice::from_raw_parts(xid as *const u8, xidlen as usize);
    leaked_cb.mark_rollback(xid.to_vec().into_boxed_slice());
}

unsafe extern "C" fn writebatch_mark_commit_callback(state: *mut c_void, xid: *const c_char, xidlen: usize) {
    // coerce the raw pointer back into a box, but "leak" it so we prevent
    // freeing the resource before we are done with it
    let boxed_cb = Box::from_raw(state as *mut &mut dyn WriteBatchIteratorComplete);
    let leaked_cb = Box::leak(boxed_cb);
    let xid = slice::from_raw_parts(xid as *const u8, xidlen as usize);
    leaked_cb.mark_commit(xid.to_vec().into_boxed_slice());
}


impl WriteBatch {
    pub fn len(&self) -> usize {
        unsafe { ffi::rocksdb_writebatch_count(self.inner) as usize }
    }

    /// Return WriteBatch serialized size (in bytes).
    pub fn size_in_bytes(&self) -> usize {
        unsafe {
            let mut batch_size: size_t = 0;
            ffi::rocksdb_writebatch_data(self.inner, &mut batch_size);
            batch_size as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate the put and delete operations within this write batch. Note that
    /// this does _not_ return an `Iterator` but instead will invoke the `put()`
    /// and `delete()` member functions of the provided `WriteBatchIterator`
    /// trait implementation.
    pub fn iterate(&self, callbacks: &mut dyn WriteBatchIterator) {
        let state = Box::into_raw(Box::new(callbacks));
        unsafe {
            ffi::rocksdb_writebatch_iterate(
                self.inner,
                state as *mut c_void,
                Some(writebatch_put_callback),
                Some(writebatch_delete_callback),
            );
            // we must manually set the raw box free since there is no
            // associated "destroy" callback for this object
            Box::from_raw(state);
        }
    }

    /// Iterate the put and delete operations within this write batch. Note that
    /// this does _not_ return an `Iterator` but instead will invoke the `put()`
    /// and `delete()` member functions of the provided `WriteBatchIterator`
    /// trait implementation.
    pub fn iterate_complete(&self, callbacks: &mut dyn WriteBatchIteratorComplete) {
        let state = Box::into_raw(Box::new(callbacks));
        unsafe {
            ffi::rocksdb_writebatch_iterate_complete(
                self.inner,
                state as *mut c_void,
                Some(writebatch_merge_cf_callback),
                Some(writebatch_merge_callback),
                Some(writebatch_put_cf_callback),
                Some(writebatch_put_callback),
                Some(writebatch_put_blob_index_callback),
                Some(writebatch_delete_cf_callback),
                Some(writebatch_delete_callback),
                Some(writebatch_delete_range_cf_callback),
                Some(writebatch_single_delete_cf_callback),
                Some(writebatch_single_delete_callback),
                Some(writebatch_log_data_callback),
                Some(writebatch_mark_begin_prepare_callback),
                Some(writebatch_mark_end_prepare_callback),
                Some(writebatch_mark_noop_callback),
                Some(writebatch_mark_rollback_callback),
                Some(writebatch_mark_commit_callback),
            );
            // we must manually set the raw box free since there is no
            // associated "destroy" callback for this object
            Box::from_raw(state);
        }
    }

    /// Insert a value into the database under the given key.
    pub fn put<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_put(
                self.inner,
                key.as_ptr() as *const c_char,
                key.len() as size_t,
                value.as_ptr() as *const c_char,
                value.len() as size_t,
            );
        }
    }

    /// Insert a value into the database under the given key.
    pub fn put_log_data<L>(&mut self, data: L)
    where
        L: AsRef<[u8]>,
    {
        let data = data.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_put_log_data(
                self.inner,
                data.as_ptr() as *const c_char,
                data.len() as size_t,
            );
        }
    }

    pub fn put_cf<K, V>(&mut self, cf: &impl AsColumnFamilyRef, key: K, value: V)
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_put_cf(
                self.inner,
                cf.inner(),
                key.as_ptr() as *const c_char,
                key.len() as size_t,
                value.as_ptr() as *const c_char,
                value.len() as size_t,
            );
        }
    }

    pub fn merge<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_merge(
                self.inner,
                key.as_ptr() as *const c_char,
                key.len() as size_t,
                value.as_ptr() as *const c_char,
                value.len() as size_t,
            );
        }
    }

    pub fn merge_cf<K, V>(&mut self, cf: &impl AsColumnFamilyRef, key: K, value: V)
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_merge_cf(
                self.inner,
                cf.inner(),
                key.as_ptr() as *const c_char,
                key.len() as size_t,
                value.as_ptr() as *const c_char,
                value.len() as size_t,
            );
        }
    }

    /// Removes the database entry for key. Does nothing if the key was not found.
    pub fn delete<K: AsRef<[u8]>>(&mut self, key: K) {
        let key = key.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_delete(
                self.inner,
                key.as_ptr() as *const c_char,
                key.len() as size_t,
            );
        }
    }

    pub fn delete_cf<K: AsRef<[u8]>>(&mut self, cf: &impl AsColumnFamilyRef, key: K) {
        let key = key.as_ref();

        unsafe {
            ffi::rocksdb_writebatch_delete_cf(
                self.inner,
                cf.inner(),
                key.as_ptr() as *const c_char,
                key.len() as size_t,
            );
        }
    }

    /// Remove database entries from start key to end key.
    ///
    /// Removes the database entries in the range ["begin_key", "end_key"), i.e.,
    /// including "begin_key" and excluding "end_key". It is not an error if no
    /// keys exist in the range ["begin_key", "end_key").
    pub fn delete_range<K: AsRef<[u8]>>(&mut self, from: K, to: K) {
        let (start_key, end_key) = (from.as_ref(), to.as_ref());

        unsafe {
            ffi::rocksdb_writebatch_delete_range(
                self.inner,
                start_key.as_ptr() as *const c_char,
                start_key.len() as size_t,
                end_key.as_ptr() as *const c_char,
                end_key.len() as size_t,
            );
        }
    }

    /// Remove database entries in column family from start key to end key.
    ///
    /// Removes the database entries in the range ["begin_key", "end_key"), i.e.,
    /// including "begin_key" and excluding "end_key". It is not an error if no
    /// keys exist in the range ["begin_key", "end_key").
    pub fn delete_range_cf<K: AsRef<[u8]>>(&mut self, cf: &impl AsColumnFamilyRef, from: K, to: K) {
        let (start_key, end_key) = (from.as_ref(), to.as_ref());

        unsafe {
            ffi::rocksdb_writebatch_delete_range_cf(
                self.inner,
                cf.inner(),
                start_key.as_ptr() as *const c_char,
                start_key.len() as size_t,
                end_key.as_ptr() as *const c_char,
                end_key.len() as size_t,
            );
        }
    }

    /// Clear all updates buffered in this batch.
    pub fn clear(&mut self) {
        unsafe {
            ffi::rocksdb_writebatch_clear(self.inner);
        }
    }
}

impl Default for WriteBatch {
    fn default() -> Self {
        Self {
            inner: unsafe { ffi::rocksdb_writebatch_create() },
        }
    }
}

impl Drop for WriteBatch {
    fn drop(&mut self) {
        unsafe {
            ffi::rocksdb_writebatch_destroy(self.inner);
        }
    }
}

unsafe impl Send for WriteBatch {}
