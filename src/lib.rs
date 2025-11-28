mod protocols;
mod types;
mod utils;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::sync::OnceLock;

// 重新导出公共类型
pub use protocols::icmp::ping::async_ping::AsyncPinger;
pub use protocols::icmp::ping::sync::Pinger;
pub use protocols::icmp::stream::async_stream::AsyncPingStream;
pub use protocols::icmp::stream::sync::PingStream;
pub use types::result::PingResult;

// =================== 模块级函数 ===================

/// 创建非阻塞 ping 流
#[pyfunction]
#[pyo3(signature = (target, interval_ms=1000, interface=None, ipv4=false, ipv6=false, count=None))]
fn create_ping_stream(
    target: &Bound<PyAny>,
    interval_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
    count: Option<usize>,
) -> PyResult<PingStream> {
    // 直接使用 PingStream 的构造函数
    PingStream::new(target, interval_ms, interface, ipv4, ipv6, count)
}

/// 执行单次 ping（同步版本）
///
/// # 参数
/// - `timeout_ms`: 等待响应的超时时间（毫秒），默认 1000ms
///   注意：内部实现中，这个值会被用作 `interval_ms` 传递给底层 ping 命令
#[pyfunction]
#[pyo3(signature = (target, timeout_ms=1000, interface=None, ipv4=false, ipv6=false))]
fn ping_once(
    target: &Bound<PyAny>,
    timeout_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<PingResult> {
    // 创建 Pinger 实例
    // 注意：这里将 timeout_ms 作为 interval_ms 传递，因为 ping_once 中会将其用作超时时间
    let pinger = Pinger::new(target, timeout_ms, interface, ipv4, ipv6)?;

    // 执行 ping_once
    pinger.ping_once()
}

/// 执行单次 ping（异步版本）
///
/// # 参数
/// - `timeout_ms`: 等待响应的超时时间（毫秒），默认 1000ms
///   注意：内部实现中，这个值会被用作 `interval_ms` 传递给底层 ping 命令
#[pyfunction]
#[pyo3(signature = (target, timeout_ms=1000, interface=None, ipv4=false, ipv6=false))]
fn ping_once_async<'py>(
    py: Python<'py>,
    target: &Bound<PyAny>,
    timeout_ms: i64,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<Bound<'py, PyAny>> {
    // 创建 AsyncPinger 实例
    // 注意：这里将 timeout_ms 作为 interval_ms 传递，因为 ping_once 中会将其用作超时时间
    let pinger = AsyncPinger::new(target, timeout_ms, interface, ipv4, ipv6)?;

    // 执行异步 ping_once
    pinger.ping_once(py)
}

/// 执行多次 ping（同步版本）
#[pyfunction]
#[pyo3(signature = (target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=false, ipv6=false))]
fn ping_multiple(
    target: &Bound<PyAny>,
    count: i32,
    interval_ms: i64,
    timeout_ms: Option<i64>,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<Vec<PingResult>> {
    // 创建 Pinger 实例
    let pinger = Pinger::new(target, interval_ms, interface, ipv4, ipv6)?;

    // 执行 ping_multiple
    pinger.ping_multiple(count, timeout_ms)
}

/// 执行多次 ping（异步版本）
#[pyfunction]
#[pyo3(signature = (target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=false, ipv6=false))]
#[allow(clippy::too_many_arguments)] // 添加允许多参数的属性
fn ping_multiple_async<'py>(
    py: Python<'py>,
    target: &Bound<PyAny>,
    count: i32,
    interval_ms: i64,
    timeout_ms: Option<i64>,
    interface: Option<String>,
    ipv4: bool,
    ipv6: bool,
) -> PyResult<Bound<'py, PyAny>> {
    // 创建 AsyncPinger 实例
    let pinger = AsyncPinger::new(target, interval_ms, interface, ipv4, ipv6)?;

    // 执行异步 ping_multiple
    pinger.ping_multiple(py, count, timeout_ms)
}

pub fn get_ping_rs_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION.get_or_init(|| {
        let version = env!("CARGO_PKG_VERSION");
        // cargo uses "1.0-alpha1" etc. while python uses "1.0.0a1", this is not full compatibility,
        // but it's good enough for now
        // see https://docs.rs/semver/1.0.9/semver/struct.Version.html#method.parse for rust spec
        // see https://peps.python.org/pep-0440/ for python spec
        // it seems the dot after "alpha/beta" e.g. "-alpha.1" is not necessary, hence why this works
        version.replace("-alpha", "a").replace("-beta", "b")
    })
}

/// Python 模块定义
#[pymodule]
fn _ping_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 初始化日志
    pyo3_log::init();

    // 添加类
    m.add_class::<PingResult>()?;
    m.add_class::<Pinger>()?;
    m.add_class::<AsyncPinger>()?;
    m.add_class::<PingStream>()?;
    m.add_class::<AsyncPingStream>()?;

    // 添加函数
    m.add_function(wrap_pyfunction!(ping_once, m)?)?;
    m.add_function(wrap_pyfunction!(ping_once_async, m)?)?;
    m.add_function(wrap_pyfunction!(ping_multiple, m)?)?;
    m.add_function(wrap_pyfunction!(ping_multiple_async, m)?)?;
    m.add_function(wrap_pyfunction!(create_ping_stream, m)?)?;

    // 添加版本信息
    m.add("__version__", get_ping_rs_version())?;

    // 未来可以添加TCP相关类型
    // m.add_class::<protocols::tcp::ping::TcpPinger>()?;
    // m.add_class::<protocols::tcp::stream::TcpPingStream>()?;

    Ok(())
}
