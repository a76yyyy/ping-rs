use std::time::Duration;

/// DNS 预解析配置选项
///
/// 用于控制 DNS 主机名解析的行为
#[derive(Clone, Copy, Debug)]
pub struct DnsPreResolveOptions {
    /// 是否启用 DNS 预解析（默认为 true）
    pub enable: bool,
    /// DNS 解析超时时间（默认为 None，表示使用 options.interval）
    pub timeout: Option<Duration>,
}

impl Default for DnsPreResolveOptions {
    fn default() -> Self {
        Self {
            enable: true,
            timeout: None,
        }
    }
}
