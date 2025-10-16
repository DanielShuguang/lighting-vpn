/**
 * VPN 相关类型定义
 */

/**
 * VPN 配置接口
 */
export interface VpnConfig {
  id: string
  name: string
  protocol: VpnProtocol
  server: string
  port: number
  password?: string
  method?: string
  remarks?: string
  created_at: string
  updated_at: string
}

/**
 * VPN 协议类型
 */
export type VpnProtocol = 'Shadowsocks' | 'ShadowsocksR' | 'V2Ray' | 'Vmess' | 'Trojan'

/**
 * 连接状态类型
 */
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

/**
 * 延迟测试结果
 */
export interface LatencyTestResult {
  latency: number | null
  success: boolean
  error?: string
}

/**
 * 批量测试结果
 */
export interface BatchTestResult {
  config_id: string
  result: LatencyTestResult
}

/**
 * 连接信息
 */
export interface ConnectionInfo {
  config_id: string
  connected_at?: string
  status: ConnectionStatus
}

/**
 * 配置列表响应
 */
export interface ConfigsResponse {
  configs: VpnConfig[]
}

/**
 * 协议标签配置
 */
export interface ProtocolTag {
  type: 'success' | 'info' | 'warning' | 'error' | 'default'
  text: string
}

/**
 * 协议标签映射
 */
export type ProtocolTagMap = Record<VpnProtocol, ProtocolTag>

/**
 * 状态标签配置
 */
export interface StatusTag {
  type: 'success' | 'info' | 'warning' | 'error' | 'default'
  text: string
}

/**
 * 状态标签映射
 */
export type StatusTagMap = Record<ConnectionStatus, StatusTag>
