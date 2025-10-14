use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub latency: Option<u64>, // 延迟，单位：毫秒
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTestResult {
    pub config_id: String,
    pub config_name: String,
    pub result: TestResult,
}

// 测试单个服务器的连接性和延迟
pub async fn test_connection(server: &str, port: u16, timeout_secs: u64) -> TestResult {
    let address = format!("{}:{}", server, port);
    let start = Instant::now();

    match timeout(
        Duration::from_secs(timeout_secs),
        TcpStream::connect(&address),
    )
    .await
    {
        Ok(Ok(_stream)) => {
            let latency = start.elapsed().as_millis() as u64;
            TestResult {
                success: true,
                latency: Some(latency),
                error: None,
            }
        }
        Ok(Err(e)) => TestResult {
            success: false,
            latency: None,
            error: Some(format!("连接失败: {}", e)),
        },
        Err(_) => TestResult {
            success: false,
            latency: None,
            error: Some(format!("连接超时 ({}秒)", timeout_secs)),
        },
    }
}

// 测试延迟（通过多次连接取平均值）
pub async fn test_latency(server: &str, port: u16, count: u32, timeout_secs: u64) -> TestResult {
    let mut latencies = Vec::new();
    let mut errors = Vec::new();

    for _ in 0..count {
        let result = test_connection(server, port, timeout_secs).await;
        if result.success {
            if let Some(latency) = result.latency {
                latencies.push(latency);
            }
        } else {
            if let Some(error) = result.error {
                errors.push(error);
            }
        }
    }

    if latencies.is_empty() {
        TestResult {
            success: false,
            latency: None,
            error: Some(format!("所有测试都失败了: {:?}", errors)),
        }
    } else {
        let avg_latency = latencies.iter().sum::<u64>() / latencies.len() as u64;
        TestResult {
            success: true,
            latency: Some(avg_latency),
            error: None,
        }
    }
}

// 通过 HTTP 请求测试连接（可选用于更全面的测试）
pub async fn test_http_connection(
    url: &str,
    timeout_secs: u64,
    use_proxy: bool,
    proxy_url: Option<&str>,
) -> TestResult {
    let mut client_builder = reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs));

    if use_proxy {
        if let Some(proxy) = proxy_url {
            if let Ok(proxy) = reqwest::Proxy::all(proxy) {
                client_builder = client_builder.proxy(proxy);
            }
        }
    }

    let start = Instant::now();

    match client_builder.build() {
        Ok(client) => match client.get(url).send().await {
            Ok(response) => {
                let latency = start.elapsed().as_millis() as u64;
                if response.status().is_success() {
                    TestResult {
                        success: true,
                        latency: Some(latency),
                        error: None,
                    }
                } else {
                    TestResult {
                        success: false,
                        latency: Some(latency),
                        error: Some(format!("HTTP 状态码: {}", response.status())),
                    }
                }
            }
            Err(e) => TestResult {
                success: false,
                latency: None,
                error: Some(format!("请求失败: {}", e)),
            },
        },
        Err(e) => TestResult {
            success: false,
            latency: None,
            error: Some(format!("创建客户端失败: {}", e)),
        },
    }
}

// 批量测试多个配置
pub async fn batch_test_connections(
    configs: Vec<(String, String, String, u16)>, // (id, name, server, port)
    timeout_secs: u64,
) -> Vec<BatchTestResult> {
    let mut results = Vec::new();

    for (config_id, config_name, server, port) in configs {
        let result = test_connection(&server, port, timeout_secs).await;
        results.push(BatchTestResult {
            config_id,
            config_name,
            result,
        });
    }

    results
}

// 批量测试延迟
pub async fn batch_test_latencies(
    configs: Vec<(String, String, String, u16)>, // (id, name, server, port)
    count: u32,
    timeout_secs: u64,
) -> Vec<BatchTestResult> {
    let mut results = Vec::new();

    for (config_id, config_name, server, port) in configs {
        let result = test_latency(&server, port, count, timeout_secs).await;
        results.push(BatchTestResult {
            config_id,
            config_name,
            result,
        });
    }

    results
}

// 实时延迟测试（用于持续监控）
#[allow(dead_code)]
pub async fn continuous_latency_test(
    server: &str,
    port: u16,
    interval_secs: u64,
    duration_secs: u64,
) -> Vec<TestResult> {
    let mut results = Vec::new();
    let end_time = Instant::now() + Duration::from_secs(duration_secs);

    while Instant::now() < end_time {
        let result = test_connection(server, port, 5).await;
        results.push(result);

        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }

    results
}
