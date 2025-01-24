#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use anyhow::{bail, Result};
use wasmtime::component::{Linker, Resource, ResourceTable};
use wasmtime_wasi_io::{
    poll::{subscribe, DynPollable, Pollable},
    streams::{DynInputStream, DynOutputStream, InputStream, OutputStream},
    IoImpl, IoView,
};

wasmtime::component::bindgen!({
    world: "toy:embedding/bindings",
    async: { only_imports: [] },
    trappable_imports: true,
    with: {
        "wasi:io": wasmtime_wasi_io::bindings::wasi::io,
    }
});

pub fn add_to_linker_async<T: Embedding>(linker: &mut Linker<T>) -> Result<()> {
    let closure = type_annotate::<T, _>(|t| EImpl(wasmtime_wasi_io::IoImpl(t)));
    wasi::clocks::monotonic_clock::add_to_linker_get_host(linker, closure)?;
    wasi::cli::environment::add_to_linker_get_host(linker, closure)?;
    wasi::cli::exit::add_to_linker_get_host(linker, closure)?;
    wasi::cli::stdin::add_to_linker_get_host(linker, closure)?;
    wasi::cli::stdout::add_to_linker_get_host(linker, closure)?;
    wasi::cli::stderr::add_to_linker_get_host(linker, closure)?;
    wasi::filesystem::preopens::add_to_linker_get_host(linker, closure)?;
    wasi::filesystem::types::add_to_linker_get_host(linker, closure)?;
    wasi::random::random::add_to_linker_get_host(linker, closure)?;
    wasi::http::types::add_to_linker_get_host(linker, closure)?;
    Ok(())
}
fn type_annotate<T: Embedding, F>(val: F) -> F
where
    F: Fn(&mut T) -> EImpl<&mut T>,
{
    val
}

pub struct EImpl<T>(IoImpl<T>);
impl<T: IoView> IoView for EImpl<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(&mut self.0 .0)
    }
}
pub trait Embedding: wasmtime_wasi_io::IoView {
    fn monotonic_now(&self) -> u64;
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable;
    fn stdin(&self) -> impl InputStream;
    fn stdout(&self) -> impl OutputStream;
    fn stderr(&self) -> impl OutputStream;
}

impl<T: Embedding> Embedding for EImpl<T> {
    fn monotonic_now(&self) -> u64 {
        T::monotonic_now(&self.0 .0)
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        T::monotonic_timer(&self.0 .0, deadline)
    }
    fn stdin(&self) -> impl InputStream {
        T::stdin(&self.0 .0)
    }
    fn stdout(&self) -> impl OutputStream {
        T::stdout(&self.0 .0)
    }
    fn stderr(&self) -> impl OutputStream {
        T::stderr(&self.0 .0)
    }
}

impl<T: ?Sized + Embedding> Embedding for &mut T {
    fn monotonic_now(&self) -> u64 {
        T::monotonic_now(self)
    }
    fn monotonic_timer(&self, deadline: u64) -> impl Pollable {
        T::monotonic_timer(self, deadline)
    }
    fn stdin(&self) -> impl InputStream {
        T::stdin(self)
    }
    fn stdout(&self) -> impl OutputStream {
        T::stdout(self)
    }
    fn stderr(&self) -> impl OutputStream {
        T::stderr(self)
    }
}

impl<E: Embedding> wasi::clocks::monotonic_clock::Host for EImpl<E> {
    fn now(&mut self) -> Result<wasi::clocks::monotonic_clock::Instant> {
        Ok(self.monotonic_now())
    }
    fn resolution(&mut self) -> Result<wasi::clocks::monotonic_clock::Duration> {
        Ok(1)
    }
    fn subscribe_duration(
        &mut self,
        duration: wasi::clocks::monotonic_clock::Duration,
    ) -> Result<Resource<DynPollable>> {
        self.subscribe_instant(self.monotonic_now() + duration)
    }
    fn subscribe_instant(
        &mut self,
        deadline: wasi::clocks::monotonic_clock::Instant,
    ) -> Result<Resource<DynPollable>> {
        let timer = self.monotonic_timer(deadline);
        let deadline = self.table().push(timer)?;
        Ok(subscribe(self.table(), deadline)?)
    }
}

impl<E: Embedding> wasi::cli::environment::Host for EImpl<E> {
    fn get_arguments(&mut self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    fn get_environment(&mut self) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }
    fn initial_cwd(&mut self) -> Result<Option<String>> {
        Ok(None)
    }
}

impl<E: Embedding> wasi::cli::exit::Host for EImpl<E> {
    fn exit(&mut self, code: Result<(), ()>) -> Result<()> {
        if code.is_ok() {
            bail!("wasi exit success")
        } else {
            bail!("wasi exit error")
        }
    }
}

impl<E: Embedding> wasi::cli::stdin::Host for EImpl<E> {
    fn get_stdin(&mut self) -> Result<Resource<DynInputStream>> {
        let stdin: DynInputStream = Box::new(self.stdin());
        Ok(self.table().push(stdin)?)
    }
}

impl<E: Embedding> wasi::cli::stdout::Host for EImpl<E> {
    fn get_stdout(&mut self) -> Result<Resource<DynOutputStream>> {
        let stdout: DynOutputStream = Box::new(self.stdout());
        Ok(self.table().push(stdout)?)
    }
}

impl<E: Embedding> wasi::cli::stderr::Host for EImpl<E> {
    fn get_stderr(&mut self) -> Result<Resource<DynOutputStream>> {
        let stderr: DynOutputStream = Box::new(self.stderr());
        Ok(self.table().push(stderr)?)
    }
}

impl<E: Embedding> wasi::filesystem::preopens::Host for EImpl<E> {
    fn get_directories(
        &mut self,
    ) -> Result<Vec<(Resource<wasi::filesystem::types::Descriptor>, String)>> {
        // Never construct a Descriptor, so all of the bails in the rest of Filesystem should be
        // unreachable.
        Ok(Vec::new())
    }
}

impl<E: Embedding> wasi::filesystem::types::HostDescriptor for EImpl<E> {
    fn read_via_stream(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: u64,
    ) -> Result<Result<Resource<DynInputStream>, wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn write_via_stream(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: u64,
    ) -> Result<Result<Resource<DynOutputStream>, wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn append_via_stream(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<Result<Resource<DynOutputStream>, wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn advise(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: u64,
        _: u64,
        _: wasi::filesystem::types::Advice,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn sync_data(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn get_flags(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<Result<wasi::filesystem::types::DescriptorFlags, wasi::filesystem::types::ErrorCode>>
    {
        bail!("no filesystem")
    }
    fn get_type(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<Result<wasi::filesystem::types::DescriptorType, wasi::filesystem::types::ErrorCode>>
    {
        bail!("no filesystem")
    }
    fn set_size(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: u64,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn set_times(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: wasi::filesystem::types::NewTimestamp,
        _: wasi::filesystem::types::NewTimestamp,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn read(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: u64,
        _: u64,
    ) -> Result<Result<(Vec<u8>, bool), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn write(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: Vec<u8>,
        _: u64,
    ) -> Result<Result<u64, wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }

    fn read_directory(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<
        Result<
            Resource<wasi::filesystem::types::DirectoryEntryStream>,
            wasi::filesystem::types::ErrorCode,
        >,
    > {
        bail!("no filesystem")
    }
    fn sync(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn create_directory_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn stat(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<Result<wasi::filesystem::types::DescriptorStat, wasi::filesystem::types::ErrorCode>>
    {
        bail!("no filesystem")
    }
    fn stat_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: wasi::filesystem::types::PathFlags,
        _: String,
    ) -> Result<Result<wasi::filesystem::types::DescriptorStat, wasi::filesystem::types::ErrorCode>>
    {
        bail!("no filesystem")
    }
    fn set_times_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: wasi::filesystem::types::PathFlags,
        _: String,
        _: wasi::filesystem::types::NewTimestamp,
        _: wasi::filesystem::types::NewTimestamp,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn link_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: wasi::filesystem::types::PathFlags,
        _: String,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn open_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: wasi::filesystem::types::PathFlags,
        _: String,
        _: wasi::filesystem::types::OpenFlags,
        _: wasi::filesystem::types::DescriptorFlags,
    ) -> Result<
        Result<Resource<wasi::filesystem::types::Descriptor>, wasi::filesystem::types::ErrorCode>,
    > {
        bail!("no filesystem")
    }
    fn readlink_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
    ) -> Result<Result<String, wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn remove_directory_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn rename_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn symlink_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
        _: String,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn unlink_file_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: String,
    ) -> Result<Result<(), wasi::filesystem::types::ErrorCode>> {
        bail!("no filesystem")
    }
    fn is_same_object(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<bool> {
        bail!("no filesystem")
    }
    fn metadata_hash(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
    ) -> Result<
        Result<wasi::filesystem::types::MetadataHashValue, wasi::filesystem::types::ErrorCode>,
    > {
        bail!("no filesystem")
    }
    fn metadata_hash_at(
        &mut self,
        _: Resource<wasi::filesystem::types::Descriptor>,
        _: wasi::filesystem::types::PathFlags,
        _: String,
    ) -> Result<
        Result<wasi::filesystem::types::MetadataHashValue, wasi::filesystem::types::ErrorCode>,
    > {
        bail!("no filesystem")
    }

    fn drop(&mut self, _: Resource<wasi::filesystem::types::Descriptor>) -> Result<()> {
        bail!("no filesystem")
    }
}
impl<E: Embedding> wasi::filesystem::types::HostDirectoryEntryStream for EImpl<E> {
    fn read_directory_entry(
        &mut self,
        _: Resource<wasi::filesystem::types::DirectoryEntryStream>,
    ) -> Result<
        Result<Option<wasi::filesystem::types::DirectoryEntry>, wasi::filesystem::types::ErrorCode>,
    > {
        bail!("no filesystem")
    }
    fn drop(&mut self, _: Resource<wasi::filesystem::types::DirectoryEntryStream>) -> Result<()> {
        bail!("no filesystem")
    }
}
impl<E: Embedding> wasi::filesystem::types::Host for EImpl<E> {
    fn filesystem_error_code(
        &mut self,
        _: Resource<wasmtime_wasi_io::streams::Error>,
    ) -> Result<Option<wasi::filesystem::types::ErrorCode>> {
        Ok(None)
    }
}

impl<E: Embedding> wasi::random::random::Host for EImpl<E> {
    fn get_random_bytes(&mut self, len: u64) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        vec.resize(len as usize, 0u8);
        Ok(vec)
    }
    fn get_random_u64(&mut self) -> Result<u64> {
        Ok(0)
    }
}

impl<E: Embedding> wasi::http::types::HostIncomingRequest for EImpl<E> {
    fn method(
        &mut self,
        _: Resource<wasi::http::types::IncomingRequest>,
    ) -> Result<wasi::http::types::Method> {
        todo!()
    }
    fn path_with_query(
        &mut self,
        _: Resource<wasi::http::types::IncomingRequest>,
    ) -> Result<Option<String>> {
        todo!()
    }
    fn scheme(
        &mut self,
        _: Resource<wasi::http::types::IncomingRequest>,
    ) -> Result<Option<wasi::http::types::Scheme>> {
        todo!()
    }
    fn authority(
        &mut self,
        _: Resource<wasi::http::types::IncomingRequest>,
    ) -> Result<Option<String>> {
        todo!()
    }
    fn headers(
        &mut self,
        _: Resource<wasi::http::types::IncomingRequest>,
    ) -> Result<Resource<wasi::http::types::Headers>> {
        todo!()
    }
    fn consume(
        &mut self,
        _: Resource<wasi::http::types::IncomingRequest>,
    ) -> Result<Result<Resource<wasi::http::types::IncomingBody>, ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::IncomingRequest>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostOutgoingResponse for EImpl<E> {
    fn new(
        &mut self,
        _: Resource<wasi::http::types::Headers>,
    ) -> Result<Resource<wasi::http::types::OutgoingResponse>> {
        todo!()
    }
    fn status_code(
        &mut self,
        _: Resource<wasi::http::types::OutgoingResponse>,
    ) -> Result<wasi::http::types::StatusCode> {
        todo!()
    }
    fn set_status_code(
        &mut self,
        _: Resource<wasi::http::types::OutgoingResponse>,
        _: wasi::http::types::StatusCode,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn headers(
        &mut self,
        _: Resource<wasi::http::types::OutgoingResponse>,
    ) -> Result<Resource<wasi::http::types::Headers>> {
        todo!()
    }
    fn body(
        &mut self,
        _: Resource<wasi::http::types::OutgoingResponse>,
    ) -> Result<Result<Resource<wasi::http::types::OutgoingBody>, ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::OutgoingResponse>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostOutgoingRequest for EImpl<E> {
    fn new(
        &mut self,
        _: Resource<wasi::http::types::Headers>,
    ) -> Result<Resource<wasi::http::types::OutgoingRequest>> {
        todo!()
    }
    fn body(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
    ) -> Result<Result<Resource<wasi::http::types::OutgoingBody>, ()>> {
        todo!()
    }
    fn method(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
    ) -> Result<wasi::http::types::Method> {
        todo!()
    }
    fn set_method(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
        _: wasi::http::types::Method,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn path_with_query(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
    ) -> Result<Option<String>> {
        todo!()
    }
    fn set_path_with_query(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
        _: Option<String>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn scheme(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
    ) -> Result<Option<wasi::http::types::Scheme>> {
        todo!()
    }
    fn set_scheme(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
        _: Option<wasi::http::types::Scheme>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn authority(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
    ) -> Result<Option<String>> {
        todo!()
    }
    fn set_authority(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
        _: Option<String>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn headers(
        &mut self,
        _: Resource<wasi::http::types::OutgoingRequest>,
    ) -> Result<Resource<wasi::http::types::Headers>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::OutgoingRequest>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostIncomingResponse for EImpl<E> {
    fn status(
        &mut self,
        _: Resource<wasi::http::types::IncomingResponse>,
    ) -> Result<wasi::http::types::StatusCode> {
        todo!()
    }
    fn headers(
        &mut self,
        _: Resource<wasi::http::types::IncomingResponse>,
    ) -> Result<Resource<wasi::http::types::Headers>> {
        todo!()
    }
    fn consume(
        &mut self,
        _: Resource<wasi::http::types::IncomingResponse>,
    ) -> Result<Result<Resource<wasi::http::types::IncomingBody>, ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::IncomingResponse>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostFutureIncomingResponse for EImpl<E> {
    fn subscribe(
        &mut self,
        _: Resource<wasi::http::types::FutureIncomingResponse>,
    ) -> Result<Resource<DynPollable>> {
        todo!()
    }
    fn get(
        &mut self,
        _: Resource<wasi::http::types::FutureIncomingResponse>,
    ) -> Result<
        Option<
            Result<
                Result<Resource<wasi::http::types::IncomingResponse>, wasi::http::types::ErrorCode>,
                (),
            >,
        >,
    > {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::FutureIncomingResponse>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostIncomingBody for EImpl<E> {
    fn stream(
        &mut self,
        _: Resource<wasi::http::types::IncomingBody>,
    ) -> Result<Result<Resource<DynInputStream>, ()>> {
        todo!()
    }
    fn finish(
        &mut self,
        _: Resource<wasi::http::types::IncomingBody>,
    ) -> Result<Resource<wasi::http::types::FutureTrailers>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::IncomingBody>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostOutgoingBody for EImpl<E> {
    fn write(
        &mut self,
        _: Resource<wasi::http::types::OutgoingBody>,
    ) -> Result<Result<Resource<DynOutputStream>, ()>> {
        todo!()
    }
    fn finish(
        &mut self,
        _: Resource<wasi::http::types::OutgoingBody>,
        _: Option<Resource<wasi::http::types::Trailers>>,
    ) -> Result<Result<(), wasi::http::types::ErrorCode>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::OutgoingBody>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostFields for EImpl<E> {
    fn new(&mut self) -> Result<Resource<wasi::http::types::Fields>> {
        todo!()
    }
    fn from_list(
        &mut self,
        _: Vec<(wasi::http::types::FieldKey, wasi::http::types::FieldValue)>,
    ) -> Result<Result<Resource<wasi::http::types::Fields>, wasi::http::types::HeaderError>> {
        todo!()
    }
    fn get(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
        _: wasi::http::types::FieldKey,
    ) -> Result<Vec<wasi::http::types::FieldValue>> {
        todo!()
    }
    fn has(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
        _: wasi::http::types::FieldKey,
    ) -> Result<bool> {
        todo!()
    }
    fn set(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
        _: wasi::http::types::FieldKey,
        _: Vec<wasi::http::types::FieldValue>,
    ) -> Result<Result<(), wasi::http::types::HeaderError>> {
        todo!()
    }
    fn delete(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
        _: wasi::http::types::FieldKey,
    ) -> Result<Result<(), wasi::http::types::HeaderError>> {
        todo!()
    }
    fn append(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
        _: wasi::http::types::FieldKey,
        _: wasi::http::types::FieldValue,
    ) -> Result<Result<(), wasi::http::types::HeaderError>> {
        todo!()
    }
    fn entries(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
    ) -> Result<Vec<(wasi::http::types::FieldKey, wasi::http::types::FieldValue)>> {
        todo!()
    }
    fn clone(
        &mut self,
        _: Resource<wasi::http::types::Fields>,
    ) -> Result<Resource<wasi::http::types::Fields>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::Fields>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostFutureTrailers for EImpl<E> {
    fn subscribe(
        &mut self,
        _: Resource<wasi::http::types::FutureTrailers>,
    ) -> Result<Resource<DynPollable>> {
        todo!()
    }
    fn get(
        &mut self,
        _: Resource<wasi::http::types::FutureTrailers>,
    ) -> Result<
        Option<
            Result<
                Result<Option<Resource<wasi::http::types::Trailers>>, wasi::http::types::ErrorCode>,
                (),
            >,
        >,
    > {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::FutureTrailers>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostResponseOutparam for EImpl<E> {
    fn set(
        &mut self,
        _: Resource<wasi::http::types::ResponseOutparam>,
        _: Result<Resource<wasi::http::types::OutgoingResponse>, wasi::http::types::ErrorCode>,
    ) -> Result<()> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::ResponseOutparam>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::HostRequestOptions for EImpl<E> {
    fn new(&mut self) -> Result<Resource<wasi::http::types::RequestOptions>> {
        todo!()
    }
    fn connect_timeout(
        &mut self,
        _: Resource<wasi::http::types::RequestOptions>,
    ) -> Result<Option<wasi::clocks::monotonic_clock::Duration>> {
        todo!()
    }
    fn set_connect_timeout(
        &mut self,
        _: Resource<wasi::http::types::RequestOptions>,
        _: Option<wasi::clocks::monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn first_byte_timeout(
        &mut self,
        _: Resource<wasi::http::types::RequestOptions>,
    ) -> Result<Option<wasi::clocks::monotonic_clock::Duration>> {
        todo!()
    }
    fn set_first_byte_timeout(
        &mut self,
        _: Resource<wasi::http::types::RequestOptions>,
        _: Option<wasi::clocks::monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn between_bytes_timeout(
        &mut self,
        _: Resource<wasi::http::types::RequestOptions>,
    ) -> Result<Option<wasi::clocks::monotonic_clock::Duration>> {
        todo!()
    }
    fn set_between_bytes_timeout(
        &mut self,
        _: Resource<wasi::http::types::RequestOptions>,
        _: Option<wasi::clocks::monotonic_clock::Duration>,
    ) -> Result<Result<(), ()>> {
        todo!()
    }
    fn drop(&mut self, _: Resource<wasi::http::types::RequestOptions>) -> Result<()> {
        todo!()
    }
}
impl<E: Embedding> wasi::http::types::Host for EImpl<E> {
    fn http_error_code(
        &mut self,
        _: Resource<wasmtime_wasi_io::streams::Error>,
    ) -> Result<Option<wasi::http::types::ErrorCode>> {
        todo!()
    }
}
