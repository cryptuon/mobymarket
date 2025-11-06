# =€ Moby Market Deployment Guide

Complete deployment and infrastructure guide for the Moby Market whale trading platform.

## =Ë Table of Contents

- [Prerequisites](#prerequisites)
- [Infrastructure Setup](#infrastructure-setup)
- [Environment Configuration](#environment-configuration)
- [Database Setup](#database-setup)
- [Service Deployment](#service-deployment)
- [API Configuration](#api-configuration)
- [Monitoring & Logging](#monitoring--logging)
- [Security Configuration](#security-configuration)
- [Scaling & Performance](#scaling--performance)
- [Troubleshooting](#troubleshooting)

## =' Prerequisites

### System Requirements

**Minimum Production Requirements:**
- **CPU**: 16+ cores
- **RAM**: 64GB+
- **Storage**: 2TB+ SSD (NVMe preferred)
- **Network**: 10Gbps+ with low latency to major exchanges
- **OS**: Ubuntu 22.04 LTS or similar

**Recommended Production Setup:**
- **CPU**: 32+ cores (AMD EPYC/Intel Xeon)
- **RAM**: 128GB+
- **Storage**: 5TB+ NVMe SSD in RAID 1
- **Network**: 25Gbps+ dedicated lines to major DeFi hubs
- **Redundancy**: Multi-region deployment

### Software Dependencies

```bash
# Core dependencies
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo apt-get update && sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql-client \
    redis-tools \
    nginx \
    certbot

# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js (for monitoring tools)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

## <× Infrastructure Setup

### Docker Compose Architecture

Create the main deployment configuration:

```yaml
# docker-compose.yml
version: '3.8'

services:
  # Core Platform Services
  moby-platform:
    build:
      context: ./platform
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
      - "8443:8443"
    environment:
      - ENVIRONMENT=production
      - DATABASE_URL=postgresql://moby:${DB_PASSWORD}@postgres:5432/moby_market
      - REDIS_URL=redis://redis:6379
      - ORACLE_ENDPOINTS=${ORACLE_ENDPOINTS}
      - BRIDGE_PRIVATE_KEY=${BRIDGE_PRIVATE_KEY}
    depends_on:
      - postgres
      - redis
      - prometheus
    volumes:
      - ./config:/app/config
      - ./logs:/app/logs
    restart: unless-stopped
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
        reservations:
          cpus: '2.0'
          memory: 4G

  # Privacy Service
  moby-privacy:
    build:
      context: ./libs/moby-privacy
      dockerfile: Dockerfile
    ports:
      - "8081:8081"
    environment:
      - ZK_CIRCUIT_PATH=/app/circuits
      - TRUSTED_SETUP_PATH=/app/setup
    volumes:
      - ./privacy/circuits:/app/circuits
      - ./privacy/setup:/app/setup
    restart: unless-stopped

  # Governance Service
  moby-governance:
    build:
      context: ./libs/moby-governance
      dockerfile: Dockerfile
    ports:
      - "8082:8082"
    environment:
      - GOVERNANCE_CONTRACT=${GOVERNANCE_CONTRACT}
      - VOTING_TOKEN_CONTRACT=${VOTING_TOKEN_CONTRACT}
    restart: unless-stopped

  # Bridge Service
  moby-bridge:
    build:
      context: ./libs/moby-bridge
      dockerfile: Dockerfile
    ports:
      - "8083:8083"
    environment:
      - ETHEREUM_RPC=${ETHEREUM_RPC}
      - POLYGON_RPC=${POLYGON_RPC}
      - AVALANCHE_RPC=${AVALANCHE_RPC}
      - ARBITRUM_RPC=${ARBITRUM_RPC}
      - BRIDGE_PRIVATE_KEY=${BRIDGE_PRIVATE_KEY}
    restart: unless-stopped

  # Oracle Service
  moby-oracle:
    build:
      context: ./libs/moby-oracle
      dockerfile: Dockerfile
    ports:
      - "8084:8084"
    environment:
      - CHAINLINK_ENDPOINT=${CHAINLINK_ENDPOINT}
      - PYTH_ENDPOINT=${PYTH_ENDPOINT}
      - BAND_ENDPOINT=${BAND_ENDPOINT}
      - PRICE_UPDATE_INTERVAL=5
    restart: unless-stopped

  # DEX Service
  moby-dex:
    build:
      context: ./libs/moby-dex
      dockerfile: Dockerfile
    ports:
      - "8085:8085"
    environment:
      - UNISWAP_V3_ROUTER=${UNISWAP_V3_ROUTER}
      - CURVE_REGISTRY=${CURVE_REGISTRY}
      - SUSHISWAP_ROUTER=${SUSHISWAP_ROUTER}
    restart: unless-stopped

  # Database Services
  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=moby
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=moby_market
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    command: redis-server --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    restart: unless-stopped

  # Monitoring Stack
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
    restart: unless-stopped

  # Load Balancer
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/ssl:/etc/nginx/ssl
      - ./nginx/logs:/var/log/nginx
    depends_on:
      - moby-platform
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:

networks:
  default:
    driver: bridge
    ipam:
      driver: default
      config:
        - subnet: 172.20.0.0/16
```

### Production Kubernetes Deployment

For high-scale production deployments:

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: moby-market
---
# k8s/moby-platform-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: moby-platform
  namespace: moby-market
spec:
  replicas: 5
  selector:
    matchLabels:
      app: moby-platform
  template:
    metadata:
      labels:
        app: moby-platform
    spec:
      containers:
      - name: moby-platform
        image: moby-market/platform:latest
        ports:
        - containerPort: 8080
        - containerPort: 8443
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: moby-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: moby-secrets
              key: redis-url
        resources:
          requests:
            cpu: 2000m
            memory: 4Gi
          limits:
            cpu: 4000m
            memory: 8Gi
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/status
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: moby-platform-service
  namespace: moby-market
spec:
  selector:
    app: moby-platform
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: https
    port: 443
    targetPort: 8443
  type: LoadBalancer
```

## ™ Environment Configuration

### Production Environment Variables

Create `.env.production`:

```bash
# Database Configuration
DB_PASSWORD=super_secure_password_here
DATABASE_URL=postgresql://moby:${DB_PASSWORD}@postgres:5432/moby_market
DATABASE_POOL_SIZE=20
DATABASE_TIMEOUT=30

# Redis Configuration
REDIS_PASSWORD=redis_secure_password
REDIS_URL=redis://:${REDIS_PASSWORD}@redis:6379
REDIS_POOL_SIZE=10

# Blockchain Configuration
ETHEREUM_RPC=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
ETHEREUM_WS=wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID
POLYGON_RPC=https://polygon-mainnet.infura.io/v3/YOUR_PROJECT_ID
AVALANCHE_RPC=https://api.avax.network/ext/bc/C/rpc
ARBITRUM_RPC=https://arb1.arbitrum.io/rpc
OPTIMISM_RPC=https://mainnet.optimism.io

# Private Keys (Use secure key management in production)
BRIDGE_PRIVATE_KEY=0x...  # Secure key for bridge operations
GOVERNANCE_PRIVATE_KEY=0x...  # Governance operations key
DEX_PRIVATE_KEY=0x...  # DEX trading key

# Oracle Configuration
CHAINLINK_ENDPOINT=https://api.chain.link
PYTH_ENDPOINT=https://pyth.network/api
BAND_ENDPOINT=https://laozi1.bandchain.org
ORACLE_UPDATE_INTERVAL=5
ORACLE_TIMEOUT=10

# Smart Contract Addresses
GOVERNANCE_CONTRACT=0x...
VOTING_TOKEN_CONTRACT=0x...
UNISWAP_V3_ROUTER=0xE592427A0AEce92De3Edee1F18E0157C05861564
CURVE_REGISTRY=0x90E00ACe148ca3b23Ac1bC8C240C2a7Dd9c2d7f5
SUSHISWAP_ROUTER=0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F

# Security Configuration
JWT_SECRET=ultra_secure_jwt_secret_min_32_chars
API_RATE_LIMIT=1000
MAX_TRADE_SIZE=1000000000  # $1B max trade size
MIN_TRADE_SIZE=1000        # $1K min trade size

# Monitoring
GRAFANA_PASSWORD=grafana_admin_password
PROMETHEUS_RETENTION=90d
LOG_LEVEL=info
METRICS_ENABLED=true

# Feature Flags
PRIVACY_ENABLED=true
GOVERNANCE_ENABLED=true
BRIDGE_ENABLED=true
ORACLE_ENABLED=true
DEX_ENABLED=true
YIELD_OPTIMIZATION=true
MEV_PROTECTION=true
CROSS_CHAIN_TRADING=true

# Performance Tuning
WORKER_THREADS=16
CONNECTION_POOL_SIZE=50
CACHE_TTL=300
BATCH_SIZE=100
WEBSOCKET_MAX_CONNECTIONS=10000

# Business Configuration
PLATFORM_FEE=0.002          # 0.2%
WHALE_THRESHOLD=1000000     # $1M
MAX_SLIPPAGE=0.05           # 5%
DEFAULT_PRIVACY_LEVEL=medium
```

### SSL/TLS Configuration

```bash
# Generate SSL certificates (production should use proper CA)
mkdir -p nginx/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
    -keyout nginx/ssl/moby-market.key \
    -out nginx/ssl/moby-market.crt \
    -subj "/C=US/ST=CA/L=SF/O=MobyMarket/CN=api.moby-market.com"

# For production, use Let's Encrypt
certbot certonly --nginx -d api.moby-market.com -d ws.moby-market.com
```

## =Ä Database Setup

### PostgreSQL Schema

```sql
-- sql/init.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users and Authentication
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) UNIQUE NOT NULL,
    tier VARCHAR(20) DEFAULT 'basic',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true
);

-- API Keys
CREATE TABLE api_keys (
    key_id VARCHAR(64) PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    permissions TEXT[] DEFAULT ARRAY[]::TEXT[],
    rate_limit_per_minute INTEGER DEFAULT 100,
    rate_limit_per_hour INTEGER DEFAULT 1000,
    rate_limit_per_day INTEGER DEFAULT 10000,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true
);

-- Trading Records
CREATE TABLE trades (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    trade_type VARCHAR(20) NOT NULL,
    token_in VARCHAR(10) NOT NULL,
    token_out VARCHAR(10) NOT NULL,
    amount_in DECIMAL(38, 18) NOT NULL,
    amount_out DECIMAL(38, 18) NOT NULL,
    slippage DECIMAL(10, 6) NOT NULL,
    gas_used DECIMAL(38, 18) NOT NULL,
    profit_loss DECIMAL(38, 18),
    strategy_used VARCHAR(50),
    execution_time_ms INTEGER,
    transaction_hash VARCHAR(66),
    block_number BIGINT,
    chain VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Revenue Tracking
CREATE TABLE revenue_streams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    stream_type VARCHAR(30) NOT NULL,
    amount DECIMAL(38, 18) NOT NULL,
    user_id UUID REFERENCES users(id),
    transaction_hash VARCHAR(66),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Portfolio Positions
CREATE TABLE positions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    asset VARCHAR(10) NOT NULL,
    amount DECIMAL(38, 18) NOT NULL,
    average_price DECIMAL(38, 18) NOT NULL,
    chain VARCHAR(20) NOT NULL,
    position_type VARCHAR(20) DEFAULT 'spot',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Yield Positions
CREATE TABLE yield_positions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    protocol VARCHAR(50) NOT NULL,
    strategy VARCHAR(30) NOT NULL,
    amount DECIMAL(38, 18) NOT NULL,
    estimated_apy DECIMAL(10, 6) NOT NULL,
    start_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    auto_compound BOOLEAN DEFAULT true,
    chain VARCHAR(20) NOT NULL
);

-- Market Analytics Cache
CREATE TABLE market_data_cache (
    id SERIAL PRIMARY KEY,
    data_type VARCHAR(50) NOT NULL,
    data JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Whale Tracking
CREATE TABLE whale_activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    whale_address_hash VARCHAR(64) NOT NULL, -- Anonymized
    trade_size DECIMAL(38, 18) NOT NULL,
    token_pair VARCHAR(20) NOT NULL,
    chain VARCHAR(20) NOT NULL,
    dex VARCHAR(50) NOT NULL,
    price_impact DECIMAL(10, 6) NOT NULL,
    strategy_detected VARCHAR(50),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Performance Metrics
CREATE TABLE performance_metrics (
    id SERIAL PRIMARY KEY,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB
);

-- System Health
CREATE TABLE system_health_logs (
    id SERIAL PRIMARY KEY,
    component VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    cpu_usage DECIMAL(5, 2),
    memory_usage DECIMAL(5, 2),
    error_count INTEGER DEFAULT 0,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_trades_user_id ON trades(user_id);
CREATE INDEX idx_trades_created_at ON trades(created_at);
CREATE INDEX idx_trades_chain ON trades(chain);
CREATE INDEX idx_revenue_streams_type ON revenue_streams(stream_type);
CREATE INDEX idx_revenue_streams_created_at ON revenue_streams(created_at);
CREATE INDEX idx_positions_user_id ON positions(user_id);
CREATE INDEX idx_whale_activities_timestamp ON whale_activities(timestamp);
CREATE INDEX idx_performance_metrics_timestamp ON performance_metrics(timestamp);
CREATE INDEX idx_market_data_cache_type_expires ON market_data_cache(data_type, expires_at);
```

### Database Optimization

```sql
-- Database tuning for high-frequency trading
-- postgresql.conf optimizations

-- Memory Settings
shared_buffers = 25% of RAM
effective_cache_size = 75% of RAM
work_mem = 256MB
maintenance_work_mem = 2GB

-- Connection Settings
max_connections = 200
shared_preload_libraries = 'pg_stat_statements'

-- Write Performance
wal_buffers = 16MB
checkpoint_completion_target = 0.9
checkpoint_timeout = 10min
max_wal_size = 4GB

-- Query Performance
random_page_cost = 1.1
effective_io_concurrency = 200
```

## =¢ Service Deployment

### Build Scripts

```bash
#!/bin/bash
# scripts/build.sh

set -e

echo "<× Building Moby Market Platform..."

# Build all libraries first
cd libs/moby-privacy && cargo build --release
cd ../moby-governance && cargo build --release
cd ../moby-bridge && cargo build --release
cd ../moby-oracle && cargo build --release
cd ../moby-dex && cargo build --release

# Build main platform
cd ../../platform && cargo build --release

# Build Docker images
docker build -t moby-market/privacy:latest libs/moby-privacy/
docker build -t moby-market/governance:latest libs/moby-governance/
docker build -t moby-market/bridge:latest libs/moby-bridge/
docker build -t moby-market/oracle:latest libs/moby-oracle/
docker build -t moby-market/dex:latest libs/moby-dex/
docker build -t moby-market/platform:latest platform/

echo " Build completed successfully!"
```

### Deployment Script

```bash
#!/bin/bash
# scripts/deploy.sh

set -e

ENVIRONMENT=${1:-production}
echo "=€ Deploying Moby Market to $ENVIRONMENT..."

# Load environment variables
source .env.$ENVIRONMENT

# Pre-deployment checks
echo "= Running pre-deployment checks..."
./scripts/health-check.sh

# Database migrations
echo "=Ê Running database migrations..."
cd platform && sqlx migrate run

# Deploy services
echo "=3 Starting services..."
docker-compose -f docker-compose.yml -f docker-compose.$ENVIRONMENT.yml up -d

# Wait for services to be healthy
echo "ó Waiting for services to be ready..."
./scripts/wait-for-services.sh

# Run post-deployment tests
echo ">ê Running post-deployment tests..."
./scripts/integration-tests.sh

# Setup monitoring
echo "=Ê Configuring monitoring..."
./scripts/setup-monitoring.sh

echo " Deployment completed successfully!"
echo "< Platform available at: https://api.moby-market.com"
echo "=Ê Monitoring available at: https://monitoring.moby-market.com"
```

### Health Check Script

```bash
#!/bin/bash
# scripts/health-check.sh

echo "<å Running health checks..."

# Check system resources
available_memory=$(free -m | awk 'NR==2{printf "%.1f%%", $3*100/$2 }')
available_disk=$(df -h | awk '$NF=="/"{printf "%s", $5}')
cpu_usage=$(top -bn1 | grep load | awk '{printf "%.2f", $(NF-2)}')

echo "=¾ Memory usage: $available_memory"
echo "=½ Disk usage: $available_disk"
echo "¡ CPU load: $cpu_usage"

# Check network connectivity
echo "< Checking network connectivity..."
curl -f https://ethereum.org > /dev/null || (echo "L Ethereum network unreachable" && exit 1)
curl -f https://polygon.technology > /dev/null || (echo "L Polygon network unreachable" && exit 1)

# Check external service dependencies
echo "= Checking external services..."
curl -f https://api.coingecko.com/api/v3/ping > /dev/null || (echo "L CoinGecko API unreachable" && exit 1)

echo " All health checks passed!"
```

## =' API Configuration

### Nginx Configuration

```nginx
# nginx/nginx.conf
events {
    worker_connections 4096;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Performance optimizations
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 100M;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s;
    limit_req_zone $binary_remote_addr zone=ws:10m rate=50r/s;

    # Upstream backend servers
    upstream moby_platform {
        least_conn;
        server moby-platform:8080 max_fails=3 fail_timeout=30s;
        keepalive 32;
    }

    # HTTP to HTTPS redirect
    server {
        listen 80;
        server_name api.moby-market.com ws.moby-market.com;
        return 301 https://$server_name$request_uri;
    }

    # Main API server
    server {
        listen 443 ssl http2;
        server_name api.moby-market.com;

        # SSL configuration
        ssl_certificate /etc/nginx/ssl/moby-market.crt;
        ssl_certificate_key /etc/nginx/ssl/moby-market.key;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;

        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

        # API endpoints
        location /api/ {
            limit_req zone=api burst=200 nodelay;

            proxy_pass http://moby_platform;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # Timeouts
            proxy_connect_timeout 5s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;

            # Response buffering
            proxy_buffering on;
            proxy_buffer_size 8k;
            proxy_buffers 16 8k;
        }

        # Health check endpoint (no rate limiting)
        location /api/v1/health {
            proxy_pass http://moby_platform;
            access_log off;
        }
    }

    # WebSocket server
    server {
        listen 443 ssl http2;
        server_name ws.moby-market.com;

        ssl_certificate /etc/nginx/ssl/moby-market.crt;
        ssl_certificate_key /etc/nginx/ssl/moby-market.key;

        location /ws {
            limit_req zone=ws burst=100 nodelay;

            proxy_pass http://moby_platform;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # WebSocket timeouts
            proxy_read_timeout 86400;
            proxy_send_timeout 86400;
        }
    }
}
```

## =Ê Monitoring & Logging

### Prometheus Configuration

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

scrape_configs:
  - job_name: 'moby-platform'
    static_configs:
      - targets: ['moby-platform:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'moby-privacy'
    static_configs:
      - targets: ['moby-privacy:8081']

  - job_name: 'moby-governance'
    static_configs:
      - targets: ['moby-governance:8082']

  - job_name: 'moby-bridge'
    static_configs:
      - targets: ['moby-bridge:8083']

  - job_name: 'moby-oracle'
    static_configs:
      - targets: ['moby-oracle:8084']

  - job_name: 'moby-dex'
    static_configs:
      - targets: ['moby-dex:8085']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Grafana Dashboards

```json
{
  "dashboard": {
    "title": "Moby Market - Trading Performance",
    "panels": [
      {
        "title": "Trading Volume (24h)",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(moby_trading_volume_total{period=\"24h\"})",
            "legendFormat": "Volume USD"
          }
        ]
      },
      {
        "title": "Active Trades per Second",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(moby_trades_total[5m])",
            "legendFormat": "TPS"
          }
        ]
      },
      {
        "title": "MEV Protection Effectiveness",
        "type": "gauge",
        "targets": [
          {
            "expr": "moby_mev_protection_effectiveness",
            "legendFormat": "Protection %"
          }
        ]
      },
      {
        "title": "Cross-Chain Success Rate",
        "type": "gauge",
        "targets": [
          {
            "expr": "moby_bridge_success_rate",
            "legendFormat": "Success Rate"
          }
        ]
      }
    ]
  }
}
```

### Alert Rules

```yaml
# monitoring/rules/alerts.yml
groups:
  - name: moby.alerts
    rules:
      - alert: HighErrorRate
        expr: rate(moby_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(moby_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "95th percentile latency is {{ $value }}s"

      - alert: DatabaseConnectionIssue
        expr: moby_database_connections_active / moby_database_connections_max > 0.8
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Database connection pool near capacity"

      - alert: LowLiquidity
        expr: moby_total_liquidity_usd < 100000000  # $100M
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Total liquidity below threshold"

      - alert: WhaleActivitySpike
        expr: increase(moby_whale_trades_total[1h]) > 100
        for: 0m
        labels:
          severity: info
        annotations:
          summary: "Unusual whale activity detected"

      - alert: BridgeFailure
        expr: rate(moby_bridge_failures_total[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Cross-chain bridge experiencing failures"

      - alert: OracleStale
        expr: time() - moby_oracle_last_update_timestamp > 300  # 5 minutes
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Oracle data is stale"

      - alert: MemoryUsageHigh
        expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"

      - alert: DiskSpaceLow
        expr: (1 - (node_filesystem_avail_bytes / node_filesystem_size_bytes)) > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Low disk space"
```

## = Security Configuration

### Security Hardening

```bash
#!/bin/bash
# scripts/security-hardening.sh

echo "= Applying security hardening..."

# Firewall configuration
ufw --force reset
ufw default deny incoming
ufw default allow outgoing

# Allow SSH (change port as needed)
ufw allow 22/tcp

# Allow HTTP/HTTPS
ufw allow 80/tcp
ufw allow 443/tcp

# Allow monitoring
ufw allow from 10.0.0.0/8 to any port 9090  # Prometheus
ufw allow from 10.0.0.0/8 to any port 3000  # Grafana

# Database access (internal only)
ufw allow from 172.20.0.0/16 to any port 5432
ufw allow from 172.20.0.0/16 to any port 6379

# Enable firewall
ufw --force enable

# Secure shared memory
echo "tmpfs /run/shm tmpfs defaults,noexec,nosuid 0 0" >> /etc/fstab

# Kernel security parameters
cat >> /etc/sysctl.conf << EOF
# IP Spoofing protection
net.ipv4.conf.default.rp_filter = 1
net.ipv4.conf.all.rp_filter = 1

# Ignore ICMP ping requests
net.ipv4.icmp_echo_ignore_all = 1

# Ignore send redirects
net.ipv4.conf.all.send_redirects = 0

# Disable source packet routing
net.ipv4.conf.all.accept_source_route = 0

# Log Martians
net.ipv4.conf.all.log_martians = 1

# Ignore ICMP redirects
net.ipv4.conf.all.accept_redirects = 0

# Disable IPv6
net.ipv6.conf.all.disable_ipv6 = 1
EOF

sysctl -p

echo " Security hardening completed!"
```

### API Key Management

```rust
// Example secure API key generation
use ring::{digest, pbkdf2, rand};
use ring::rand::SecureRandom;

pub fn generate_secure_api_key() -> String {
    let rng = rand::SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key).unwrap();

    hex::encode(key)
}

pub fn hash_api_key(key: &str, salt: &[u8]) -> Vec<u8> {
    let mut hash = vec![0u8; digest::SHA256_OUTPUT_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(100_000).unwrap(),
        salt,
        key.as_bytes(),
        &mut hash,
    );
    hash
}
```

## ¡ Scaling & Performance

### Auto-scaling Configuration

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: moby-platform-hpa
  namespace: moby-market
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: moby-platform
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: moby_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
```

### Performance Optimization

```toml
# platform/config/production.toml
[server]
workers = 16
max_connections = 1000
keep_alive = 30
timeout = 60

[database]
pool_size = 50
max_connections = 200
connection_timeout = 30
idle_timeout = 600

[cache]
redis_pool_size = 20
default_ttl = 300
max_memory_policy = "allkeys-lru"

[trading]
max_concurrent_trades = 100
order_timeout = 30
slippage_tolerance = 0.05
gas_price_multiplier = 1.1

[monitoring]
metrics_interval = 5
health_check_interval = 10
log_level = "info"
```

## =' Troubleshooting

### Common Issues and Solutions

#### Database Connection Issues

```bash
# Check database connectivity
docker exec -it moby-market_postgres_1 psql -U moby -d moby_market -c "SELECT version();"

# Check connection pool status
SELECT state, count(*) FROM pg_stat_activity GROUP BY state;

# Fix: Increase connection limits
# In postgresql.conf:
max_connections = 200
shared_buffers = 256MB
```

#### High Memory Usage

```bash
# Check memory usage by service
docker stats --no-stream

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full ./target/release/moby-platform

# Fix: Tune garbage collection
export RUST_BACKTRACE=1
export RUST_LOG=debug
```

#### Network Connectivity Issues

```bash
# Test blockchain connectivity
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  $ETHEREUM_RPC

# Check DNS resolution
nslookup api.moby-market.com

# Test SSL certificates
openssl s_client -connect api.moby-market.com:443 -servername api.moby-market.com
```

#### Performance Tuning

```bash
# Monitor system performance
htop
iotop
nethogs

# Check application metrics
curl -s http://localhost:8080/metrics | grep moby_

# Database query optimization
EXPLAIN ANALYZE SELECT * FROM trades WHERE user_id = 'uuid' AND created_at > NOW() - INTERVAL '1 day';
```

### Log Analysis

```bash
# Centralized logging with ELK stack
docker run -d \
  --name elasticsearch \
  -p 9200:9200 \
  -e "discovery.type=single-node" \
  docker.elastic.co/elasticsearch/elasticsearch:8.8.0

# Application logs
tail -f logs/moby-platform.log | grep ERROR

# Database logs
tail -f /var/log/postgresql/postgresql-15-main.log

# Nginx access logs
tail -f nginx/logs/access.log | awk '{print $1, $7, $9, $10}'
```

### Disaster Recovery

```bash
#!/bin/bash
# scripts/backup.sh

# Database backup
pg_dump -U moby -h localhost moby_market > backups/moby_market_$(date +%Y%m%d_%H%M%S).sql

# Configuration backup
tar -czf backups/config_$(date +%Y%m%d_%H%M%S).tar.gz config/ nginx/ monitoring/

# Docker volume backup
docker run --rm -v moby-market_postgres_data:/data -v $(pwd)/backups:/backup alpine \
  tar -czf /backup/postgres_data_$(date +%Y%m%d_%H%M%S).tar.gz -C /data .

echo " Backup completed!"
```

---

## <¯ Production Checklist

Before going live, ensure:

- [ ] All security hardening steps completed
- [ ] SSL certificates installed and configured
- [ ] Database properly tuned and backed up
- [ ] Monitoring and alerting configured
- [ ] Load testing completed
- [ ] Disaster recovery plan tested
- [ ] API rate limiting configured
- [ ] Private keys securely managed
- [ ] Network security rules applied
- [ ] Performance benchmarks established

## =Þ Support

For deployment support:
- **Documentation**: Check README.md and ARCHITECTURE.md
- **Issues**: GitHub Issues
- **Security**: security@moby-market.com
- **Emergency**: ops@moby-market.com

---

*Moby Market Platform - Enterprise-grade whale trading infrastructure*