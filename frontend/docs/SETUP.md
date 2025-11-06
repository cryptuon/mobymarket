# 🚀 Development Setup Guide

This guide will help you set up the Moby Market frontend development environment on your local machine.

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

### Required Software

- **Node.js** (v18.0.0 or higher)
  ```bash
  # Check your version
  node --version
  npm --version
  ```

- **Git** (latest version)
  ```bash
  # Check your version
  git --version
  ```

### Recommended Tools

- **VS Code** with recommended extensions (see [Editor Setup](#-editor-setup))
- **Chrome/Firefox** with developer tools
- **Postman** or similar API testing tool

## 🛠️ Installation

### 1. Clone the Repository

```bash
# Clone the repository
git clone https://github.com/your-org/moby-market.git

# Navigate to the frontend directory
cd moby-market/frontend
```

### 2. Install Dependencies

```bash
# Install npm dependencies
npm install

# Or using yarn (if preferred)
yarn install
```

### 3. Environment Configuration

Create environment files for different environments:

```bash
# Copy the example environment file
cp .env.example .env.local
```

#### Environment Variables

Edit `.env.local` with your configuration:

```env
# API Configuration
VITE_API_BASE_URL=http://localhost:3000/api
VITE_WS_BASE_URL=ws://localhost:3000

# App Configuration
VITE_APP_NAME=Moby Market
VITE_APP_VERSION=1.0.0
VITE_APP_ENV=development

# Feature Flags
VITE_ENABLE_DEMO_MODE=true
VITE_ENABLE_ANALYTICS=false
VITE_ENABLE_ERROR_REPORTING=false

# External Services
VITE_WALLET_CONNECT_PROJECT_ID=your_project_id
VITE_COINGECKO_API_KEY=your_api_key
VITE_ALCHEMY_API_KEY=your_api_key

# Authentication
VITE_JWT_EXPIRES_IN=24h
VITE_REFRESH_TOKEN_EXPIRES_IN=7d

# Development
VITE_MOCK_API=true
VITE_DEBUG_MODE=true
```

### 4. Start Development Server

```bash
# Start the development server
npm run dev

# The application will be available at:
# http://localhost:5173
```

## 🎯 Editor Setup

### VS Code Configuration

Install the recommended VS Code extensions:

```json
{
  "recommendations": [
    "vue.volar",
    "vue.typescript-vue-plugin",
    "bradlc.vscode-tailwindcss",
    "ms-vscode.vscode-typescript-next",
    "esbenp.prettier-vscode",
    "dbaeumer.vscode-eslint",
    "formulahendry.auto-rename-tag",
    "christian-kohler.path-intellisense"
  ]
}
```

#### Workspace Settings

Create `.vscode/settings.json`:

```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "typescript.preferences.importModuleSpecifier": "relative",
  "vue.inlayHints.missingProps": true,
  "vue.inlayHints.inlineHandlerLeading": true,
  "tailwindCSS.includeLanguages": {
    "vue": "html",
    "vue-html": "html"
  },
  "files.associations": {
    "*.vue": "vue"
  }
}
```

### Code Snippets

Create `.vscode/vue.code-snippets`:

```json
{
  "Vue Component": {
    "prefix": "vue-component",
    "body": [
      "<template>",
      "  <div>",
      "    $0",
      "  </div>",
      "</template>",
      "",
      "<script setup lang=\"ts\">",
      "",
      "</script>",
      "",
      "<style scoped>",
      "",
      "</style>"
    ]
  },
  "Composable": {
    "prefix": "composable",
    "body": [
      "import { ref, computed } from 'vue'",
      "",
      "export function use$1() {",
      "  const $2 = ref($3)",
      "",
      "  return {",
      "    $2",
      "  }",
      "}"
    ]
  }
}
```

## 🔧 Available Scripts

```bash
# Development
npm run dev              # Start development server
npm run dev:host         # Start dev server accessible from network

# Building
npm run build            # Build for production
npm run build:staging    # Build for staging environment
npm run preview          # Preview production build

# Code Quality
npm run lint             # Run ESLint
npm run lint:fix         # Fix ESLint errors
npm run type-check       # Run TypeScript type checking
npm run format           # Format code with Prettier

# Testing
npm run test             # Run unit tests
npm run test:watch       # Run tests in watch mode
npm run test:coverage    # Run tests with coverage
npm run test:e2e         # Run end-to-end tests

# Analysis
npm run analyze          # Analyze bundle size
npm run deps:check       # Check for outdated dependencies
npm run deps:update      # Update dependencies
```

## 🌍 Environment Configuration

### Development Environment

For local development, the application expects:

- **Backend API**: Running on `http://localhost:3000`
- **WebSocket**: Available at `ws://localhost:3000`
- **Mock Data**: Enabled by default

### API Integration

The app can work in two modes:

#### 1. Mock Mode (Default)
```env
VITE_MOCK_API=true
```
- Uses mock data for development
- No backend required
- All features functional with sample data

#### 2. Live API Mode
```env
VITE_MOCK_API=false
VITE_API_BASE_URL=http://localhost:3000/api
```
- Requires running backend
- Real API integration
- Full functionality

## 🔌 Backend Setup (Optional)

If you want to run with a real backend:

### 1. Start the Backend Server

```bash
# In a separate terminal, navigate to backend directory
cd ../backend

# Install dependencies and start
npm install
npm run dev
```

### 2. Update Environment

```env
VITE_MOCK_API=false
VITE_API_BASE_URL=http://localhost:3000/api
VITE_WS_BASE_URL=ws://localhost:3000
```

## 📱 Mobile Development

### Browser DevTools

1. Open Chrome DevTools
2. Click the device toggle button
3. Select a mobile device
4. Test responsive layouts

### Local Network Testing

```bash
# Start dev server accessible from network
npm run dev:host

# Access from mobile device using your IP
# http://192.168.x.x:5173
```

## 🧪 Testing Setup

### Unit Testing

Tests use Vitest and Vue Test Utils:

```bash
# Run tests
npm run test

# Watch mode
npm run test:watch
```

### E2E Testing

E2E tests use Playwright:

```bash
# Install Playwright browsers
npx playwright install

# Run E2E tests
npm run test:e2e
```

## 🔍 Debugging

### Vue DevTools

1. Install Vue DevTools browser extension
2. Open browser DevTools
3. Navigate to Vue tab
4. Inspect components, stores, and events

### Debug Configuration

VS Code debug configuration (`.vscode/launch.json`):

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "chrome",
      "request": "launch",
      "name": "Debug Vue App",
      "url": "http://localhost:5173",
      "webRoot": "${workspaceFolder}/src",
      "sourceMapPathOverrides": {
        "webpack:///src/*": "${webRoot}/*"
      }
    }
  ]
}
```

## 📦 Package Management

### Adding Dependencies

```bash
# Production dependency
npm install package-name

# Development dependency
npm install -D package-name

# Update package.json and install
npm install
```

### Dependency Management

```bash
# Check outdated packages
npm run deps:check

# Update dependencies
npm run deps:update

# Security audit
npm audit
npm audit fix
```

## 🌐 Browser Support

### Supported Browsers

- **Chrome**: Latest 2 versions
- **Firefox**: Latest 2 versions
- **Safari**: Latest 2 versions
- **Edge**: Latest 2 versions

### Polyfills

Modern features are handled by Vite's built-in polyfills.

## 🚨 Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Error: Port 5173 is already in use
# Solution: Kill the process or use a different port
npm run dev -- --port 5174
```

#### Module Not Found
```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
```

#### TypeScript Errors
```bash
# Check TypeScript configuration
npm run type-check

# Restart TypeScript service in VS Code
# Cmd/Ctrl + Shift + P -> "TypeScript: Restart TS Server"
```

#### Build Errors
```bash
# Clear build cache
rm -rf dist
npm run build
```

### Performance Issues

#### Slow Development Server
```bash
# Increase Node.js memory limit
export NODE_OPTIONS="--max-old-space-size=4096"
npm run dev
```

#### Large Bundle Size
```bash
# Analyze bundle
npm run analyze
```

## 🔧 Advanced Configuration

### Custom Vite Plugins

Add to `vite.config.ts`:

```typescript
export default defineConfig({
  plugins: [
    vue(),
    // Add your custom plugins here
  ],
  // Custom configuration
})
```

### TailwindCSS Customization

Modify `tailwind.config.ts`:

```typescript
export default {
  content: ['./src/**/*.{vue,js,ts}'],
  theme: {
    extend: {
      // Your customizations
    }
  }
}
```

## 📚 Next Steps

After setup, explore:

1. **[Architecture Guide](./ARCHITECTURE.md)** - Understand the codebase structure
2. **[Component Guide](./guides/COMPONENTS.md)** - Learn component development
3. **[API Guide](./guides/API.md)** - Understand API integration
4. **[Contributing Guide](./CONTRIBUTING.md)** - Start contributing

## 🆘 Getting Help

If you encounter issues:

1. Check this documentation
2. Search existing issues on GitHub
3. Create a new issue with detailed information
4. Ask in the development team chat

---

**Happy coding! 🚀**