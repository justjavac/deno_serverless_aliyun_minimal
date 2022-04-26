use deno_core::anyhow::Error;
use deno_core::located_script_name;
use deno_core::op;
use deno_core::AsyncRefCell;
use deno_core::AsyncResult;
use deno_core::CancelHandle;
use deno_core::CancelTryFuture;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::RcRef;
use deno_core::Resource;
use deno_core::ResourceId;
use deno_core::Snapshot;
use deno_core::ZeroCopyBuf;
use std::cell::RefCell;
use std::env;
use std::net::SocketAddr;
use std::rc::Rc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

static CTSR_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/FC_RUNTIME_SNAPSHOT.bin"));

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

// Note: a `tokio::net::TcpListener` doesn't need to be wrapped in a cell,
// because it only supports one op (`accept`) which does not require a mutable
// reference to the listener.
struct TcpListener {
    inner: tokio::net::TcpListener,
    cancel: CancelHandle,
}

impl TcpListener {
    async fn accept(self: Rc<Self>) -> Result<TcpStream, std::io::Error> {
        let cancel = RcRef::map(&self, |r| &r.cancel);
        let stream = self.inner.accept().try_or_cancel(cancel).await?.0.into();
        Ok(stream)
    }
}

impl Resource for TcpListener {
    fn close(self: Rc<Self>) {
        self.cancel.cancel();
    }
}

impl TryFrom<std::net::TcpListener> for TcpListener {
    type Error = std::io::Error;
    fn try_from(std_listener: std::net::TcpListener) -> Result<Self, Self::Error> {
        tokio::net::TcpListener::try_from(std_listener).map(|tokio_listener| Self {
            inner: tokio_listener,
            cancel: Default::default(),
        })
    }
}

struct TcpStream {
    rd: AsyncRefCell<tokio::net::tcp::OwnedReadHalf>,
    wr: AsyncRefCell<tokio::net::tcp::OwnedWriteHalf>,
    // When a `TcpStream` resource is closed, all pending 'read' ops are
    // canceled, while 'write' ops are allowed to complete. Therefore only
    // 'read' futures are attached to this cancel handle.
    cancel: CancelHandle,
}

impl TcpStream {
    async fn read(self: Rc<Self>, mut buf: ZeroCopyBuf) -> Result<(usize, ZeroCopyBuf), Error> {
        let mut rd = RcRef::map(&self, |r| &r.rd).borrow_mut().await;
        let cancel = RcRef::map(self, |r| &r.cancel);
        let nread = rd
            .read(&mut buf)
            .try_or_cancel(cancel)
            .await
            .map_err(Error::from)?;
        Ok((nread, buf))
    }

    async fn write(self: Rc<Self>, buf: ZeroCopyBuf) -> Result<usize, Error> {
        let mut wr = RcRef::map(self, |r| &r.wr).borrow_mut().await;
        wr.write(&buf).await.map_err(Error::from)
    }
}

impl Resource for TcpStream {
    fn read_return(self: Rc<Self>, buf: ZeroCopyBuf) -> AsyncResult<(usize, ZeroCopyBuf)> {
        Box::pin(self.read(buf))
    }

    fn write(self: Rc<Self>, buf: ZeroCopyBuf) -> AsyncResult<usize> {
        Box::pin(self.write(buf))
    }

    fn close(self: Rc<Self>) {
        self.cancel.cancel()
    }
}

impl From<tokio::net::TcpStream> for TcpStream {
    fn from(s: tokio::net::TcpStream) -> Self {
        let (rd, wr) = s.into_split();
        Self {
            rd: rd.into(),
            wr: wr.into(),
            cancel: Default::default(),
        }
    }
}

fn create_js_runtime() -> JsRuntime {
    let ext = deno_core::Extension::builder()
        .ops(vec![op_listen::decl(), op_accept::decl()])
        .build();

    JsRuntime::new(deno_core::RuntimeOptions {
        startup_snapshot: Some(Snapshot::Static(CTSR_SNAPSHOT)),
        extensions: vec![ext],
        ..Default::default()
    })
}

#[op]
fn op_listen(state: &mut OpState) -> Result<ResourceId, Error> {
    log::debug!("listen");
    let addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
    let std_listener = std::net::TcpListener::bind(&addr)?;
    std_listener.set_nonblocking(true)?;
    // std_listener.set_ttl(0).expect("could not set TTL");
    let listener = TcpListener::try_from(std_listener)?;
    let rid = state.resource_table.add(listener);
    Ok(rid)
}

#[op]
async fn op_accept(state: Rc<RefCell<OpState>>, rid: ResourceId) -> Result<ResourceId, Error> {
    log::debug!("accept rid={}", rid);

    let listener = state.borrow().resource_table.get::<TcpListener>(rid)?;
    let stream = listener.accept().await?;
    let rid = state.borrow_mut().resource_table.add(stream);
    Ok(rid)
}

fn main() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(
        env::args()
            .find(|a| a == "-D")
            .map(|_| log::LevelFilter::Debug)
            .unwrap_or(log::LevelFilter::Warn),
    );

    // NOTE: `--help` arg will display V8 help and exit
    deno_core::v8_set_flags(env::args().collect());

    let mut js_runtime = create_js_runtime();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let future = async move {
        js_runtime
            .execute_script(&located_script_name!(), "main()")
            .unwrap();
        js_runtime.run_event_loop(false).await
    };
    runtime.block_on(future).unwrap();
}
