//! Minimal hand-rolled D-Bus client - just enough wire protocol for rtkit and
//! the realtime portal: connect to a bus over a unix socket, authenticate
//! (`AUTH EXTERNAL`), say `Hello`, and exchange fixed-signature method calls.
//!
//! NOTE(linux): a D-Bus crate dependency would pull an async runtime or a C
//! library into a zero-dep crate for two method calls. The subset implemented
//! here is the complete protocol surface those calls need: marshalling for
//! `y`/`i`/`u`/`t`/`x`/`s`/`o`/`g`/`v`, method-call/return/error messages,
//! little-endian only (replies arrive in the sender's byte order; every
//! supported target is little-endian, and a big-endian reply is rejected
//! loudly rather than misparsed).

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

const METHOD_CALL: u8 = 1;
const METHOD_RETURN: u8 = 2;
const ERROR: u8 = 3;

const FIELD_PATH: u8 = 1;
const FIELD_INTERFACE: u8 = 2;
const FIELD_MEMBER: u8 = 3;
const FIELD_ERROR_NAME: u8 = 4;
const FIELD_REPLY_SERIAL: u8 = 5;
const FIELD_DESTINATION: u8 = 6;
const FIELD_SIGNATURE: u8 = 8;

/// Local daemons answer in microseconds; the timeout exists so a wedged bus
/// can never hang a thread-priority call indefinitely.
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const CALL_DEADLINE: Duration = Duration::from_secs(10);
const MAX_MESSAGE: usize = 1 << 20;

/// Which message bus to connect to.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum BusKind {
    /// The system bus - rtkit lives here.
    System,
    /// The session bus - the xdg desktop portal lives here.
    Session,
}

/// Why a D-Bus method call failed. The variants exist to answer one question for
/// the caller: could the same call succeed if tried again later?
///
/// - `Absent` / `Io`: no -- there is nothing working to talk to.
/// - `TimedOut`: maybe -- the peer is reachable but did not answer in time.
/// - `Refused`: depends on the error name (a rate limit clears; a denial does not).
#[derive(Debug)]
pub(crate) enum CallError {
    /// Nothing is listening: the socket is missing or refused the connection, or
    /// the bus reports no such service (`ServiceUnknown` / `NameHasNoOwner`). The
    /// daemon is not running, so retrying changes nothing.
    Absent,
    /// The peer never answered within the timeout -- a busy bus or an overloaded
    /// daemon. It is reachable, just slow, so a later call may still go through.
    TimedOut,
    /// The service answered, but with an error: permission denied, bad arguments,
    /// a rate limit, and so on. `name` is the D-Bus error name; the caller reads
    /// it to decide what to do (back off on a rate limit, give up on a denial).
    Refused {
        /// Error name, e.g. `org.freedesktop.DBus.Error.AccessDenied`.
        name: String,
    },
    /// Transport broke some other way: a corrupt or unsupported (big-endian /
    /// oversize) reply, or a failure during connect / auth / `Hello`. Like
    /// `Absent`, there is nothing usable to talk to.
    Io(std::io::Error),
}

/// Sorts a raw connect/read/write error into the matching `CallError`: a missing
/// or refused socket means nothing is there (`Absent`); a timeout or would-block
/// means the peer is merely slow (`TimedOut`); anything else stays `Io`.
fn classify_transport(e: std::io::Error) -> CallError {
    use std::io::ErrorKind::{ConnectionRefused, NotFound, TimedOut, WouldBlock};

    match e.kind() {
        NotFound | ConnectionRefused => CallError::Absent,
        TimedOut | WouldBlock => CallError::TimedOut,
        _ => CallError::Io(e),
    }
}

impl From<std::io::Error> for CallError {
    fn from(e: std::io::Error) -> Self {
        classify_transport(e)
    }
}

impl std::fmt::Display for CallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallError::Absent => write!(f, "D-Bus service absent"),
            CallError::TimedOut => write!(f, "D-Bus call timed out"),
            CallError::Refused { name } => write!(f, "D-Bus error {name}"),
            CallError::Io(e) => write!(f, "D-Bus I/O error: {e}"),
        }
    }
}

/// A method-call argument. The signature is derived from the argument list.
#[derive(Debug, Copy, Clone)]
pub(crate) enum Arg<'a> {
    Str(&'a str),
    U32(u32),
    I32(i32),
    U64(u64),
}

/// An authenticated connection to a message bus, ready for method calls.
pub(crate) struct Connection {
    stream: UnixStream,
    serial: u32,
}

impl Connection {
    /// Connects, authenticates (`AUTH EXTERNAL` with our uid), and registers
    /// with the bus (`Hello`). A connect or socket failure becomes the matching
    /// [`CallError`] (so a missing daemon surfaces as `Absent`, not a raw I/O
    /// error); a rejected `Hello` surfaces as whatever `call` returns.
    pub(crate) fn open(kind: BusKind) -> Result<Self, CallError> {
        let stream = connect_bus_socket(kind)?;

        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;

        let mut conn = Connection { stream, serial: 0 };

        conn.authenticate(Instant::now() + CALL_DEADLINE)?;
        conn.call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
            &[],
        )?;

        Ok(conn)
    }

    fn authenticate(&mut self, deadline: Instant) -> std::io::Result<()> {
        // SAFETY: getuid is always safe to call and cannot fail.
        let uid = unsafe { libc::getuid() };

        let uid_hex: String = uid
            .to_string()
            .bytes()
            .map(|b| format!("{:02x}", b))
            .collect();

        self.stream
            .write_all(format!("\0AUTH EXTERNAL {}\r\n", uid_hex).as_bytes())?;

        let line = self.read_auth_line(deadline)?;
        if line.starts_with("OK ") {
            self.stream.write_all(b"BEGIN\r\n")?;
            return Ok(());
        }

        // EXTERNAL refused (`REJECTED <mechs>`). Fall back to ANONYMOUS, which
        // some bus configurations accept; the leading NUL was already sent, so a
        // bare `AUTH ANONYMOUS` (no NUL) continues the same auth conversation.
        if line.starts_with("REJECTED") {
            self.stream.write_all(b"AUTH ANONYMOUS\r\n")?;

            let retry = self.read_auth_line(deadline)?;

            if retry.starts_with("OK ") {
                self.stream.write_all(b"BEGIN\r\n")?;
                return Ok(());
            }

            return Err(std::io::Error::other(format!(
                "D-Bus AUTH EXTERNAL and ANONYMOUS both rejected: {}",
                retry.trim_end()
            )));
        }

        Err(std::io::Error::other(format!(
            "D-Bus AUTH EXTERNAL rejected: {}",
            line.trim_end()
        )))
    }

    fn read_auth_line(&mut self, deadline: Instant) -> std::io::Result<String> {
        let mut line = Vec::new();
        let mut byte = [0u8; 1];

        while line.len() < 4096 {
            self.read_exact_until(&mut byte, deadline)?;

            if byte[0] == b'\n' {
                return Ok(String::from_utf8_lossy(&line).into_owned());
            }

            if byte[0] != b'\r' {
                line.push(byte[0]);
            }
        }

        Err(std::io::Error::other("D-Bus auth line too long"))
    }

    fn remaining(deadline: Instant) -> std::io::Result<Duration> {
        deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::TimedOut, "D-Bus deadline expired")
            })
    }

    fn read_exact_until(&mut self, buf: &mut [u8], deadline: Instant) -> std::io::Result<()> {
        let mut offset = 0usize;

        while offset < buf.len() {
            let remaining = Self::remaining(deadline)?;

            self.stream
                .set_read_timeout(Some(remaining.min(IO_TIMEOUT)))?;

            match self.stream.read(&mut buf[offset..]) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "D-Bus stream closed",
                    ));
                }
                Ok(n) => offset += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Sends a method call and waits for its reply, skipping unrelated
    /// messages (the bus pushes signals like `NameAcquired` unprompted).
    /// Returns the raw reply body.
    pub(crate) fn call(
        &mut self,
        destination: &str,
        path: &str,
        interface: &str,
        member: &str,
        args: &[Arg<'_>],
    ) -> Result<Vec<u8>, CallError> {
        self.serial += 1;

        let serial = self.serial;
        let (signature, body) = marshal_args(args);
        let message = build_method_call(
            serial,
            destination,
            path,
            interface,
            member,
            &signature,
            &body,
        );

        self.stream.write_all(&message)?;

        // Bound the wait. A hostile or buggy peer can stream well-framed messages
        // whose reply_serial never matches; each read succeeds within IO_TIMEOUT,
        // so the per-read timeout alone never fires and the loop livelocks. Cap
        // BOTH the number of unrelated messages and the cumulative wall-clock time.
        const MAX_REPLY_SKIPS: usize = 64;

        let deadline = Instant::now() + CALL_DEADLINE;
        let mut skipped = 0usize;

        loop {
            if Instant::now() >= deadline {
                return Err(CallError::TimedOut);
            }

            let msg = self.read_message(deadline)?;

            if msg.reply_serial == Some(serial) {
                match msg.msg_type {
                    METHOD_RETURN => return Ok(msg.body),
                    ERROR => {
                        // ServiceUnknown / NameHasNoOwner mean the name has no
                        // owner -- the service is not running (`Absent`). Every
                        // other error is the service itself refusing the call; we
                        // keep its name so the caller can tell denied from
                        // rate-limited.
                        let name = msg.error_name.unwrap_or_default();

                        if name == "org.freedesktop.DBus.Error.ServiceUnknown"
                            || name == "org.freedesktop.DBus.Error.NameHasNoOwner"
                        {
                            return Err(CallError::Absent);
                        }

                        return Err(CallError::Refused { name });
                    }
                    _ => {}
                }
            }

            skipped += 1;

            if skipped > MAX_REPLY_SKIPS {
                // We gave up after too many unrelated messages. The peer is
                // talking, just not answering us, so this is a stall (`TimedOut`),
                // not a refusal.
                return Err(CallError::TimedOut);
            }
        }
    }

    fn read_message(&mut self, deadline: Instant) -> std::io::Result<IncomingMessage> {
        let mut fixed = [0u8; 16];

        self.read_exact_until(&mut fixed, deadline)?;

        if fixed[0] != b'l' {
            return Err(std::io::Error::other(
                "big-endian D-Bus reply - unsupported by this client",
            ));
        }

        let body_len = u32::from_le_bytes(fixed[4..8].try_into().unwrap()) as usize;
        let fields_len = u32::from_le_bytes(fixed[12..16].try_into().unwrap()) as usize;
        let fields_padded = fields_len.div_ceil(8).saturating_mul(8);

        if body_len > MAX_MESSAGE || fields_padded > MAX_MESSAGE {
            return Err(std::io::Error::other("D-Bus message too large"));
        }

        let mut fields = vec![0u8; fields_padded];
        self.read_exact_until(&mut fields, deadline)?;

        let mut body = vec![0u8; body_len];
        self.read_exact_until(&mut body, deadline)?;

        let mut msg = IncomingMessage {
            msg_type: fixed[1],
            reply_serial: None,
            error_name: None,
            body,
        };

        // Header fields: a(yv). The array data starts at message offset 16,
        // so alignment within `fields` matches alignment within the message.
        let mut r = Reader::new(&fields[..fields_len]);
        while r.pos < fields_len {
            r.align(8)?;

            if r.pos >= fields_len {
                break;
            }

            let code = r.u8()?;
            let sig = r.signature()?;

            match (code, sig.as_str()) {
                (FIELD_REPLY_SERIAL, "u") => msg.reply_serial = Some(r.u32()?),
                (FIELD_ERROR_NAME, "s") => msg.error_name = Some(r.string()?),
                _ => r.skip_value(&sig)?,
            }
        }

        Ok(msg)
    }
}

struct IncomingMessage {
    msg_type: u8,
    reply_serial: Option<u32>,
    error_name: Option<String>,
    body: Vec<u8>,
}

fn connect_bus_socket(kind: BusKind) -> std::io::Result<UnixStream> {
    let address = match kind {
        BusKind::System => std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
            .unwrap_or_else(|_| "unix:path=/run/dbus/system_bus_socket".to_string()),
        BusKind::Session => std::env::var("DBUS_SESSION_BUS_ADDRESS").map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "DBUS_SESSION_BUS_ADDRESS is not set",
            )
        })?,
    };

    // Address format: "unix:path=/run/...,guid=...;fallback2;...". Only path=
    // and abstract= transports are supported.
    let mut last_error = None;
    let mut saw_unix_transport = false;

    for entry in address.split(';') {
        let Some(rest) = entry.strip_prefix("unix:") else {
            continue;
        };

        for kv in rest.split(',') {
            let result = if let Some(path) = kv.strip_prefix("path=") {
                saw_unix_transport = true;
                UnixStream::connect(path)
            } else if let Some(name) = kv.strip_prefix("abstract=") {
                use std::os::linux::net::SocketAddrExt;
                saw_unix_transport = true;
                std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes())
                    .and_then(|addr| UnixStream::connect_addr(&addr))
            } else {
                continue;
            };

            match result {
                Ok(stream) => return Ok(stream),
                Err(err) => last_error = Some(err),
            }
        }
    }

    if let Some(err) = last_error {
        return Err(err);
    }

    if saw_unix_transport {
        return Err(std::io::Error::other(format!(
            "no reachable unix transport in D-Bus address: {}",
            address
        )));
    }

    Err(std::io::Error::other(format!(
        "no usable unix transport in D-Bus address: {}",
        address
    )))
}

// --- Marshalling -----------------------------------------------------------

#[derive(Default)]
struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn align(&mut self, n: usize) {
        while !self.buf.len().is_multiple_of(n) {
            self.buf.push(0);
        }
    }

    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    fn u32(&mut self, v: u32) {
        self.align(4);
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn i32(&mut self, v: i32) {
        self.align(4);
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn u64(&mut self, v: u64) {
        self.align(8);
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn string(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }

    fn signature(&mut self, s: &str) {
        self.u8(s.len() as u8);
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }
}

fn marshal_args(args: &[Arg<'_>]) -> (String, Vec<u8>) {
    let mut signature = String::new();
    let mut w = Writer::default();

    for arg in args {
        match arg {
            Arg::Str(s) => {
                signature.push('s');
                w.string(s);
            }
            Arg::U32(v) => {
                signature.push('u');
                w.u32(*v);
            }
            Arg::I32(v) => {
                signature.push('i');
                w.i32(*v);
            }
            Arg::U64(v) => {
                signature.push('t');
                w.u64(*v);
            }
        }
    }

    (signature, w.buf)
}

fn build_method_call(
    serial: u32,
    destination: &str,
    path: &str,
    interface: &str,
    member: &str,
    signature: &str,
    body: &[u8],
) -> Vec<u8> {
    let mut w = Writer::default();

    w.u8(b'l');
    w.u8(METHOD_CALL);
    w.u8(0); // flags
    w.u8(1); // protocol version
    w.u32(body.len() as u32);
    w.u32(serial);
    w.u32(0); // header-fields array length, patched below

    // Field structs (yv) align to 8; the array data starts at offset 16.
    let fields_start = w.buf.len();

    header_field_string(&mut w, FIELD_PATH, 'o', path);
    header_field_string(&mut w, FIELD_DESTINATION, 's', destination);
    header_field_string(&mut w, FIELD_INTERFACE, 's', interface);
    header_field_string(&mut w, FIELD_MEMBER, 's', member);

    if !signature.is_empty() {
        w.align(8);
        w.u8(FIELD_SIGNATURE);
        w.signature("g");
        w.signature(signature);
    }

    let fields_len = (w.buf.len() - fields_start) as u32;
    w.buf[12..16].copy_from_slice(&fields_len.to_le_bytes());

    // The body always starts on an 8-byte boundary, so alignment computed
    // relative to the body's own start (as `marshal_args` does) agrees with
    // the message-global alignment the wire format requires.
    w.align(8);
    w.buf.extend_from_slice(body);

    w.buf
}

fn header_field_string(w: &mut Writer, code: u8, type_char: char, value: &str) {
    w.align(8);
    w.u8(code);

    let mut sig = [0u8; 4];

    w.signature(type_char.encode_utf8(&mut sig));
    w.string(value);
}

// --- Unmarshalling ---------------------------------------------------------

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }

    fn align(&mut self, n: usize) -> std::io::Result<()> {
        while !self.pos.is_multiple_of(n) {
            self.pos += 1;
        }

        if self.pos > self.buf.len() {
            return Err(truncated());
        }

        Ok(())
    }

    fn bytes(&mut self, n: usize) -> std::io::Result<&'a [u8]> {
        // checked_add: `n` can be a wire-supplied u32 length; on a 32-bit target
        // `pos + n` could wrap and pass the bounds check, so add explicitly.
        let end = self.pos.checked_add(n).ok_or_else(truncated)?;

        if end > self.buf.len() {
            return Err(truncated());
        }

        let out = &self.buf[self.pos..end];
        self.pos = end;

        Ok(out)
    }
    fn u8(&mut self) -> std::io::Result<u8> {
        Ok(self.bytes(1)?[0])
    }

    fn u16(&mut self) -> std::io::Result<u16> {
        self.align(2)?;

        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> std::io::Result<u32> {
        self.align(4)?;

        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> std::io::Result<u64> {
        self.align(8)?;

        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }

    fn string(&mut self) -> std::io::Result<String> {
        let len = self.u32()? as usize;
        let s = String::from_utf8_lossy(self.bytes(len)?).into_owned();

        self.bytes(1)?; // nul terminator

        Ok(s)
    }

    fn signature(&mut self) -> std::io::Result<String> {
        let len = self.u8()? as usize;
        let s = String::from_utf8_lossy(self.bytes(len)?).into_owned();

        self.bytes(1)?; // nul terminator

        Ok(s)
    }

    /// Skips one value of a basic type. Container types never appear in the
    /// places this client reads (header-field variants and property replies).
    fn skip_value(&mut self, sig: &str) -> std::io::Result<()> {
        // This client only ever skips ONE basic-typed value (an unknown header
        // field's variant). A multi-type or container signature would desync the
        // parser if we skipped only its first type, so reject it outright.
        if sig.len() != 1 {
            return Err(std::io::Error::other(format!(
                "unexpected multi-type/container D-Bus signature in reply: {}",
                sig
            )));
        }

        match sig.as_bytes().first() {
            Some(b'y') => {
                self.u8()?;
            }
            Some(b'n' | b'q') => {
                self.u16()?;
            }
            Some(b'b' | b'i' | b'u') => {
                self.u32()?;
            }
            Some(b'x' | b't' | b'd') => {
                self.u64()?;
            }
            Some(b's' | b'o') => {
                self.string()?;
            }
            Some(b'g') => {
                self.signature()?;
            }
            _ => {
                return Err(std::io::Error::other(format!(
                    "unsupported D-Bus type in reply: {}",
                    sig
                )));
            }
        }

        Ok(())
    }
}

fn truncated() -> std::io::Error {
    std::io::Error::other("truncated D-Bus message")
}

/// Unwraps a reply body of signature `v` holding an integer (any width).
pub(crate) fn parse_variant_int(body: &[u8]) -> Option<i64> {
    let mut r = Reader::new(body);
    let sig = r.signature().ok()?;

    match sig.as_str() {
        "y" => r.u8().ok().map(i64::from),
        "n" => r.u16().ok().map(|v| i64::from(v as i16)),
        "q" => r.u16().ok().map(i64::from),
        "b" | "u" => r.u32().ok().map(i64::from),
        "i" => r.u32().ok().map(|v| i64::from(v as i32)),
        "x" => r.u64().ok().map(|v| v as i64),
        "t" => r.u64().ok().map(|v| v as i64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_call_wire_format() {
        let (sig, body) = marshal_args(&[Arg::U64(0x1122334455667788), Arg::I32(-5)]);
        assert_eq!(sig, "ti");
        assert_eq!(body.len(), 12);
        assert_eq!(&body[0..8], &0x1122334455667788u64.to_le_bytes());
        assert_eq!(&body[8..12], &(-5i32).to_le_bytes());

        let msg = build_method_call(
            7,
            "org.freedesktop.RealtimeKit1",
            "/org/freedesktop/RealtimeKit1",
            "org.freedesktop.RealtimeKit1",
            "MakeThreadHighPriority",
            &sig,
            &body,
        );
        assert_eq!(msg[0], b'l');
        assert_eq!(msg[1], METHOD_CALL);
        let body_len = u32::from_le_bytes(msg[4..8].try_into().unwrap()) as usize;
        assert_eq!(body_len, body.len());
        let fields_len = u32::from_le_bytes(msg[12..16].try_into().unwrap()) as usize;
        let body_start = 16 + fields_len.div_ceil(8) * 8;
        assert_eq!(&msg[body_start..], &body[..]);
        // Body must start 8-aligned for body-relative marshalling to hold.
        assert!(body_start.is_multiple_of(8));
    }

    #[test]
    fn variant_int_widths() {
        // Variant: signature then aligned value.
        let mut w = Writer::default();
        w.signature("i");
        w.i32(-15);
        assert_eq!(parse_variant_int(&w.buf), Some(-15));

        let mut w = Writer::default();
        w.signature("x");
        w.u64((-200_000i64) as u64);
        assert_eq!(parse_variant_int(&w.buf), Some(-200_000));

        let mut w = Writer::default();
        w.signature("t");
        w.u64(200_000);
        assert_eq!(parse_variant_int(&w.buf), Some(200_000));
    }
}
