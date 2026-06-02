//! A short tour of the PIN API: parse, inspect, transform, build, and validate.
//!
//! Run with `cargo run --example basic`.

use sashite_pin::{Identifier, Letter, ParseError, Side, State};

fn main() -> Result<(), ParseError> {
    // The case encodes the side, `+`/`-` the state, and `^` marks a terminal
    // piece. So "+K^" is an enhanced, terminal, first-side king.
    let king = Identifier::parse("+K^")?;
    println!("token         : {king}");
    println!("  letter      : {}", king.letter().as_char());
    println!("  side        : {:?}", king.side());
    println!("  state       : {:?}", king.state());
    println!("  terminal    : {}", king.is_terminal());
    println!("  is_first    : {}", king.is_first());
    println!("  is_enhanced : {}", king.is_enhanced());

    // Transformations return new values; the type is `Copy`, so this is cheap.
    let black = king.flipped();
    let plain = king.normalized().with_terminal(false);
    println!("flipped       : {black}");
    println!("normalized    : {plain}");

    // Building from typed components is infallible: every combination is valid.
    let pawn = Identifier::new(
        Letter::try_from_char('P')?,
        Side::Second,
        State::Enhanced,
        false,
    );
    println!("built         : {pawn} (a promoted second-side pawn)");

    // Checking validity without constructing an identifier.
    println!("is_valid(\"+r\"): {}", Identifier::is_valid("+r"));
    println!("is_valid(\"K+\"): {}", Identifier::is_valid("K+"));

    Ok(())
}
