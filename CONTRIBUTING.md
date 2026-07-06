# Contributing to Lumiswap Launch

Thank you for your interest in contributing to Lumiswap Launch! This document provides guidelines and instructions for contributors.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Areas for Contribution](#areas-for-contribution)

---

## 📜 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of background or identity.

### Expected Behavior

- Be respectful and considerate
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Respect differing viewpoints and experiences

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Publishing others' private information
- Other conduct inappropriate in a professional setting

Report violations to: conduct@lumiswap.io

---

## 🚀 Getting Started

### Prerequisites

```bash
# Rust 1.74+
rustup update stable
rustup target add wasm32-unknown-unknown

# Stellar CLI
cargo install --locked stellar-cli

# Node.js 18+
node --version  # should be 18+
```

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/lumiswap-launch
cd lumiswap-launch

# Add upstream remote
git remote add upstream https://github.com/lumiswap/lumiswap-launch
```

### Build and Test

```bash
# Contract
cd contract
cargo test
cargo build --target wasm32-unknown-unknown --release

# Frontend
cd ../frontend
npm install
npm run dev
```

---

## 🔄 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feat/your-feature-name
```

**Branch naming conventions:**
- `feat/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions or fixes
- `chore/` - Maintenance tasks

### 2. Make Changes

- Write clean, well-commented code
- Follow project coding standards
- Add tests for new functionality
- Update documentation as needed

### 3. Commit Changes

Use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat: add buy slippage calculator"
git commit -m "fix: resolve overflow in AMM calculation"
git commit -m "docs: update deployment instructions"
```

**Commit types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code restructuring
- `test`: Adding or fixing tests
- `chore`: Maintenance

### 4. Keep Your Fork Updated

```bash
git fetch upstream
git rebase upstream/main
```

### 5. Push and Create PR

```bash
git push origin feat/your-feature-name
```

Then open a Pull Request on GitHub.

---

## 📏 Coding Standards

### Rust (Contract)

```rust
// Use descriptive names
fn calculate_tokens_out(xlm_in: i128, curve: &CurveState) -> Result<i128, Error> {
    // Always handle errors explicitly
    let new_xlm = curve.virtual_xlm
        .checked_add(xlm_in)
        .ok_or(Error::MathOverflow)?;
    
    // Document complex logic
    // Formula: tokens_out = y - (k / (x + dx))
    let new_tokens = curve.k
        .checked_div(new_xlm)
        .ok_or(Error::DivisionByZero)?;
    
    Ok(curve.virtual_tokens - new_tokens)
}
```

**Guidelines:**
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix all warnings
- Use `Result<T, Error>` for fallible operations
- Prefer `checked_*` math operations
- Document public functions with `///` comments
- Keep functions focused and under 50 lines
- Use descriptive variable names

### TypeScript (Frontend)

```typescript
/**
 * Calculate slippage percentage for a trade
 * @param expected - Expected output amount
 * @param actual - Actual output amount
 * @returns Slippage percentage (0-100)
 */
export function calculateSlippage(
    expected: bigint,
    actual: bigint
): number {
    if (expected === 0n) return 0;
    
    const diff = expected - actual;
    const slippage = Number(diff * 10000n / expected) / 100;
    
    return Math.max(0, slippage);
}
```

**Guidelines:**
- Use TypeScript strict mode
- No `any` types (use `unknown` if necessary)
- Prefer `const` over `let`
- Use async/await over promises
- Handle errors explicitly
- Document exported functions with JSDoc
- Use meaningful variable names
- Keep functions pure when possible

### Code Formatting

**Rust:**
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

**TypeScript:**
```bash
npm run lint
npm run type-check
```

---

## 🧪 Testing Guidelines

### Contract Tests

```rust
#[test]
fn test_buy_with_slippage_protection() {
    let (env, contract, buyer) = setup_test();
    
    // Get expected output
    let expected = contract.get_buy_quote(&0, &1000);
    
    // Try to buy with too high min_tokens (should fail)
    let result = contract.try_buy(
        &buyer,
        &0,
        &1000,
        &(expected + 1)
    );
    
    assert_eq!(result, Err(Ok(Error::SlippageExceeded)));
}
```

**Guidelines:**
- Test happy paths and error cases
- Use descriptive test names: `test_<function>_<scenario>`
- One assertion per test when possible
- Use `setup_test()` helpers for common setup
- Test edge cases (zero, max values, overflow)
- Run full test suite: `cargo test`

### Frontend Tests

```typescript
describe('calculateSlippage', () => {
    it('should return 0 for equal amounts', () => {
        expect(calculateSlippage(100n, 100n)).toBe(0);
    });
    
    it('should calculate 5% slippage correctly', () => {
        expect(calculateSlippage(100n, 95n)).toBe(5);
    });
    
    it('should handle zero expected amount', () => {
        expect(calculateSlippage(0n, 100n)).toBe(0);
    });
});
```

**Guidelines:**
- Test user interactions
- Test error handling
- Test edge cases
- Use descriptive test names
- Mock external dependencies

---

## 🔀 Pull Request Process

### Before Submitting

- [ ] Code follows project style guidelines
- [ ] All tests pass: `cargo test` and `npm test`
- [ ] No linting errors: `cargo clippy` and `npm run lint`
- [ ] Documentation updated if needed
- [ ] Commits follow Conventional Commits format
- [ ] Branch is up to date with main

### PR Title

Use Conventional Commits format:

```
feat: add sell slippage tolerance setting
fix: resolve price calculation overflow
docs: update deployment script examples
```

### PR Description Template

```markdown
## Description
Brief description of what this PR does.

## Motivation
Why is this change needed?

## Changes
- List key changes
- One per line

## Testing
How was this tested?
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manually tested on testnet

## Screenshots (if applicable)
Add screenshots for UI changes

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests pass
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by at least one maintainer
3. **Testing** on testnet for contract changes
4. **Approval** from maintainer
5. **Merge** to main branch

### After Merge

- Your branch will be deleted
- Changes deployed to testnet
- Listed in changelog for next release

---

## 🎯 Areas for Contribution

### High Priority

| Area | Difficulty | Description |
|------|-----------|-------------|
| **Wallet Integration** | Medium | Complete Freighter integration |
| **DEX Migration** | Hard | Implement Stellar DEX liquidity seeding |
| **Price Charts** | Medium | Real-time price chart components |
| **Mobile UI** | Easy | Responsive mobile optimizations |

### Contract Improvements

- [ ] Gas optimization for buy/sell functions
- [ ] Emergency pause mechanism
- [ ] Multi-signature admin functions
- [ ] Configurable curve parameters per launch
- [ ] Trading volume analytics

### Frontend Features

- [ ] Launch explorer/search
- [ ] User portfolio tracking
- [ ] Price alerts
- [ ] Historical data visualization
- [ ] Dark/light theme toggle

### Documentation

- [ ] Video tutorials
- [ ] Architecture diagrams
- [ ] API reference
- [ ] Integration examples
- [ ] Troubleshooting guide

### Testing

- [ ] Fuzz testing for AMM math
- [ ] End-to-end testnet scenarios
- [ ] Load testing
- [ ] Security test cases
- [ ] Frontend unit test coverage

---

## 💬 Getting Help

- **Discord**: [Join our community](https://discord.gg/lumiswap)
- **GitHub Discussions**: Ask questions and share ideas
- **Email**: dev@lumiswap.io

---

## 🏆 Recognition

Contributors are recognized in:
- README.md contributors section
- Release notes
- Project website

Significant contributors may be invited to join the core team.

---

## 📝 License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for contributing to Lumiswap Launch! 🚀
