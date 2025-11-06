# 🚀 Deployment Guide

This guide covers deploying the Moby Market frontend to various environments and platforms.

## 📋 Overview

The Moby Market frontend is a Vue.js 3 application built with Vite, designed for modern deployment strategies including:

- **Static hosting** (Vercel, Netlify, GitHub Pages)
- **Container deployment** (Docker, Kubernetes)
- **CDN deployment** (CloudFront, CloudFlare)
- **Traditional servers** (Nginx, Apache)

## 🏗️ Build Process

### Production Build

```bash
# Build for production
npm run build

# Output will be in the 'dist' directory
# dist/
# ├── assets/
# │   ├── css/
# │   └── js/
# ├── images/
# ├── index.html
# └── manifest.json
```

### Build Optimization

The build process includes:

- **Code splitting**: Automatic vendor and route-based splitting
- **Tree shaking**: Dead code elimination
- **Minification**: CSS and JavaScript compression
- **Asset optimization**: Image compression and format conversion
- **Service worker**: For PWA functionality

### Environment-Specific Builds

```bash
# Development build
npm run build:dev

# Staging build
npm run build:staging

# Production build
npm run build

# Preview build locally
npm run preview
```

## 🌍 Environment Configuration

### Environment Files

Create environment-specific files:

```
.env                # Default environment variables
.env.local          # Local overrides (not committed)
.env.development    # Development environment
.env.staging        # Staging environment
.env.production     # Production environment
```

### Production Environment Variables

```env
# Production .env.production
VITE_APP_ENV=production
VITE_API_BASE_URL=https://api.mobymarket.com
VITE_WS_BASE_URL=wss://api.mobymarket.com
VITE_APP_NAME=Moby Market
VITE_APP_VERSION=1.0.0

# Feature Flags
VITE_ENABLE_DEMO_MODE=false
VITE_ENABLE_ANALYTICS=true
VITE_ENABLE_ERROR_REPORTING=true

# External Services
VITE_WALLET_CONNECT_PROJECT_ID=prod_project_id
VITE_COINGECKO_API_KEY=prod_api_key
VITE_ALCHEMY_API_KEY=prod_api_key

# Security
VITE_MOCK_API=false
VITE_DEBUG_MODE=false
```

## ☁️ Cloud Deployment

### Vercel Deployment

#### Automatic Deployment

1. **Connect Repository**
   ```bash
   # Install Vercel CLI
   npm i -g vercel

   # Login and deploy
   vercel login
   vercel
   ```

2. **Configuration** (`vercel.json`)
   ```json
   {
     "version": 2,
     "builds": [
       {
         "src": "package.json",
         "use": "@vercel/static-build",
         "config": {
           "distDir": "dist"
         }
       }
     ],
     "routes": [
       {
         "src": "/assets/(.*)",
         "headers": {
           "cache-control": "public, max-age=31536000, immutable"
         }
       },
       {
         "handle": "filesystem"
       },
       {
         "src": "/.*",
         "dest": "/index.html"
       }
     ],
     "env": {
       "VITE_API_BASE_URL": "@vite_api_base_url",
       "VITE_WS_BASE_URL": "@vite_ws_base_url"
     }
   }
   ```

3. **Environment Variables**
   Set in Vercel dashboard or CLI:
   ```bash
   vercel env add VITE_API_BASE_URL production
   vercel env add VITE_WS_BASE_URL production
   ```

### Netlify Deployment

#### Automatic Deployment

1. **Configuration** (`netlify.toml`)
   ```toml
   [build]
     publish = "dist"
     command = "npm run build"

   [build.environment]
     NODE_VERSION = "18"

   [[redirects]]
     from = "/*"
     to = "/index.html"
     status = 200

   [[headers]]
     for = "/assets/*"
     [headers.values]
       Cache-Control = "public, max-age=31536000, immutable"

   [[headers]]
     for = "/*.js"
     [headers.values]
       Cache-Control = "public, max-age=31536000, immutable"

   [[headers]]
     for = "/*.css"
     [headers.values]
       Cache-Control = "public, max-age=31536000, immutable"
   ```

2. **Deploy via CLI**
   ```bash
   # Install Netlify CLI
   npm install -g netlify-cli

   # Build and deploy
   npm run build
   netlify deploy --prod --dir=dist
   ```

### AWS S3 + CloudFront

#### S3 Static Hosting

1. **Create S3 Bucket**
   ```bash
   # AWS CLI deployment script
   aws s3 mb s3://moby-market-frontend
   aws s3 website s3://moby-market-frontend --index-document index.html --error-document index.html
   ```

2. **Upload Build**
   ```bash
   # Sync dist folder to S3
   aws s3 sync dist/ s3://moby-market-frontend --delete

   # Set cache headers
   aws s3 cp dist/ s3://moby-market-frontend --recursive --cache-control "public, max-age=31536000" --exclude "*.html"
   aws s3 cp dist/index.html s3://moby-market-frontend --cache-control "public, max-age=0, must-revalidate"
   ```

3. **CloudFront Distribution**
   ```json
   {
     "Origins": [{
       "DomainName": "moby-market-frontend.s3.amazonaws.com",
       "Id": "S3-moby-market-frontend",
       "S3OriginConfig": {
         "OriginAccessIdentity": ""
       }
     }],
     "DefaultCacheBehavior": {
       "TargetOriginId": "S3-moby-market-frontend",
       "ViewerProtocolPolicy": "redirect-to-https",
       "CachePolicyId": "managed-caching-optimized"
     },
     "CustomErrorResponses": [{
       "ErrorCode": 404,
       "ResponseCode": 200,
       "ResponsePagePath": "/index.html"
     }]
   }
   ```

## 🐳 Docker Deployment

### Dockerfile

```dockerfile
# Multi-stage build
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy built assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost/ || exit 1

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### Nginx Configuration

```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Gzip compression
    gzip on;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/javascript
        application/xml+rss
        application/json;

    server {
        listen 80;
        server_name _;
        root /usr/share/nginx/html;
        index index.html;

        # Cache static assets
        location /assets/ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }

        # SPA fallback
        location / {
            try_files $uri $uri/ /index.html;
            add_header Cache-Control "no-cache, no-store, must-revalidate";
        }

        # Security headers
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;
        add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    }
}
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  moby-market-frontend:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "80:80"
    environment:
      - NODE_ENV=production
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Build and Deploy

```bash
# Build Docker image
docker build -t moby-market-frontend .

# Run container
docker run -d -p 80:80 --name moby-market-frontend moby-market-frontend

# Docker Compose
docker-compose up -d
```

## ⚙️ Kubernetes Deployment

### Deployment Configuration

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: moby-market-frontend
  labels:
    app: moby-market-frontend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: moby-market-frontend
  template:
    metadata:
      labels:
        app: moby-market-frontend
    spec:
      containers:
      - name: frontend
        image: moby-market-frontend:latest
        ports:
        - containerPort: 80
        env:
        - name: NODE_ENV
          value: "production"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: moby-market-frontend-service
spec:
  selector:
    app: moby-market-frontend
  ports:
  - protocol: TCP
    port: 80
    targetPort: 80
  type: LoadBalancer

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: moby-market-frontend-ingress
  annotations:
    kubernetes.io/ingress.class: "nginx"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - mobymarket.com
    secretName: moby-market-tls
  rules:
  - host: mobymarket.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: moby-market-frontend-service
            port:
              number: 80
```

### Deploy to Kubernetes

```bash
# Apply deployment
kubectl apply -f k8s-deployment.yaml

# Check status
kubectl get pods -l app=moby-market-frontend
kubectl get services
kubectl get ingress

# Scale deployment
kubectl scale deployment moby-market-frontend --replicas=5
```

## 🔧 CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3

    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'

    - name: Install dependencies
      run: npm ci

    - name: Run tests
      run: npm run test

    - name: Run type check
      run: npm run type-check

    - name: Run linting
      run: npm run lint

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    steps:
    - uses: actions/checkout@v3

    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'

    - name: Install dependencies
      run: npm ci

    - name: Build application
      run: npm run build
      env:
        VITE_API_BASE_URL: ${{ secrets.VITE_API_BASE_URL }}
        VITE_WS_BASE_URL: ${{ secrets.VITE_WS_BASE_URL }}

    - name: Upload build artifacts
      uses: actions/upload-artifact@v3
      with:
        name: dist
        path: dist/

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    steps:
    - name: Download build artifacts
      uses: actions/download-artifact@v3
      with:
        name: dist
        path: dist/

    - name: Deploy to Vercel
      uses: amondnet/vercel-action@v20
      with:
        vercel-token: ${{ secrets.VERCEL_TOKEN }}
        vercel-org-id: ${{ secrets.VERCEL_ORG_ID }}
        vercel-project-id: ${{ secrets.VERCEL_PROJECT_ID }}
        vercel-args: '--prod'
```

## 📊 Monitoring & Analytics

### Error Tracking

```typescript
// src/services/monitoring.ts
import * as Sentry from '@sentry/vue'

if (import.meta.env.PROD) {
  Sentry.init({
    dsn: import.meta.env.VITE_SENTRY_DSN,
    environment: import.meta.env.VITE_APP_ENV,
    integrations: [
      new Sentry.BrowserTracing(),
    ],
    tracesSampleRate: 0.1,
  })
}
```

### Performance Monitoring

```typescript
// src/services/analytics.ts
import { getCLS, getFID, getFCP, getLCP, getTTFB } from 'web-vitals'

if (import.meta.env.PROD) {
  getCLS(sendToAnalytics)
  getFID(sendToAnalytics)
  getFCP(sendToAnalytics)
  getLCP(sendToAnalytics)
  getTTFB(sendToAnalytics)
}

function sendToAnalytics({ name, value, id }) {
  // Send to your analytics service
  gtag('event', name, {
    value: Math.round(value),
    event_label: id,
  })
}
```

## 🛡️ Security Considerations

### Content Security Policy

```html
<!-- In index.html -->
<meta http-equiv="Content-Security-Policy" content="
  default-src 'self';
  script-src 'self' 'unsafe-eval' https://vercel.live;
  style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
  img-src 'self' data: https: blob:;
  font-src 'self' https://fonts.gstatic.com;
  connect-src 'self' https://api.mobymarket.com wss://api.mobymarket.com;
  media-src 'self';
  object-src 'none';
  base-uri 'self';
  form-action 'self';
  frame-ancestors 'none';
  upgrade-insecure-requests;
">
```

### Environment Security

```bash
# Never commit sensitive environment variables
echo ".env.local" >> .gitignore
echo ".env.production" >> .gitignore

# Use secrets management for production
# - Vercel: Environment Variables dashboard
# - AWS: Parameter Store or Secrets Manager
# - Kubernetes: Secrets
```

## 🔍 Health Checks

### Application Health

```typescript
// src/services/health.ts
export async function checkHealth() {
  try {
    const response = await fetch('/api/health')
    return response.ok
  } catch {
    return false
  }
}

// Expose health endpoint for load balancers
router.get('/health', (req, res) => {
  res.status(200).json({ status: 'ok' })
})
```

### Monitoring Script

```bash
#!/bin/bash
# health-check.sh

URL="https://mobymarket.com"
EXPECTED_STATUS=200

STATUS=$(curl -s -o /dev/null -w "%{http_code}" $URL)

if [ $STATUS -eq $EXPECTED_STATUS ]; then
  echo "✅ Application is healthy (Status: $STATUS)"
  exit 0
else
  echo "❌ Application is unhealthy (Status: $STATUS)"
  exit 1
fi
```

## 📚 Deployment Checklist

### Pre-Deployment

- [ ] Run all tests (`npm run test`)
- [ ] Type check passes (`npm run type-check`)
- [ ] Linting passes (`npm run lint`)
- [ ] Build succeeds (`npm run build`)
- [ ] Environment variables configured
- [ ] Performance audit completed
- [ ] Security scan completed

### Post-Deployment

- [ ] Application loads successfully
- [ ] API connections working
- [ ] WebSocket connections established
- [ ] Error tracking configured
- [ ] Analytics tracking active
- [ ] Performance monitoring active
- [ ] Health checks passing
- [ ] SSL certificate valid
- [ ] DNS records updated

### Rollback Plan

1. **Immediate rollback**: Revert to previous deployment
2. **Database issues**: Restore from backup
3. **API issues**: Switch to maintenance mode
4. **Monitoring**: Check error rates and performance

---

This deployment guide ensures reliable, secure, and scalable deployment of the Moby Market frontend across various platforms and environments.