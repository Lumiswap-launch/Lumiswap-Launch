//! Automated Market Maker (AMM) math for bonding curve.
//!
//! Uses constant product formula: x * y = k
//! where x = virtual_xlm, y = virtual_tokens

use crate::errors::Error;
use crate::types::CurveState;

/// Calculate tokens received for XLM input.
///
/// Formula: Δy = y - (k / (x + Δx))
/// where Δx = xlm_in
pub fn calculate_tokens_out(xlm_in: i128, curve: &CurveState) -> Result<i128, Error> {
    if xlm_in <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Calculate new XLM reserve
    let new_xlm = curve
        .virtual_xlm
        .checked_add(xlm_in)
        .ok_or(Error::MathOverflow)?;

    // Calculate new token reserve: new_tokens = k / new_xlm
    let new_tokens = curve.k.checked_div(new_xlm).ok_or(Error::DivisionByZero)?;

    // Tokens out = old_tokens - new_tokens
    let tokens_out = curve
        .virtual_tokens
        .checked_sub(new_tokens)
        .ok_or(Error::MathOverflow)?;

    if tokens_out <= 0 {
        return Err(Error::InvalidAmount);
    }

    Ok(tokens_out)
}

/// Calculate XLM received for token input.
///
/// Formula: Δx = x - (k / (y + Δy))
/// where Δy = tokens_in
pub fn calculate_xlm_out(tokens_in: i128, curve: &CurveState) -> Result<i128, Error> {
    if tokens_in <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Calculate new token reserve
    let new_tokens = curve
        .virtual_tokens
        .checked_add(tokens_in)
        .ok_or(Error::MathOverflow)?;

    // Calculate new XLM reserve: new_xlm = k / new_tokens
    let new_xlm = curve.k.checked_div(new_tokens).ok_or(Error::DivisionByZero)?;

    // XLM out = old_xlm - new_xlm
    let xlm_out = curve
        .virtual_xlm
        .checked_sub(new_xlm)
        .ok_or(Error::MathOverflow)?;

    if xlm_out <= 0 {
        return Err(Error::InvalidAmount);
    }

    Ok(xlm_out)
}

/// Calculate current spot price: XLM per token.
///
/// Formula: price = virtual_xlm / virtual_tokens
///
/// Returns price with 7 decimal precision (stroops per token unit).
pub fn calculate_spot_price(curve: &CurveState) -> Result<i128, Error> {
    if curve.virtual_tokens == 0 {
        return Err(Error::DivisionByZero);
    }

    // Price = virtual_xlm / virtual_tokens
    // Multiply by 10^7 for precision
    let price = curve
        .virtual_xlm
        .checked_mul(10_000_000)
        .ok_or(Error::MathOverflow)?
        .checked_div(curve.virtual_tokens)
        .ok_or(Error::DivisionByZero)?;

    Ok(price)
}

/// Calculate average execution price for a trade.
///
/// Returns price with 7 decimal precision.
pub fn calculate_avg_price(amount_in: i128, amount_out: i128) -> Result<i128, Error> {
    if amount_out == 0 {
        return Err(Error::DivisionByZero);
    }

    let avg_price = amount_in
        .checked_mul(10_000_000)
        .ok_or(Error::MathOverflow)?
        .checked_div(amount_out)
        .ok_or(Error::DivisionByZero)?;

    Ok(avg_price)
}

/// Calculate price impact percentage for a buy.
///
/// Returns basis points (10000 = 100%).
pub fn calculate_buy_price_impact(
    xlm_in: i128,
    curve: &CurveState,
) -> Result<u32, Error> {
    let spot_price_before = calculate_spot_price(curve)?;
    let tokens_out = calculate_tokens_out(xlm_in, curve)?;
    let avg_price = calculate_avg_price(xlm_in, tokens_out)?;

    // Impact = (avg_price - spot_price) / spot_price * 10000
    let price_diff = avg_price.checked_sub(spot_price_before).ok_or(Error::MathOverflow)?;
    
    if price_diff < 0 {
        return Ok(0);
    }

    let impact = price_diff
        .checked_mul(10_000)
        .ok_or(Error::MathOverflow)?
        .checked_div(spot_price_before)
        .ok_or(Error::DivisionByZero)?;

    Ok(impact.min(10_000) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_curve() -> CurveState {
        let virtual_xlm = 30_000 * 10_000_000i128; // 30,000 XLM
        let virtual_tokens = 1_000_000 * 10_000_000i128; // 1M tokens
        CurveState {
            virtual_xlm,
            virtual_tokens,
            k: virtual_xlm * virtual_tokens,
        }
    }

    #[test]
    fn test_buy_tokens() {
        let curve = create_test_curve();
        let xlm_in = 1_000 * 10_000_000i128; // 1000 XLM
        
        let tokens_out = calculate_tokens_out(xlm_in, &curve).unwrap();
        
        // Should receive approximately 32,258 tokens
        assert!(tokens_out > 30_000 * 10_000_000);
        assert!(tokens_out < 35_000 * 10_000_000);
    }

    #[test]
    fn test_sell_tokens() {
        let curve = create_test_curve();
        let tokens_in = 10_000 * 10_000_000i128; // 10,000 tokens
        
        let xlm_out = calculate_xlm_out(tokens_in, &curve).unwrap();
        
        // Should receive approximately 297 XLM
        assert!(xlm_out > 0);
        assert!(xlm_out < 300 * 10_000_000);
    }

    #[test]
    fn test_spot_price() {
        let curve = create_test_curve();
        let price = calculate_spot_price(&curve).unwrap();
        
        // Price should be 0.03 XLM per token = 300,000 stroops
        assert_eq!(price, 300_000);
    }

    #[test]
    fn test_price_impact() {
        let curve = create_test_curve();
        let xlm_in = 10_000 * 10_000_000i128; // Large buy: 10,000 XLM
        
        let impact = calculate_buy_price_impact(xlm_in, &curve).unwrap();
        
        // Large buy should have significant price impact
        assert!(impact > 1500); // > 15%
    }

    #[test]
    fn test_zero_amount_fails() {
        let curve = create_test_curve();
        
        assert_eq!(
            calculate_tokens_out(0, &curve),
            Err(Error::InvalidAmount)
        );
        
        assert_eq!(
            calculate_xlm_out(0, &curve),
            Err(Error::InvalidAmount)
        );
    }

    #[test]
    fn test_constant_product_preserved() {
        let mut curve = create_test_curve();
        let initial_k = curve.k;
        
        // Simulate a buy
        let xlm_in = 1_000 * 10_000_000i128;
        let tokens_out = calculate_tokens_out(xlm_in, &curve).unwrap();
        
        curve.virtual_xlm += xlm_in;
        curve.virtual_tokens -= tokens_out;
        
        let new_k = curve.virtual_xlm * curve.virtual_tokens;
        
        // k should remain constant (within rounding error)
        assert!(new_k >= initial_k);
        assert!(new_k - initial_k < initial_k / 1000); // < 0.1% error
    }
}
