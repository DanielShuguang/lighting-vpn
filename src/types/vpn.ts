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
  group?: string // 分组标签
  subscription_id?: string
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

/**
 * 代理模式类型
 */
export type ProxyMode = 'global' | 'pac' | 'direct'

/**
 * 代理模式配置
 */
export interface ProxyModeConfig {
  mode: ProxyMode
  label: string
  description: string
}

/**
 * 代理模式列表
 */
export const PROXY_MODES: ProxyModeConfig[] = [
  {
    mode: 'global',
    label: '全局代理',
    description: '所有流量都通过代理'
  },
  {
    mode: 'pac',
    label: 'PAC模式',
    description: '根据规则智能分流（国内直连，国外走代理）'
  },
  {
    mode: 'direct',
    label: '直连模式',
    description: '仅代理需要代理的流量'
  }
]
