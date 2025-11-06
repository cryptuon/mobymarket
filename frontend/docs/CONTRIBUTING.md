# 🤝 Contributing to Moby Market

Thank you for your interest in contributing to Moby Market! This guide will help you get started with contributing to our whale trading platform.

## 📋 Table of Contents

- [Code of Conduct](#-code-of-conduct)
- [Getting Started](#-getting-started)
- [Development Process](#-development-process)
- [Pull Request Process](#-pull-request-process)
- [Coding Standards](#-coding-standards)
- [Testing Guidelines](#-testing-guidelines)
- [Documentation](#-documentation)
- [Issue Reporting](#-issue-reporting)
- [Community](#-community)

## 📜 Code of Conduct

We are committed to fostering an open and welcoming environment. Please read and follow our Code of Conduct:

### Our Pledge

- **Be respectful**: Treat everyone with respect and kindness
- **Be inclusive**: Welcome contributors from all backgrounds
- **Be collaborative**: Work together towards common goals
- **Be professional**: Maintain professional communication

### Expected Behavior

- Use welcoming and inclusive language
- Respect differing viewpoints and experiences
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## 🚀 Getting Started

### Prerequisites

Before contributing, ensure you have:

1. **Development Environment**: Follow our [Setup Guide](./SETUP.md)
2. **Understanding**: Read the [Architecture Guide](./ARCHITECTURE.md)
3. **GitHub Account**: For submitting pull requests

### Your First Contribution

Looking for ways to contribute? Check out:

- **Good First Issues**: Tagged issues perfect for newcomers
- **Documentation**: Help improve our documentation
- **Bug Fixes**: Fix reported bugs
- **Feature Requests**: Implement requested features

### Setting Up Your Fork

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/moby-market.git
cd moby-market/frontend

# Add upstream remote
git remote add upstream https://github.com/original-org/moby-market.git

# Install dependencies
npm install
```

## 🔄 Development Process

### Branch Strategy

We follow the **Git Flow** branching strategy:

```
main                # Production-ready code
├── develop         # Integration branch
├── feature/*       # New features
├── bugfix/*        # Bug fixes
├── hotfix/*        # Critical production fixes
└── release/*       # Release preparation
```

### Creating a Feature Branch

```bash
# Update your local develop branch
git checkout develop
git pull upstream develop

# Create a feature branch
git checkout -b feature/amazing-new-feature

# Make your changes
# Commit your changes
git add .
git commit -m "feat: add amazing new feature"

# Push to your fork
git push origin feature/amazing-new-feature
```

### Commit Message Format

We follow the **Conventional Commits** specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

#### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **test**: Adding or fixing tests
- **chore**: Maintenance tasks

#### Examples

```bash
# Feature
git commit -m "feat(portfolio): add rebalancing wizard"

# Bug fix
git commit -m "fix(trading): resolve order book update issue"

# Documentation
git commit -m "docs: update API integration guide"

# Breaking change
git commit -m "feat!: change portfolio API structure

BREAKING CHANGE: portfolio endpoints now return different data structure"
```

## 🔍 Pull Request Process

### Before Submitting

1. **Test your changes**:
   ```bash
   npm run test
   npm run type-check
   npm run lint
   ```

2. **Update documentation** if needed

3. **Add tests** for new features

4. **Ensure build passes**:
   ```bash
   npm run build
   ```

### Creating a Pull Request

1. **Push your branch** to your fork
2. **Open a pull request** against the `develop` branch
3. **Fill out the PR template** completely
4. **Link related issues** using keywords (fixes #123)

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] New tests added
- [ ] Manual testing completed

## Screenshots (if applicable)
Add screenshots for UI changes

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No breaking changes (or properly documented)
```

### Review Process

1. **Automated checks** run on every PR
2. **Code review** by maintainers
3. **Testing** in staging environment
4. **Approval** from required reviewers
5. **Merge** after all checks pass

## 📏 Coding Standards

### TypeScript Guidelines

```typescript
// Use explicit types for function parameters and returns
function calculatePortfolioValue(positions: Position[]): number {
  return positions.reduce((total, position) => total + position.value, 0)
}

// Use interfaces for object structures
interface User {
  id: string
  email: string
  preferences: UserPreferences
}

// Use enums for constants
enum OrderType {
  MARKET = 'market',
  LIMIT = 'limit',
  STOP = 'stop'
}
```

### Vue Component Guidelines

```vue
<template>
  <!-- Use semantic HTML -->
  <main class="portfolio-dashboard">
    <header class="dashboard-header">
      <h1>Portfolio Dashboard</h1>
    </header>

    <!-- Use descriptive class names -->
    <section class="portfolio-overview">
      <PortfolioCard
        v-for="portfolio in portfolios"
        :key="portfolio.id"
        :portfolio="portfolio"
        @update="handlePortfolioUpdate"
      />
    </section>
  </main>
</template>

<script setup lang="ts">
// Import types
import type { Portfolio } from '@/types/portfolio'

// Define props with types
interface Props {
  portfolios: Portfolio[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// Define emits
const emit = defineEmits<{
  update: [portfolio: Portfolio]
}>()

// Use descriptive variable names
const handlePortfolioUpdate = (portfolio: Portfolio) => {
  emit('update', portfolio)
}
</script>

<style scoped>
/* Use meaningful class names */
.portfolio-dashboard {
  @apply min-h-screen bg-gray-50;
}

.dashboard-header {
  @apply bg-white shadow-sm p-6 mb-8;
}

.portfolio-overview {
  @apply grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6 p-6;
}
</style>
```

### CSS/TailwindCSS Guidelines

```css
/* Use utility classes when possible */
.btn-primary {
  @apply px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:ring-2 focus:ring-blue-500;
}

/* Group related utilities */
.card {
  @apply
    bg-white
    rounded-lg
    shadow-md
    border border-gray-200
    p-6
    hover:shadow-lg
    transition-shadow;
}

/* Use responsive prefixes appropriately */
.responsive-grid {
  @apply
    grid
    grid-cols-1
    md:grid-cols-2
    lg:grid-cols-3
    xl:grid-cols-4
    gap-4;
}
```

### File Organization

```
components/
├── ui/                   # Reusable UI components
│   ├── Button.vue        # Single responsibility
│   ├── Card.vue          # Well-documented
│   └── Modal.vue         # Consistent API
├── portfolio/            # Feature-specific components
│   ├── PortfolioCard.vue
│   └── PortfolioForm.vue
└── trading/
    ├── OrderBook.vue
    └── TradingChart.vue
```

## 🧪 Testing Guidelines

### Unit Testing

```typescript
// tests/components/Button.test.ts
import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import Button from '@/components/ui/Button.vue'

describe('Button', () => {
  it('renders with correct text', () => {
    const wrapper = mount(Button, {
      slots: {
        default: 'Click me'
      }
    })

    expect(wrapper.text()).toBe('Click me')
  })

  it('emits click event', async () => {
    const wrapper = mount(Button)

    await wrapper.trigger('click')

    expect(wrapper.emitted('click')).toBeTruthy()
  })

  it('applies variant classes correctly', () => {
    const wrapper = mount(Button, {
      props: {
        variant: 'primary'
      }
    })

    expect(wrapper.classes()).toContain('btn-primary')
  })
})
```

### Integration Testing

```typescript
// tests/pages/Portfolio.test.ts
import { mount } from '@vue/test-utils'
import { createTestingPinia } from '@pinia/testing'
import Portfolio from '@/pages/Portfolio.vue'

describe('Portfolio Page', () => {
  it('displays portfolio data correctly', async () => {
    const wrapper = mount(Portfolio, {
      global: {
        plugins: [createTestingPinia({
          initialState: {
            portfolio: {
              portfolios: [mockPortfolio]
            }
          }
        })]
      }
    })

    expect(wrapper.find('[data-testid="portfolio-value"]').text()).toBe('$125,000')
  })
})
```

### Test Coverage

Maintain minimum coverage levels:
- **Statements**: 80%
- **Branches**: 75%
- **Functions**: 80%
- **Lines**: 80%

```bash
# Run tests with coverage
npm run test:coverage
```

## 📖 Documentation

### Code Documentation

```typescript
/**
 * Calculates the total value of a portfolio
 * @param positions - Array of portfolio positions
 * @param includeUnrealized - Whether to include unrealized gains/losses
 * @returns Total portfolio value in USD
 * @example
 * ```typescript
 * const value = calculatePortfolioValue(positions, true)
 * console.log(`Portfolio worth: $${value}`)
 * ```
 */
function calculatePortfolioValue(
  positions: Position[],
  includeUnrealized: boolean = true
): number {
  // Implementation...
}
```

### Component Documentation

```vue
<template>
  <!-- Component template -->
</template>

<script setup lang="ts">
/**
 * PortfolioCard - Displays portfolio summary information
 *
 * @component
 * @example
 * ```vue
 * <PortfolioCard
 *   :portfolio="portfolio"
 *   :loading="false"
 *   @select="handlePortfolioSelect"
 * />
 * ```
 */

interface Props {
  /** Portfolio data to display */
  portfolio: Portfolio
  /** Loading state indicator */
  loading?: boolean
}

const emit = defineEmits<{
  /** Emitted when portfolio is selected */
  select: [portfolio: Portfolio]
}>()
</script>
```

### API Documentation

```typescript
/**
 * Portfolio API Service
 * Handles all portfolio-related API operations
 */
export class PortfolioService {
  /**
   * Retrieves all portfolios for the authenticated user
   * @returns Promise resolving to array of portfolios
   * @throws {ApiError} When request fails
   */
  async getPortfolios(): Promise<Portfolio[]> {
    // Implementation...
  }
}
```

## 🐛 Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
**Bug Description**
Clear description of the bug

**Steps to Reproduce**
1. Go to '...'
2. Click on '...'
3. See error

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: [e.g., macOS 12.0]
- Browser: [e.g., Chrome 95.0]
- Node.js: [e.g., 18.0.0]

**Screenshots**
Add screenshots if applicable

**Additional Context**
Any other relevant information
```

### Feature Requests

Use the feature request template:

```markdown
**Feature Description**
Clear description of the feature

**Use Case**
Why is this feature needed?

**Proposed Solution**
How should this be implemented?

**Alternatives**
Other solutions considered

**Additional Context**
Any other relevant information
```

## 👥 Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time chat and collaboration
- **Email**: security@mobymarket.com for security issues

### Getting Help

1. **Search existing issues** before creating new ones
2. **Check documentation** for answers
3. **Ask in discussions** for general questions
4. **Create detailed issues** for bugs

### Code Reviews

When reviewing code:

1. **Be constructive**: Provide helpful feedback
2. **Be specific**: Point out exact issues
3. **Be respectful**: Maintain professional tone
4. **Be thorough**: Check logic, style, and tests

### Recognition

Contributors are recognized through:

- **Changelog mentions** for significant contributions
- **GitHub contributors** page
- **Special thanks** in release notes

## 📚 Resources

### Learning Resources

- **[Vue.js Guide](https://vuejs.org/guide/)**
- **[TypeScript Handbook](https://www.typescriptlang.org/docs/)**
- **[TailwindCSS Docs](https://tailwindcss.com/docs)**
- **[Testing Library](https://testing-library.com/docs/vue-testing-library/intro/)**

### Tools

- **[Vue DevTools](https://devtools.vuejs.org/)**
- **[VS Code Extensions](https://marketplace.visualstudio.com/items?itemName=Vue.volar)**
- **[GitHub CLI](https://cli.github.com/)**

## 🎉 Thank You!

Thank you for contributing to Moby Market! Your efforts help make this platform better for everyone in the crypto trading community.

---

**Questions?** Feel free to reach out through any of our communication channels. We're here to help!