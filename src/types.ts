export type DnsTestResult = {
  server_address: string;
  resolution_time_ms?: number | null;
  query_successful: boolean;
  latency_avg_ms?: number | null;
  jitter_avg_ms?: number | null;
  success_percent: number;
  dnssec_validated: boolean;
  ipv4_ips: string[];
  ipv6_ips: string[];
  error_msg?: string | null;
  avg_time?: number | null;
};

export type DownloadTestResult = {
  server_address: string;
  resolved_ip?: string | null;
  duration_ms: number;
  bytes_read: number;
  bandwidth_mbps: number;
  query_successful: boolean;
  http_status?: number | null;
  error_msg?: string | null;
};

export type DnsBenchmarkParams = {
  domainOrIp: string;
  samples?: number;
  timeoutSecs?: number;
  customServers?: string[];
};

export type DownloadSpeedParams = {
  url: string;
  durationSecs?: number;
  timeoutSecs?: number;
  customServers?: string[];
};
